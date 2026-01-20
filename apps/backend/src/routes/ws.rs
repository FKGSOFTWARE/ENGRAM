use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::llm::EvaluationRequest;
use crate::models::Card;
use crate::AppState;

// Voice session state machine
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Idle,
    PresentingCard,
    WaitingForAnswer,
    Evaluating,
    ShowingFeedback,
}

// Client -> Server messages
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    StartSession { card_limit: Option<i32> },
    EndSession,
    AudioChunk { data: Vec<u8> },
    EndAudio,
    Command { action: String },
    TextAnswer { answer: String },
    NextCard,
    RateCard { rating: String },
}

// Server -> Client messages
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)] // AudioChunk reserved for future TTS streaming
pub enum ServerMessage {
    SessionStarted { card_count: i32 },
    SessionEnded,
    StateChanged { state: SessionState },
    CardPresented { card: CardPresentation },
    AudioChunk { data: Vec<u8> },
    Evaluation { is_correct: bool, score: f32, feedback: String, suggested_rating: String },
    Error { message: String },
    SessionComplete { cards_reviewed: i32 },
}

#[derive(Debug, Serialize)]
pub struct CardPresentation {
    pub id: String,
    pub front: String,
    pub index: i32,
    pub total: i32,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Session state
    let mut session_state = SessionState::Idle;
    let mut cards: Vec<Card> = Vec::new();
    let mut current_index: usize = 0;
    let mut cards_reviewed = 0;

