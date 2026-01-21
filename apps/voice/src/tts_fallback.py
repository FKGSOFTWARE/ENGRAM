"""
TTS implementation using Gemini 2.5 Flash TTS.

Uses Google's native TTS model for high-quality speech synthesis.
"""

import asyncio
import io
import logging
import os
from pathlib import Path
from typing import Optional

import numpy as np
from dotenv import load_dotenv

from .pipeline import TTSResult

logger = logging.getLogger(__name__)

# Load .env from project root (ENGRAM/.env)
_project_root = Path(__file__).parent.parent.parent.parent
_env_file = _project_root / ".env"
if _env_file.exists():
    load_dotenv(_env_file)
    logger.debug(f"Loaded environment from {_env_file}")


class GeminiTTS:
    """
    TTS engine using Gemini 2.5 Flash TTS model.

    Provides high-quality text-to-speech using Google's native
    audio generation capabilities.
    """

    def __init__(self, sample_rate: int = 24000):
        self.sample_rate = sample_rate
        self._client = None
        self._is_ready = False
        self._voice = "Kore"  # Default voice

    @property
    def is_ready(self) -> bool:
        return self._is_ready

    async def initialize(self) -> None:
        """Initialize the Gemini TTS client."""
        logger.info("Initializing Gemini TTS...")

        try:
            from google import genai

            api_key = os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY")
            if not api_key:
                raise ValueError("GEMINI_API_KEY or GOOGLE_API_KEY environment variable required")

            self._client = genai.Client(api_key=api_key)
            self._is_ready = True
            logger.info("Gemini TTS initialized successfully")

        except ImportError:
            logger.error("google-genai package not installed. Run: pip install google-genai")
            raise
        except Exception as e:
            logger.error(f"Failed to initialize Gemini TTS: {e}")
            raise

    async def shutdown(self) -> None:
        """Release resources."""
        self._is_ready = False
        self._client = None

    async def synthesize(
        self,
        text: str,
        voice: Optional[str] = None,
    ) -> TTSResult:
        """
        Synthesize text to speech using Gemini TTS.

        Args:
            text: Text to synthesize
            voice: Optional voice name (Kore, Puck, Charon, etc.)

        Returns:
            TTSResult with PCM audio bytes and metadata
        """
        if not self._is_ready or self._client is None:
            raise RuntimeError("Gemini TTS not initialized")

        from google.genai import types

        voice_name = voice or self._voice

        try:
            # Run in executor since the SDK may be blocking
            loop = asyncio.get_event_loop()
            response = await loop.run_in_executor(
                None,
                lambda: self._client.models.generate_content(
                    model="gemini-2.5-flash-preview-tts",
                    contents=text,
                    config=types.GenerateContentConfig(
                        response_modalities=["AUDIO"],
                        speech_config=types.SpeechConfig(
                            voice_config=types.VoiceConfig(
                                prebuilt_voice_config=types.PrebuiltVoiceConfig(
                                    voice_name=voice_name
                                )
                            )
                        )
                    )
                )
            )

            # Extract audio data from response
            audio_data = None
            if response.candidates:
                for part in response.candidates[0].content.parts:
                    if hasattr(part, 'inline_data') and part.inline_data:
                        audio_data = part.inline_data.data
                        break

            if audio_data is None:
                logger.warning("No audio data in Gemini TTS response, returning silent audio")
                return self._generate_silent_audio(text)

            # Gemini returns PCM 24kHz 16-bit mono
            duration = len(audio_data) / (self.sample_rate * 2)  # 2 bytes per sample

            return TTSResult(
                audio=audio_data,
                sample_rate=self.sample_rate,
                duration=duration,
            )

        except Exception as e:
            logger.warning(f"Gemini TTS synthesis failed: {e}, returning silent audio")
            # Return silence on error rather than crashing
            return self._generate_silent_audio(text)

    def _generate_silent_audio(self, text: str) -> TTSResult:
        """Generate silent audio as fallback."""
        words = len(text.split())
        duration = max(1.0, words * 0.4)
        n_samples = int(duration * self.sample_rate)
        audio = np.zeros(n_samples, dtype=np.int16)

        return TTSResult(
            audio=audio.tobytes(),
            sample_rate=self.sample_rate,
            duration=duration,
            is_silent=True,
        )

    def set_voice(self, voice: str) -> None:
        """Set the default voice for synthesis."""
        self._voice = voice


