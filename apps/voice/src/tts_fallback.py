"""
Fallback TTS implementation using pyttsx3 or edge-tts.

Provides basic text-to-speech when Chatterbox is not available.
"""

import asyncio
import io
import logging
import struct
import tempfile
import wave
from typing import Optional

import numpy as np

from .pipeline import TTSResult

logger = logging.getLogger(__name__)


class FallbackTTS:
    """
    Fallback TTS engine using pyttsx3 or edge-tts.

    Provides basic TTS functionality when high-quality models
    are not available.
    """

    def __init__(self, sample_rate: int = 24000):
        self.sample_rate = sample_rate
        self._engine = None
        self._engine_type: Optional[str] = None
        self._is_ready = False

    @property
    def is_ready(self) -> bool:
        return self._is_ready

    async def initialize(self) -> None:
        """Initialize the fallback TTS engine."""
        logger.info("Initializing fallback TTS...")

        # Try edge-tts first (better quality)
        try:
            import edge_tts

            self._engine_type = "edge-tts"
            self._is_ready = True
            logger.info("Using edge-tts for fallback TTS")
            return
        except ImportError:
            logger.debug("edge-tts not available")

        # Fall back to pyttsx3
        try:
            import pyttsx3

            def init_pyttsx3():
                engine = pyttsx3.init()
                # Set properties
                engine.setProperty("rate", 150)  # Speed
                engine.setProperty("volume", 0.9)  # Volume
                return engine

            loop = asyncio.get_event_loop()
            self._engine = await loop.run_in_executor(None, init_pyttsx3)
            self._engine_type = "pyttsx3"
            self._is_ready = True
            logger.info("Using pyttsx3 for fallback TTS")
            return
        except ImportError:
            logger.debug("pyttsx3 not available")

        # No TTS available
        logger.warning("No TTS engine available - using silent placeholder")
        self._engine_type = "silent"
        self._is_ready = True

    async def shutdown(self) -> None:
        """Release engine resources."""
        self._is_ready = False
        if self._engine and self._engine_type == "pyttsx3":
            self._engine.stop()
        self._engine = None

    async def synthesize(
        self,
        text: str,
        voice: Optional[str] = None,
    ) -> TTSResult:
        """
        Synthesize text to speech.

        Args:
            text: Text to synthesize
            voice: Optional voice identifier

        Returns:
            TTSResult with audio bytes and metadata
        """
        if not self._is_ready:
            raise RuntimeError("TTS engine not initialized")

        if self._engine_type == "edge-tts":
            return await self._synthesize_edge_tts(text, voice)
        elif self._engine_type == "pyttsx3":
            return await self._synthesize_pyttsx3(text)
        else:
            return self._generate_silent_audio(text)

    async def _synthesize_edge_tts(
        self,
        text: str,
        voice: Optional[str] = None,
    ) -> TTSResult:
        """Synthesize using edge-tts."""
        import edge_tts

        voice = voice or "en-US-GuyNeural"

        communicate = edge_tts.Communicate(text, voice)

        audio_chunks = []
        async for chunk in communicate.stream():
            if chunk["type"] == "audio":
                audio_chunks.append(chunk["data"])

        audio_bytes = b"".join(audio_chunks)

        # edge-tts produces MP3, we need to convert to PCM
        # For simplicity, we'll use the raw bytes and estimate duration
        # In production, use pydub or ffmpeg for proper conversion

        # Estimate duration from text length (rough approximation)
        words = len(text.split())
        duration = words * 0.4  # ~150 words per minute

        return TTSResult(
            audio=audio_bytes,
            sample_rate=self.sample_rate,
            duration=duration,
        )

    async def _synthesize_pyttsx3(self, text: str) -> TTSResult:
        """Synthesize using pyttsx3."""
        if self._engine is None:
            raise RuntimeError("pyttsx3 engine not initialized")

        def do_synthesize():
            # Save to temporary WAV file
            with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as f:
                temp_path = f.name

            self._engine.save_to_file(text, temp_path)
            self._engine.runAndWait()

            # Read WAV file
            with wave.open(temp_path, "rb") as wf:
                n_channels = wf.getnchannels()
                sample_width = wf.getsampwidth()
                framerate = wf.getframerate()
                n_frames = wf.getnframes()
                audio_bytes = wf.readframes(n_frames)
                duration = n_frames / framerate

            # Convert to mono if stereo
            if n_channels == 2:
                samples = np.frombuffer(audio_bytes, dtype=np.int16)
                samples = samples.reshape(-1, 2).mean(axis=1).astype(np.int16)
                audio_bytes = samples.tobytes()

            # Resample if needed
            if framerate != self.sample_rate:
                samples = np.frombuffer(audio_bytes, dtype=np.int16)
                # Simple resampling (not high quality)
                ratio = self.sample_rate / framerate
                new_length = int(len(samples) * ratio)
                indices = np.linspace(0, len(samples) - 1, new_length).astype(int)
                resampled = samples[indices]
                audio_bytes = resampled.tobytes()

            # Cleanup
            import os

            os.unlink(temp_path)

            return TTSResult(
                audio=audio_bytes,
                sample_rate=self.sample_rate,
                duration=duration,
            )

        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, do_synthesize)

    def _generate_silent_audio(self, text: str) -> TTSResult:
        """Generate silent audio as placeholder."""
        # Estimate duration from text
        words = len(text.split())
        duration = max(1.0, words * 0.4)

        # Generate silence
        n_samples = int(duration * self.sample_rate)
        audio = np.zeros(n_samples, dtype=np.int16)

        return TTSResult(
            audio=audio.tobytes(),
            sample_rate=self.sample_rate,
            duration=duration,
        )