    // Audio buffer for incoming audio chunks
    let mut audio_buffer: Vec<u8> = Vec::new();

    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Binary(data)) => {
                // Binary data is audio
                audio_buffer.extend_from_slice(&data);
                continue;
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        };

        // Parse client message
        let client_msg: ClientMessage = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(e) => {
                let error = ServerMessage::Error {
                    message: format!("Invalid message: {}", e),
                };
                let _ = sender.send(Message::Text(serde_json::to_string(&error).unwrap().into())).await;
                continue;
            }
        };

        // Handle message based on current state
        let response = match client_msg {
            ClientMessage::StartSession { card_limit } => {
                let limit = card_limit.unwrap_or(10);
                let now = chrono::Utc::now();

                // Fetch due cards
                match sqlx::query_as::<_, Card>(
                    "SELECT * FROM cards WHERE next_review <= ? ORDER BY next_review ASC LIMIT ?",
                )
                .bind(&now)
                .bind(limit)
                .fetch_all(&state.db)
                .await
                {
                    Ok(fetched_cards) => {
                        let count = fetched_cards.len() as i32;
                        cards = fetched_cards;
                        current_index = 0;
                        cards_reviewed = 0;

                        if cards.is_empty() {
                            session_state = SessionState::Idle;
                            ServerMessage::SessionComplete { cards_reviewed: 0 }
                        } else {
                            session_state = SessionState::PresentingCard;
                            ServerMessage::SessionStarted { card_count: count }
                        }
                    }
                    Err(e) => ServerMessage::Error {
                        message: format!("Failed to fetch cards: {}", e),
                    },
                }
            }

            ClientMessage::EndSession => {
                session_state = SessionState::Idle;
                cards.clear();
                ServerMessage::SessionEnded
            }

            ClientMessage::NextCard => {
                if session_state == SessionState::Idle {
                    ServerMessage::Error {
                        message: "No active session".to_string(),
                    }
                } else if current_index >= cards.len() {
                    session_state = SessionState::Idle;
                    ServerMessage::SessionComplete { cards_reviewed }
                } else {
                    session_state = SessionState::PresentingCard;
                    let card = &cards[current_index];
                    ServerMessage::CardPresented {
                        card: CardPresentation {
                            id: card.id.clone(),
                            front: card.front.clone(),
                            index: current_index as i32 + 1,
                            total: cards.len() as i32,
                        },
                    }
                }
            }

            ClientMessage::TextAnswer { answer } => {
                if session_state != SessionState::PresentingCard && session_state != SessionState::WaitingForAnswer {
                    ServerMessage::Error {
                        message: "Not waiting for answer".to_string(),
                    }
                } else if current_index >= cards.len() {
                    ServerMessage::Error {
                        message: "No current card".to_string(),
                    }
                } else {
                    session_state = SessionState::Evaluating;

                    // Send state change
                    let state_msg = ServerMessage::StateChanged { state: session_state };
                    let _ = sender.send(Message::Text(serde_json::to_string(&state_msg).unwrap().into())).await;

                    let card = &cards[current_index];

                    // Evaluate with LLM if available
                    if state.llm.has_available_provider() {
                        let request = EvaluationRequest {
                            card_front: card.front.clone(),
                            card_back: card.back.clone(),
                            user_answer: answer,
                        };

                        match state.llm.evaluate_answer(request).await {
                            Ok(eval) => {
                                session_state = SessionState::ShowingFeedback;
                                ServerMessage::Evaluation {
                                    is_correct: eval.is_correct,
                                    score: eval.score,
                                    feedback: eval.feedback,
                                    suggested_rating: format!("{:?}", eval.suggested_rating).to_lowercase(),
                                }
                            }
                            Err(e) => {
                                session_state = SessionState::ShowingFeedback;
                                ServerMessage::Error {
                                    message: format!("Evaluation failed: {}", e),
                                }
                            }
                        }
                    } else {
                        session_state = SessionState::ShowingFeedback;
                        ServerMessage::Error {
                            message: "No LLM provider available for evaluation".to_string(),
                        }
                    }
                }
            }

            ClientMessage::RateCard { rating } => {
                if session_state != SessionState::ShowingFeedback {
                    ServerMessage::Error {
                        message: "Not in feedback state".to_string(),
                    }
                } else if current_index >= cards.len() {
                    ServerMessage::Error {
                        message: "No current card".to_string(),
                    }
                } else {
                    let card = &cards[current_index];
                    let rating_enum = match rating.as_str() {
                        "again" => crate::models::ReviewRating::Again,
                        "hard" => crate::models::ReviewRating::Hard,
                        "good" => crate::models::ReviewRating::Good,
                        "easy" => crate::models::ReviewRating::Easy,
                        _ => crate::models::ReviewRating::Good,
                    };

                    // Build FSRS state from card
                    let fsrs_state = crate::services::spaced_repetition::FSRSState {
                        stability: card.stability,
                        difficulty: card.difficulty,
                        reps: card.repetitions,
                        lapses: card.lapses,
                        last_review: card.last_review,
                    };

                    // Calculate new scheduling using FSRS directly
                    let (new_interval, new_stability, new_difficulty, new_reps, new_lapses, next_review) =
                        crate::services::spaced_repetition::calculate_fsrs(&fsrs_state, rating_enum, None);

                    // Map difficulty back to ease_factor for backward compatibility
                    let new_ease_factor = (2.5 - (new_difficulty - 1.0) * 0.17).clamp(1.3, 3.0);

                    let now = chrono::Utc::now();
                    let _ = sqlx::query(
                        "UPDATE cards SET interval = ?, ease_factor = ?, repetitions = ?, stability = ?, difficulty = ?, lapses = ?, next_review = ?, last_review = ?, updated_at = ? WHERE id = ?",
                    )
                    .bind(new_interval)
                    .bind(new_ease_factor)
                    .bind(new_reps)
                    .bind(new_stability)
                    .bind(new_difficulty)
                    .bind(new_lapses)
                    .bind(&next_review)
                    .bind(&now)  // last_review is now
                    .bind(&now)  // updated_at
                    .bind(&card.id)
                    .execute(&state.db)
                    .await;

                    // Record review
                    let review = crate::models::Review::new(card.id.clone(), rating_enum, None);
                    let _ = sqlx::query(
                        "INSERT INTO reviews (id, card_id, rating, reviewed_at) VALUES (?, ?, ?, ?)",
                    )
                    .bind(&review.id)
                    .bind(&review.card_id)
                    .bind(review.rating)
                    .bind(&review.reviewed_at)
                    .execute(&state.db)
                    .await;

                    current_index += 1;
                    cards_reviewed += 1;

                    // Move to next card or complete
                    if current_index >= cards.len() {
                        session_state = SessionState::Idle;
                        ServerMessage::SessionComplete { cards_reviewed }
                    } else {
                        session_state = SessionState::PresentingCard;
                        let next_card = &cards[current_index];
                        ServerMessage::CardPresented {
                            card: CardPresentation {
                                id: next_card.id.clone(),
                                front: next_card.front.clone(),
                                index: current_index as i32 + 1,
                                total: cards.len() as i32,
                            },
                        }
                    }
                }
            }

            ClientMessage::AudioChunk { data } => {
                audio_buffer.extend_from_slice(&data);
                continue; // Don't send a response for audio chunks
            }

            ClientMessage::EndAudio => {
                // Process audio buffer (placeholder for Gemini Live API integration)
                // In a full implementation, this would send to speech-to-text
                audio_buffer.clear();
                session_state = SessionState::WaitingForAnswer;
                ServerMessage::StateChanged { state: session_state }
            }

            ClientMessage::Command { action } => {
                match action.as_str() {
                    "skip" => {
                        if current_index < cards.len() {
                            current_index += 1;
                            if current_index >= cards.len() {
                                session_state = SessionState::Idle;
                                ServerMessage::SessionComplete { cards_reviewed }
                            } else {
                                session_state = SessionState::PresentingCard;
                                let card = &cards[current_index];
                                ServerMessage::CardPresented {
                                    card: CardPresentation {
                                        id: card.id.clone(),
                                        front: card.front.clone(),
                                        index: current_index as i32 + 1,
                                        total: cards.len() as i32,
                                    },
                                }
                            }
                        } else {
                            ServerMessage::Error {
                                message: "No card to skip".to_string(),
                            }
                        }
                    }
                    "repeat" => {
                        if current_index < cards.len() {
                            session_state = SessionState::PresentingCard;
                            let card = &cards[current_index];
                            ServerMessage::CardPresented {
                                card: CardPresentation {
                                    id: card.id.clone(),
                                    front: card.front.clone(),
                                    index: current_index as i32 + 1,
                                    total: cards.len() as i32,
                                },
                            }
                        } else {
                            ServerMessage::Error {
                                message: "No card to repeat".to_string(),
                            }
                        }
                    }
                    _ => ServerMessage::Error {
                        message: format!("Unknown command: {}", action),
                    },
                }
            }
        };

        // Send response
        let response_json = serde_json::to_string(&response).unwrap();
        if sender.send(Message::Text(response_json.into())).await.is_err() {
            break;
        }
    }

    tracing::debug!("WebSocket connection closed");
}
