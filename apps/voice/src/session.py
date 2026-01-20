"""
Voice session state machine for ENGRAM.

Manages the flow of voice-based flashcard review sessions:
1. Start session with deck selection
2. Present card front via TTS
3. Capture user's spoken answer
4. Transcribe and evaluate answer
5. Present feedback and continue
"""

import asyncio
import base64
import json
import logging
import uuid
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Optional

import numpy as np
from fastapi import WebSocket

from .api_client import EngramAPIClient
from .pipeline import VoicePipeline
from .prompts import build_evaluation_prompt

logger = logging.getLogger(__name__)


class SessionState(Enum):
    """Voice session states."""

    IDLE = "idle"
    STARTING = "starting"
    PRESENTING_CARD = "presenting_card"
    LISTENING = "listening"
    PROCESSING = "processing"
    EVALUATING = "evaluating"
    PRESENTING_FEEDBACK = "presenting_feedback"
    ENDING = "ending"
    ENDED = "ended"
    ERROR = "error"


@dataclass
class SessionStats:
    """Statistics for a voice session."""

    cards_reviewed: int = 0
    correct_count: int = 0
    incorrect_count: int = 0
    total_audio_duration: float = 0.0
    session_duration: float = 0.0


@dataclass
class VoiceSession:
    """
    Voice-based flashcard review session.

    Handles the WebSocket connection and coordinates between
    the voice pipeline, API client, and session state machine.
    """

    websocket: WebSocket
    pipeline: VoicePipeline
    api_base_url: str
    streaming: bool = False

    session_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    state: SessionState = SessionState.IDLE
    stats: SessionStats = field(default_factory=SessionStats)

    _api_client: Optional[EngramAPIClient] = field(default=None, init=False)
    _current_card: Optional[dict] = field(default=None, init=False)
    _audio_buffer: bytes = field(default=b"", init=False)
    _deck_id: Optional[str] = field(default=None, init=False)

    def __post_init__(self):
        self._api_client = EngramAPIClient(self.api_base_url)

    async def run(self) -> None:
        """Main session loop for command-based interaction."""
        self.state = SessionState.IDLE

        while self.state != SessionState.ENDED:
            try:
                message = await self.websocket.receive_json()
                await self._handle_message(message)
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Session error: {e}", exc_info=True)
                self.state = SessionState.ERROR
                await self._send_error(str(e))
                break

    async def run_streaming(self) -> None:
        """Main session loop for continuous audio streaming."""
        self.state = SessionState.IDLE

        while self.state != SessionState.ENDED:
            try:
                data = await self.websocket.receive()

                # Check for disconnect
                if data.get("type") == "websocket.disconnect":
                    logger.info("WebSocket disconnected by client")
                    break

                if "text" in data:
                    # Parse the JSON from the already-received text message
                    message = json.loads(data["text"])
                    await self._handle_message(message)
                elif "bytes" in data:
                    await self._handle_audio_chunk(data["bytes"])
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Streaming session error: {e}", exc_info=True)
                self.state = SessionState.ERROR
                # Only try to send error if it's not a connection error
                if "disconnect" not in str(e).lower() and "closed" not in str(e).lower():
                    try:
                        await self._send_error(str(e))
                    except Exception:
                        pass  # Connection already closed
                break

    async def _handle_message(self, message: dict) -> None:
        """Handle incoming WebSocket message."""
        msg_type = message.get("type")

        if msg_type == "start_session":
            await self._start_session(message.get("deck_id"))

        elif msg_type == "end_session":
            await self._end_session()

        elif msg_type == "audio_chunk":
            # Base64-encoded audio data
            audio_b64 = message.get("audio", "")
            audio_bytes = base64.b64decode(audio_b64)
            await self._handle_audio_chunk(audio_bytes)

        elif msg_type == "end_audio":
            await self._process_audio()

        elif msg_type == "skip_card":
            await self._skip_card()

        elif msg_type == "rate_card":
            rating = message.get("rating", 2)
            await self._rate_card(rating)

        elif msg_type == "next_card":
            await self._present_next_card()

        elif msg_type == "replay_card":
            await self._replay_card()

        else:
            logger.warning(f"Unknown message type: {msg_type}")

    async def _start_session(self, deck_id: Optional[str]) -> None:
        """Start a new review session."""
        if self.state != SessionState.IDLE:
            await self._send_error("Session already in progress")
            return

        self.state = SessionState.STARTING
        self._deck_id = deck_id

        try:
            await self._send_state_update("starting")

            # Get due cards from API
            cards = await self._api_client.get_due_cards(deck_id)

            if not cards:
                await self.websocket.send_json({
                    "type": "session_complete",
                    "message": "No cards due for review",
                    "stats": self._get_stats_dict(),
                })
                self.state = SessionState.ENDED
                return

            await self.websocket.send_json({
                "type": "session_started",
                "total_cards": len(cards),
                "deck_id": deck_id,
            })

            # Present first card
            await self._present_next_card()

        except Exception as e:
            logger.error(f"Failed to start session: {e}")
            self.state = SessionState.ERROR
            await self._send_error(f"Failed to start session: {e}")

    async def _present_next_card(self) -> None:
        """Get and present the next due card."""
        self.state = SessionState.PRESENTING_CARD

        try:
            # Get next due card
            cards = await self._api_client.get_due_cards(self._deck_id, limit=1)

            if not cards:
                await self._complete_session()
                return

            self._current_card = cards[0]
            card_front = self._current_card.get("front", "")

            # Generate TTS for card front
            tts_result = await self.pipeline.synthesize(card_front)

            # Send card info and audio
            await self.websocket.send_json({
                "type": "card_presented",
                "card_id": self._current_card.get("id"),
                "front": card_front,
                "audio": base64.b64encode(tts_result.audio).decode("utf-8"),
                "audio_duration": tts_result.duration,
                "sample_rate": tts_result.sample_rate,
            })

            # Ready to listen for answer
            self.state = SessionState.LISTENING
            self._audio_buffer = b""

            await self._send_state_update("listening")

        except Exception as e:
            logger.error(f"Failed to present card: {e}")
            await self._send_error(f"Failed to present card: {e}")

    async def _handle_audio_chunk(self, audio_bytes: bytes) -> None:
        """Accumulate audio data from client."""
        if self.state != SessionState.LISTENING:
            return

        self._audio_buffer += audio_bytes

        # Optional: Send VAD feedback to client
        if len(self._audio_buffer) % 8000 == 0:  # Every ~250ms at 16kHz
            samples = np.frombuffer(self._audio_buffer[-8000:], dtype=np.int16)
            samples_float = samples.astype(np.float32) / 32768.0
            speech_prob = self.pipeline._get_speech_probability(samples_float)

            await self.websocket.send_json({
                "type": "vad_status",
                "speech_probability": speech_prob,
                "is_speech": speech_prob > self.pipeline.vad_threshold,
            })

    async def _process_audio(self) -> None:
        """Process accumulated audio and evaluate answer."""
        if self.state != SessionState.LISTENING:
            return

        if not self._audio_buffer:
            await self._send_error("No audio received")
            return

        self.state = SessionState.PROCESSING
        await self._send_state_update("processing")

        try:
            # Convert audio to numpy array
            samples = np.frombuffer(self._audio_buffer, dtype=np.int16)
            audio_float = samples.astype(np.float32) / 32768.0

            # Update stats
            duration = len(audio_float) / 16000
            self.stats.total_audio_duration += duration

            # Transcribe
            result = await self.pipeline.transcribe(audio_float)

            await self.websocket.send_json({
                "type": "transcription",
                "text": result.text,
                "confidence": result.confidence,
                "duration": result.duration,
            })

            # Evaluate the answer
            await self._evaluate_answer(result.text)

        except Exception as e:
            logger.error(f"Audio processing failed: {e}")
            await self._send_error(f"Processing failed: {e}")
            self.state = SessionState.LISTENING

    async def _evaluate_answer(self, transcribed_text: str) -> None:
        """Evaluate the transcribed answer against the card."""
        self.state = SessionState.EVALUATING
        await self._send_state_update("evaluating")

        if not self._current_card:
            await self._send_error("No current card")
            return

        try:
            card_front = self._current_card.get("front", "")
            card_back = self._current_card.get("back", "")

            # Build evaluation prompt
            prompt = build_evaluation_prompt(
                question=card_front,
                expected_answer=card_back,
                user_answer=transcribed_text,
            )

            # Get LLM evaluation from API
            evaluation = await self._api_client.evaluate_answer(
                card_id=self._current_card.get("id"),
                user_answer=transcribed_text,
                prompt=prompt,
            )

            self.state = SessionState.PRESENTING_FEEDBACK
            self.stats.cards_reviewed += 1

            # Determine if correct based on rating
            is_correct = evaluation.get("rating", 0) >= 2
            if is_correct:
                self.stats.correct_count += 1
            else:
                self.stats.incorrect_count += 1

            # Generate TTS for feedback
            feedback_text = evaluation.get("feedback", "")
            if feedback_text:
                tts_result = await self.pipeline.synthesize(feedback_text)
                feedback_audio = base64.b64encode(tts_result.audio).decode("utf-8")
            else:
                feedback_audio = None

            await self.websocket.send_json({
                "type": "evaluation",
                "rating": evaluation.get("rating"),
                "is_correct": is_correct,
                "feedback": feedback_text,
                "expected_answer": card_back,
                "user_answer": transcribed_text,
                "audio": feedback_audio,
                "stats": self._get_stats_dict(),
            })

        except Exception as e:
            logger.error(f"Evaluation failed: {e}")
            await self._send_error(f"Evaluation failed: {e}")

    async def _rate_card(self, rating: int) -> None:
        """Submit rating for current card and update spaced repetition."""
        if not self._current_card:
            return

        try:
            card_id = self._current_card.get("id")
            await self._api_client.submit_review(card_id, rating)

            await self.websocket.send_json({
                "type": "card_rated",
                "card_id": card_id,
                "rating": rating,
            })

            # Present next card
            await self._present_next_card()

        except Exception as e:
            logger.error(f"Failed to rate card: {e}")
            await self._send_error(f"Rating failed: {e}")

    async def _skip_card(self) -> None:
        """Skip the current card without rating."""
        self._current_card = None
        await self._present_next_card()

    async def _replay_card(self) -> None:
        """Replay the current card's TTS audio."""
        if not self._current_card:
            return

        card_front = self._current_card.get("front", "")
        tts_result = await self.pipeline.synthesize(card_front)

        await self.websocket.send_json({
            "type": "card_replay",
            "card_id": self._current_card.get("id"),
            "audio": base64.b64encode(tts_result.audio).decode("utf-8"),
            "sample_rate": tts_result.sample_rate,
        })

    async def _complete_session(self) -> None:
        """Complete the session when all cards are reviewed."""
        self.state = SessionState.ENDED

        await self.websocket.send_json({
            "type": "session_complete",
            "message": "All cards reviewed!",
            "stats": self._get_stats_dict(),
        })

    async def _end_session(self) -> None:
        """End the session early (user requested)."""
        self.state = SessionState.ENDING
        await self._send_state_update("ending")

        await self.websocket.send_json({
            "type": "session_ended",
            "stats": self._get_stats_dict(),
        })

        self.state = SessionState.ENDED

    async def _send_state_update(self, state: str) -> None:
        """Send state update to client."""
        await self.websocket.send_json({
            "type": "state_change",
            "state": state,
        })

    async def _send_error(self, message: str) -> None:
        """Send error message to client."""
        await self.websocket.send_json({
            "type": "error",
            "message": message,
        })

    def _get_stats_dict(self) -> dict:
        """Get session stats as dictionary."""
        return {
            "cards_reviewed": self.stats.cards_reviewed,
            "correct_count": self.stats.correct_count,
            "incorrect_count": self.stats.incorrect_count,
            "accuracy": (
                self.stats.correct_count / self.stats.cards_reviewed
                if self.stats.cards_reviewed > 0
                else 0.0
            ),
            "total_audio_duration": self.stats.total_audio_duration,
        }

    async def cleanup(self) -> None:
        """Cleanup session resources."""
        self._audio_buffer = b""
        self._current_card = None
        self.state = SessionState.ENDED
