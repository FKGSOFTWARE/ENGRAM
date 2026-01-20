"""
Chatterbox TTS integration for high-quality voice synthesis.

Chatterbox is an expressive TTS model that produces natural-sounding speech.
This module provides the async wrapper for use in the voice pipeline.
"""

import asyncio
import io
import logging
from typing import Optional

import numpy as np

from .pipeline import TTSResult

logger = logging.getLogger(__name__)


class ChatterboxTTS:
    """
    Chatterbox TTS engine wrapper.

    Provides high-quality, expressive text-to-speech synthesis
    using the Chatterbox model.
    """

    def __init__(
        self,
        model_name: str = "chatterbox",
        device: str = "cpu",
        sample_rate: int = 24000,
    ):
        self.model_name = model_name
        self.device = device
        self.sample_rate = sample_rate
        self._model = None
        self._is_ready = False

    @property
    def is_ready(self) -> bool:
        return self._is_ready

    async def initialize(self) -> None:
        """Initialize the Chatterbox model."""
        logger.info("Loading Chatterbox TTS model...")

        def load_model():
            try:
                # Try to import chatterbox
                from chatterbox import ChatterboxModel

                model = ChatterboxModel.from_pretrained(self.model_name)
                model.to(self.device)
                return model
            except ImportError:
                raise ImportError(
                    "Chatterbox TTS not installed. Install with: pip install chatterbox-tts"
                )

        loop = asyncio.get_event_loop()
        self._model = await loop.run_in_executor(None, load_model)
        self._is_ready = True
        logger.info("Chatterbox TTS model loaded")

    async def shutdown(self) -> None:
        """Release model resources."""
        self._is_ready = False
        self._model = None

        import gc
        import torch

        gc.collect()
        if torch.cuda.is_available():
            torch.cuda.empty_cache()

    async def synthesize(
        self,
        text: str,
        voice: Optional[str] = None,
        speed: float = 1.0,
        emotion: Optional[str] = None,
    ) -> TTSResult:
        """
        Synthesize text to speech using Chatterbox.

        Args:
            text: Text to synthesize
            voice: Optional voice preset
            speed: Speech rate multiplier (1.0 = normal)
            emotion: Optional emotion preset

        Returns:
            TTSResult with audio bytes and metadata
        """
        if not self._is_ready or self._model is None:
            raise RuntimeError("Chatterbox model not initialized")

        def do_synthesize():
            # Generate audio
            audio = self._model.synthesize(
                text,
                voice=voice,
                speed=speed,
            )

            # Convert to numpy array if needed
            if hasattr(audio, "numpy"):
                audio_np = audio.numpy()
            else:
                audio_np = np.array(audio)

            # Ensure float32 and normalize
            audio_np = audio_np.astype(np.float32)
            if audio_np.max() > 1.0:
                audio_np = audio_np / 32768.0

            # Convert to PCM bytes (int16)
            audio_int16 = (audio_np * 32767).astype(np.int16)
            audio_bytes = audio_int16.tobytes()

            duration = len(audio_np) / self.sample_rate

            return TTSResult(
                audio=audio_bytes,
                sample_rate=self.sample_rate,
                duration=duration,
            )

        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, do_synthesize)

    async def synthesize_streaming(
        self,
        text: str,
        voice: Optional[str] = None,
    ):
        """
        Synthesize text to speech with streaming output.

        Yields audio chunks as they are generated.

        Args:
            text: Text to synthesize
            voice: Optional voice preset

        Yields:
            Audio chunk bytes (PCM int16)
        """
        if not self._is_ready or self._model is None:
            raise RuntimeError("Chatterbox model not initialized")

        # For now, generate full audio and yield in chunks
        # Chatterbox may support native streaming in future versions
        result = await self.synthesize(text, voice)

        chunk_size = self.sample_rate // 10  # 100ms chunks
        chunk_bytes = chunk_size * 2  # 2 bytes per int16 sample

        audio = result.audio
        for i in range(0, len(audio), chunk_bytes):
            yield audio[i : i + chunk_bytes]
