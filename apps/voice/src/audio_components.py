"""
Modular audio components for ENGRAM voice sessions.

This module provides a foundation for audio segment management,
supporting both live TTS generation and future pre-generated audio.

Phase 1 (Current): All audio generated live via LLM + TTS
Phase 2 (Future):  Questions pre-generated at card creation
Phase 3 (Future):  Transitions pre-generated as audio bank
"""

from dataclasses import dataclass
from enum import Enum
from typing import Optional


class SegmentType(Enum):
    """Types of audio segments in a review session."""

    QUESTION = "question"  # Card front spoken aloud
    TRANSITION = "transition"  # "Moving on...", "Next up...", etc
    FEEDBACK = "feedback"  # Response to user's answer
    SESSION_INTRO = "session_intro"  # Warm greeting at session start
    SESSION_OUTRO = "session_outro"  # Encouraging conclusion


class AudioSource(Enum):
    """Source of the audio data."""

    LIVE_TTS = "live_tts"  # Generated in real-time
    PREGENERATED = "pregenerated"  # Pre-generated at card creation
    CACHED = "cached"  # Retrieved from audio cache


@dataclass
class AudioSegment:
    """
    Represents a segment of audio for playback.

    Attributes:
        audio: Raw audio bytes
        sample_rate: Audio sample rate (e.g., 16000, 24000)
        duration: Duration in seconds
        segment_type: Type of audio segment
        source: Where the audio came from
        text: Original text that was synthesized (if applicable)
        card_id: Associated card ID (if applicable)
    """

    audio: bytes
    sample_rate: int
    duration: float
    segment_type: SegmentType
    source: AudioSource
    text: Optional[str] = None
    card_id: Optional[str] = None


class TransitionType(Enum):
    """Types of transitions between cards."""

    NEXT = "next"  # Standard next card
    CORRECT = "correct"  # After correct answer
    INCORRECT = "incorrect"  # After incorrect answer
    SKIP = "skip"  # User skipped card
    FINAL = "final"  # Last card transition


# Transition phrases for natural flow
TRANSITION_PHRASES = {
    TransitionType.NEXT: [
        "Moving on.",
        "Next up.",
        "Here's another one.",
        "Let's continue.",
    ],
    TransitionType.CORRECT: [
        "Nice! Next one.",
        "Well done. Moving on.",
        "That's right. Here's the next.",
    ],
    TransitionType.INCORRECT: [
        "Let's try another.",
        "Moving on to the next.",
        "Here's the next one.",
    ],
    TransitionType.SKIP: [
        "Skipping ahead.",
        "Moving on.",
        "Next card.",
    ],
    TransitionType.FINAL: [
        "Last one.",
        "Final card.",
        "Here's your last one.",
    ],
}


class AudioAssembler:
    """
    Assembles audio segments for playback.

    This class provides methods to retrieve audio segments,
    abstracting away whether they're generated live or pre-stored.
    """

    def __init__(self, pipeline):
        """
        Initialize the audio assembler.

        Args:
            pipeline: VoicePipeline instance for TTS generation
        """
        self.pipeline = pipeline
        self._transition_index = 0

    async def get_question_audio(
        self,
        card_id: str,
        text: str,
    ) -> AudioSegment:
        """
        Get audio for a card question.

        Currently generates live; will support pre-generated in future.

        Args:
            card_id: The card's unique ID
            text: The question text to speak

        Returns:
            AudioSegment with the question audio
        """
        tts_result = await self.pipeline.synthesize(text)

        return AudioSegment(
            audio=tts_result.audio,
            sample_rate=tts_result.sample_rate,
            duration=tts_result.duration,
            segment_type=SegmentType.QUESTION,
            source=AudioSource.LIVE_TTS,
            text=text,
            card_id=card_id,
        )

    async def get_transition_audio(
        self,
        transition_type: TransitionType,
    ) -> AudioSegment:
        """
        Get audio for a transition between cards.

        Args:
            transition_type: Type of transition

        Returns:
            AudioSegment with the transition audio
        """
        import random

        phrases = TRANSITION_PHRASES.get(transition_type, TRANSITION_PHRASES[TransitionType.NEXT])
        text = random.choice(phrases)

        tts_result = await self.pipeline.synthesize(text)

        return AudioSegment(
            audio=tts_result.audio,
            sample_rate=tts_result.sample_rate,
            duration=tts_result.duration,
            segment_type=SegmentType.TRANSITION,
            source=AudioSource.LIVE_TTS,
            text=text,
        )

    async def get_feedback_audio(
        self,
        text: str,
    ) -> AudioSegment:
        """
        Get audio for feedback on user's answer.

        Args:
            text: The feedback text to speak

        Returns:
            AudioSegment with the feedback audio
        """
        tts_result = await self.pipeline.synthesize(text)

        return AudioSegment(
            audio=tts_result.audio,
            sample_rate=tts_result.sample_rate,
            duration=tts_result.duration,
            segment_type=SegmentType.FEEDBACK,
            source=AudioSource.LIVE_TTS,
            text=text,
        )

    async def get_session_intro_audio(
        self,
        text: str,
    ) -> AudioSegment:
        """
        Get audio for session introduction.

        Args:
            text: The intro text to speak

        Returns:
            AudioSegment with the intro audio
        """
        tts_result = await self.pipeline.synthesize(text)

        return AudioSegment(
            audio=tts_result.audio,
            sample_rate=tts_result.sample_rate,
            duration=tts_result.duration,
            segment_type=SegmentType.SESSION_INTRO,
            source=AudioSource.LIVE_TTS,
            text=text,
        )

    async def get_session_outro_audio(
        self,
        text: str,
    ) -> AudioSegment:
        """
        Get audio for session conclusion.

        Args:
            text: The outro text to speak

        Returns:
            AudioSegment with the outro audio
        """
        tts_result = await self.pipeline.synthesize(text)

        return AudioSegment(
            audio=tts_result.audio,
            sample_rate=tts_result.sample_rate,
            duration=tts_result.duration,
            segment_type=SegmentType.SESSION_OUTRO,
            source=AudioSource.LIVE_TTS,
            text=text,
        )
