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
from .prompts import (
    build_evaluation_prompt,
    build_oral_feedback_prompt,
    build_conversational_question_prompt,
    build_conversational_evaluation_prompt,
    build_session_intro_prompt,
    build_session_outro_prompt,
)

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


class ReviewMode(Enum):
    """Review modes for flashcard sessions."""

    MANUAL = "manual"  # Text-based, click to reveal, click to rate
    ORAL = "oral"  # Voice answer, LLM evaluates, structured feedback, auto-rates
    CONVERSATIONAL = "conversational"  # Natural tutoring, Feynman-style, auto-rates


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
    _review_mode: ReviewMode = field(default=ReviewMode.MANUAL, init=False)
    _total_cards: int = field(default=0, init=False)
    _current_card_number: int = field(default=0, init=False)

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
            review_mode = message.get("review_mode", "manual")
            await self._start_session(message.get("deck_id"), review_mode)

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

    async def _start_session(self, deck_id: Optional[str], review_mode: str = "manual") -> None:
        """Start a new review session."""
        if self.state != SessionState.IDLE:
            await self._send_error("Session already in progress")
            return

        self.state = SessionState.STARTING
        self._deck_id = deck_id

        # Parse review mode
        try:
            self._review_mode = ReviewMode(review_mode)
        except ValueError:
            self._review_mode = ReviewMode.MANUAL

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

            self._total_cards = len(cards)
            self._current_card_number = 0

            await self.websocket.send_json({
                "type": "session_started",
                "total_cards": len(cards),
                "deck_id": deck_id,
                "review_mode": self._review_mode.value,
            })

            # For conversational mode, generate and speak intro
            if self._review_mode == ReviewMode.CONVERSATIONAL:
                await self._conversational_intro()

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
            self._current_card_number += 1
            card_front = self._current_card.get("front", "")

            # For conversational mode, add personality to question
            if self._review_mode == ReviewMode.CONVERSATIONAL:
                question_text = await self._get_conversational_question(card_front)
            else:
                question_text = card_front

            # Generate TTS for question
            tts_result = await self.pipeline.synthesize(question_text)

            # Send card info and audio
            await self.websocket.send_json({
                "type": "card_presented",
                "card_id": self._current_card.get("id"),
                "front": card_front,
                "spoken_text": question_text,
                "audio": base64.b64encode(tts_result.audio).decode("utf-8"),
                "audio_duration": tts_result.duration,
                "sample_rate": tts_result.sample_rate,
                "audio_is_silent": getattr(tts_result, 'is_silent', False),
                "card_number": self._current_card_number,
                "total_cards": self._total_cards,
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

            # Build evaluation prompt based on review mode
            if self._review_mode == ReviewMode.ORAL:
                prompt = build_oral_feedback_prompt(
                    question=card_front,
                    expected_answer=card_back,
                    user_answer=transcribed_text,
                )
            elif self._review_mode == ReviewMode.CONVERSATIONAL:
                prompt = build_conversational_evaluation_prompt(
                    question=card_front,
                    expected_answer=card_back,
                    user_answer=transcribed_text,
                )
            else:
                # Manual mode uses standard evaluation
                prompt = build_evaluation_prompt(
                    question=card_front,
                    expected_answer=card_back,
                    user_answer=transcribed_text,
                )

            # Get LLM evaluation from API
            response = await self._api_client.evaluate_answer(
                card_id=self._current_card.get("id"),
                user_answer=transcribed_text,
                prompt=prompt,
            )

            self.state = SessionState.PRESENTING_FEEDBACK
            self.stats.cards_reviewed += 1

            # Extract evaluation from nested response structure
            # Backend returns: {"evaluation": {...}, "error": ...}
            eval_data = response.get("evaluation", {}) or {}

            # Convert suggested_rating string to numeric rating
            rating_map = {"again": 0, "hard": 1, "good": 2, "easy": 3}
            suggested_rating = eval_data.get("suggested_rating", "good")
            if isinstance(suggested_rating, str):
                rating = rating_map.get(suggested_rating.lower(), 2)
            else:
                rating = suggested_rating if isinstance(suggested_rating, int) else 2

            # Determine if correct based on is_correct field or rating
            is_correct = eval_data.get("is_correct", rating >= 2)
            if is_correct:
                self.stats.correct_count += 1
            else:
                self.stats.incorrect_count += 1

            # Get feedback text from evaluation data
            feedback_text = eval_data.get("feedback", "")

            # Generate TTS for feedback
            feedback_audio = None
            feedback_sample_rate = 24000
            feedback_audio_duration = 0.0
            feedback_audio_is_silent = False
            if feedback_text:
                tts_result = await self.pipeline.synthesize(feedback_text)
                feedback_audio = base64.b64encode(tts_result.audio).decode("utf-8")
                feedback_sample_rate = tts_result.sample_rate
                feedback_audio_duration = tts_result.duration
                feedback_audio_is_silent = getattr(tts_result, 'is_silent', False)

            # Determine if auto-advance is needed
            auto_advance = self._review_mode in (ReviewMode.ORAL, ReviewMode.CONVERSATIONAL)

            await self.websocket.send_json({
                "type": "evaluation",
                "rating": rating,
                "is_correct": is_correct,
                "feedback": feedback_text,
                "expected_answer": card_back,
                "user_answer": transcribed_text,
                "audio": feedback_audio,
                "sample_rate": feedback_sample_rate,
                "audio_is_silent": feedback_audio_is_silent,
                "stats": self._get_stats_dict(),
                "auto_advance": auto_advance,
                "review_mode": self._review_mode.value,
                "feedback_audio_duration": feedback_audio_duration,
            })

            # For oral and conversational modes, auto-rate and advance
            if auto_advance:
                # Minimum display time for evaluation feedback (in seconds)
                # Ensures user has time to read feedback even without audio
                MIN_EVALUATION_DISPLAY_SECS = 3.0

                # Wait for audio to finish playing, or minimum display time
                wait_time = max(MIN_EVALUATION_DISPLAY_SECS, feedback_audio_duration)
                await asyncio.sleep(wait_time)

                await self._auto_rate_and_advance(rating)

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

        # For conversational mode, regenerate with personality
        if self._review_mode == ReviewMode.CONVERSATIONAL:
            replay_text = await self._get_conversational_question(card_front)
        else:
            replay_text = card_front

        tts_result = await self.pipeline.synthesize(replay_text)

        await self.websocket.send_json({
            "type": "card_replay",
            "card_id": self._current_card.get("id"),
            "audio": base64.b64encode(tts_result.audio).decode("utf-8"),
            "sample_rate": tts_result.sample_rate,
            "audio_is_silent": getattr(tts_result, 'is_silent', False),
        })

    async def _complete_session(self) -> None:
        """Complete the session when all cards are reviewed."""
        self.state = SessionState.ENDED

        # For conversational mode, generate encouraging outro
        outro_audio = None
        outro_text = None
        outro_sample_rate = 24000
        outro_audio_is_silent = False
        if self._review_mode == ReviewMode.CONVERSATIONAL:
            outro_text = await self._get_session_outro()
            if outro_text:
                tts_result = await self.pipeline.synthesize(outro_text)
                outro_audio = base64.b64encode(tts_result.audio).decode("utf-8")
                outro_sample_rate = tts_result.sample_rate
                outro_audio_is_silent = getattr(tts_result, 'is_silent', False)

        await self.websocket.send_json({
            "type": "session_complete",
            "message": outro_text or "All cards reviewed!",
            "stats": self._get_stats_dict(),
            "audio": outro_audio,
            "sample_rate": outro_sample_rate,
            "audio_is_silent": outro_audio_is_silent,
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

    # =========================================================================
    # CONVERSATIONAL MODE HELPERS
    # =========================================================================

    async def _conversational_intro(self) -> None:
        """Generate and speak a warm session greeting for conversational mode."""
        try:
            prompt = build_session_intro_prompt(self._total_cards)
            intro_text = await self._api_client.generate_text(prompt)

            if intro_text:
                tts_result = await self.pipeline.synthesize(intro_text)
                await self.websocket.send_json({
                    "type": "session_intro",
                    "text": intro_text,
                    "audio": base64.b64encode(tts_result.audio).decode("utf-8"),
                    "audio_duration": tts_result.duration,
                    "sample_rate": tts_result.sample_rate,
                    "audio_is_silent": getattr(tts_result, 'is_silent', False),
                })
        except Exception as e:
            logger.warning(f"Failed to generate session intro: {e}")
            # Continue without intro - not critical

    async def _get_conversational_question(self, card_front: str) -> str:
        """
        Generate a Feynman-style question presentation for conversational mode.

        Args:
            card_front: The raw question text

        Returns:
            The question with personality added
        """
        try:
            prompt = build_conversational_question_prompt(
                question=card_front,
                card_number=self._current_card_number,
                total_cards=self._total_cards,
            )
            question_text = await self._api_client.generate_text(prompt)
            return question_text if question_text else card_front
        except Exception as e:
            logger.warning(f"Failed to generate conversational question: {e}")
            return card_front  # Fallback to plain question

    async def _get_session_outro(self) -> str:
        """
        Generate an encouraging session conclusion for conversational mode.

        Returns:
            Outro text or empty string on failure
        """
        try:
            accuracy = (
                self.stats.correct_count / self.stats.cards_reviewed
                if self.stats.cards_reviewed > 0
                else 0.0
            )
            prompt = build_session_outro_prompt(
                correct_count=self.stats.correct_count,
                total_count=self.stats.cards_reviewed,
                accuracy=accuracy,
            )
            return await self._api_client.generate_text(prompt)
        except Exception as e:
            logger.warning(f"Failed to generate session outro: {e}")
            return ""

    async def _auto_rate_and_advance(self, rating: int) -> None:
        """
        Auto-rate the current card and advance to the next.

        Used by oral and conversational modes where the LLM determines the rating.

        Args:
            rating: The LLM-determined rating (0-3)
        """
        if not self._current_card:
            return

        try:
            card_id = self._current_card.get("id")
            await self._api_client.submit_review(card_id, rating)

            await self.websocket.send_json({
                "type": "card_rated",
                "card_id": card_id,
                "rating": rating,
                "auto_rated": True,
            })

            # Present next card
            await self._present_next_card()

        except Exception as e:
            logger.error(f"Failed to auto-rate card: {e}")
            await self._send_error(f"Auto-rating failed: {e}")

    async def cleanup(self) -> None:
        """Cleanup session resources."""
        self._audio_buffer = b""
        self._current_card = None
        self.state = SessionState.ENDED
        if self._api_client:
            await self._api_client.close()