# Alias for backward compatibility
FallbackTTS = GeminiTTS


# =============================================================================
# NON-MVP: Alternative TTS implementations
# =============================================================================
# The following code provides fallback TTS options using edge-tts and pyttsx3.
# These are commented out for MVP but can be re-enabled if Gemini TTS is
# unavailable or for offline/cost-sensitive deployments.
# =============================================================================

# class EdgeTTSFallback:
#     """
#     NON-MVP: Fallback TTS using Microsoft Edge TTS.
#
#     Requires: pip install edge-tts pydub
#     Also requires: ffmpeg installed on system
#     """
#
#     def __init__(self, sample_rate: int = 24000):
#         self.sample_rate = sample_rate
#         self._is_ready = False
#
#     async def initialize(self) -> None:
#         """Initialize edge-tts."""
#         try:
#             import edge_tts
#             self._is_ready = True
#             logger.info("Edge TTS initialized")
#         except ImportError:
#             logger.error("edge-tts not available")
#             raise
#
#     async def synthesize(self, text: str, voice: Optional[str] = None) -> TTSResult:
#         """Synthesize using edge-tts."""
#         import edge_tts
#         from pydub import AudioSegment
#
#         voice = voice or "en-US-GuyNeural"
#         communicate = edge_tts.Communicate(text, voice)
#
#         audio_chunks = []
#         async for chunk in communicate.stream():
#             if chunk["type"] == "audio":
#                 audio_chunks.append(chunk["data"])
#
#         mp3_bytes = b"".join(audio_chunks)
#
#         # Convert MP3 to PCM using pydub
#         audio_segment = AudioSegment.from_mp3(io.BytesIO(mp3_bytes))
#         audio_segment = audio_segment.set_channels(1)
#         audio_segment = audio_segment.set_frame_rate(self.sample_rate)
#         audio_segment = audio_segment.set_sample_width(2)
#
#         audio_bytes = audio_segment.raw_data
#         duration = len(audio_segment) / 1000.0
#
#         return TTSResult(
#             audio=audio_bytes,
#             sample_rate=self.sample_rate,
#             duration=duration,
#         )


# class Pyttsx3Fallback:
#     """
#     NON-MVP: Fallback TTS using pyttsx3 (offline, lower quality).
#
#     Requires: pip install pyttsx3
#     """
#
#     def __init__(self, sample_rate: int = 24000):
#         self.sample_rate = sample_rate
#         self._engine = None
#         self._is_ready = False
#
#     async def initialize(self) -> None:
#         """Initialize pyttsx3."""
#         import pyttsx3
#
#         def init_engine():
#             engine = pyttsx3.init()
#             engine.setProperty("rate", 150)
#             engine.setProperty("volume", 0.9)
#             return engine
#
#         loop = asyncio.get_event_loop()
#         self._engine = await loop.run_in_executor(None, init_engine)
#         self._is_ready = True
#         logger.info("pyttsx3 TTS initialized")
#
#     async def synthesize(self, text: str, voice: Optional[str] = None) -> TTSResult:
#         """Synthesize using pyttsx3."""
#         import tempfile
#         import wave
#
#         def do_synthesize():
#             with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as f:
#                 temp_path = f.name
#
#             self._engine.save_to_file(text, temp_path)
#             self._engine.runAndWait()
#
#             with wave.open(temp_path, "rb") as wf:
#                 framerate = wf.getframerate()
#                 n_frames = wf.getnframes()
#                 audio_bytes = wf.readframes(n_frames)
#                 duration = n_frames / framerate
#
#             import os
#             os.unlink(temp_path)
#
#             # Resample if needed (simplified)
#             if framerate != self.sample_rate:
#                 samples = np.frombuffer(audio_bytes, dtype=np.int16)
#                 ratio = self.sample_rate / framerate
#                 new_length = int(len(samples) * ratio)
#                 indices = np.linspace(0, len(samples) - 1, new_length).astype(int)
#                 resampled = samples[indices]
#                 audio_bytes = resampled.tobytes()
#
#             return TTSResult(
#                 audio=audio_bytes,
#                 sample_rate=self.sample_rate,
#                 duration=duration,
#             )
#
#         loop = asyncio.get_event_loop()
#         return await loop.run_in_executor(None, do_synthesize)
