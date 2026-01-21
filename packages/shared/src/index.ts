// Types shared between frontend and backend

export interface Card {
  id: string;
  front: string;
  back: string;
  source_id: string | null;
  ease_factor: number;
  interval: number;
  repetitions: number;
  next_review: string; // ISO date string
  created_at: string;
  updated_at: string;
}

export interface CreateCard {
  front: string;
  back: string;
  source_id?: string;
}

export interface UpdateCard {
  front?: string;
  back?: string;
}

export type ReviewRating = 'again' | 'hard' | 'good' | 'easy';

export interface SubmitReview {
  card_id: string;
  rating: ReviewRating;
  user_answer?: string;
}

export interface Review {
  id: string;
  card_id: string;
  rating: number;
  user_answer: string | null;
  llm_evaluation: string | null;
  reviewed_at: string;
}

export type SourceType = 'manual' | 'text' | 'pdf' | 'url';

export interface Source {
  id: string;
  source_type: SourceType;
  title: string | null;
  url: string | null;
  content_hash: string | null;
  created_at: string;
}

// Voice session types (for Phase 4)
export type SessionState =
  | 'idle'
  | 'presenting_card'
  | 'waiting_for_answer'
  | 'evaluating'
  | 'showing_feedback';

export type ReviewMode = 'manual' | 'oral' | 'conversational';

export interface VoiceSessionMessage {
  type:
    | 'start_session'
    | 'end_session'
    | 'audio_chunk'
    | 'end_audio'
    | 'command'
    | 'text_answer'
    | 'rate_card'
    | 'next_card'
    | 'skip_card'
    | 'replay_card';
  data?: unknown;
  card_limit?: number;
  review_mode?: ReviewMode;
  answer?: string;
  rating?: ReviewRating | number;
}

export interface VoiceSessionResponse {
  type:
    | 'card_presented'
    | 'card_replay'
    | 'card_rated'
    | 'audio_chunk'
    | 'transcription'
    | 'evaluation'
    | 'session_state'
    | 'session_started'
    | 'session_ended'
    | 'session_complete'
    | 'session_intro'
    | 'state_change'
    | 'vad_status'
    | 'error';
  data?: unknown;
  // Common fields that may be present
  card_id?: string;
  front?: string;
  spoken_text?: string;
  audio?: string;
  audio_duration?: number;
  sample_rate?: number;
  card_number?: number;
  total_cards?: number;
  text?: string;
  confidence?: number;
  duration?: number;
  rating?: number;
  is_correct?: boolean;
  feedback?: string;
  expected_answer?: string;
  user_answer?: string;
  auto_advance?: boolean;
  auto_rated?: boolean;
  review_mode?: ReviewMode;
  stats?: {
    cards_reviewed: number;
    correct_count: number;
    incorrect_count: number;
    accuracy: number;
    total_audio_duration?: number;
  };
  message?: string;
  state?: string;
  speech_probability?: number;
  is_speech?: boolean;
}
