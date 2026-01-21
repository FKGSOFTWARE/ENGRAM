"""
Voice processing pipeline with Silero VAD and faster-whisper STT.

This module implements the core audio processing:
- Voice Activity Detection (VAD) using Silero
- Speech-to-Text (STT) using faster-whisper
- Text-to-Speech (TTS) using Gemini 2.5 Flash TTS
"""

import asyncio
import io
import logging
from dataclasses import dataclass
from typing import AsyncIterator, Optional

import numpy as np
import torch

logger = logging.getLogger(__name__)


@dataclass
class TranscriptionResult:
    """Result from speech-to-text transcription."""

    text: str
    language: str
    confidence: float
    duration: float


@dataclass
class TTSResult:
    """Result from text-to-speech synthesis."""

    audio: bytes
    sample_rate: int
    duration: float
    is_silent: bool = False  # True if fallback silent audio was generated


class VoicePipeline:
    """
    Voice processing pipeline integrating VAD, STT, and TTS.

    Components:
    - Silero VAD: Detects speech segments in audio
    - faster-whisper: Transcribes speech to text
    - Gemini TTS: Synthesizes text to speech (Google's native audio model)
    """

    def __init__(
        self,
        whisper_model: str = "base.en",
        whisper_device: str = "cpu",
        vad_threshold: float = 0.5,
    ):
        self.whisper_model_name = whisper_model
        self.whisper_device = whisper_device
        self.vad_threshold = vad_threshold

        self._vad_model: Optional[torch.jit.ScriptModule] = None
        self._whisper_model: Optional["WhisperModel"] = None
        self._tts_engine: Optional["TTSEngine"] = None
        self._is_ready = False

    @property
    def is_ready(self) -> bool:
        """Check if pipeline is initialized and ready."""
        return self._is_ready

    async def initialize(self) -> None:
        """Initialize all pipeline components."""
        await asyncio.gather(
            self._init_vad(),
            self._init_stt(),
            self._init_tts(),
        )
        self._is_ready = True
        logger.info("Voice pipeline fully initialized")

    async def _init_vad(self) -> None:
        """Initialize Silero VAD model."""
        logger.info("Loading Silero VAD model...")

        def load_vad():
            model, utils = torch.hub.load(
                repo_or_dir="snakers4/silero-vad",
                model="silero_vad",
                force_reload=False,
                trust_repo=True,
            )
            return model

        loop = asyncio.get_event_loop()
        self._vad_model = await loop.run_in_executor(None, load_vad)
        logger.info("Silero VAD model loaded")

    async def _init_stt(self) -> None:
        """Initialize faster-whisper STT model."""
        logger.info(f"Loading faster-whisper model: {self.whisper_model_name}")

        def load_whisper():
            from faster_whisper import WhisperModel

            return WhisperModel(
                self.whisper_model_name,
                device=self.whisper_device,
                compute_type="int8" if self.whisper_device == "cpu" else "float16",
            )

        loop = asyncio.get_event_loop()
        self._whisper_model = await loop.run_in_executor(None, load_whisper)
        logger.info("faster-whisper model loaded")

    async def _init_tts(self) -> None:
        """Initialize TTS engine (Gemini TTS)."""
        logger.info("Initializing TTS engine...")

        from .tts_fallback import GeminiTTS

        self._tts_engine = GeminiTTS()
        await self._tts_engine.initialize()
        logger.info("Gemini TTS initialized")

    async def shutdown(self) -> None:
        """Cleanup pipeline resources."""
        self._is_ready = False

        if self._tts_engine:
            await self._tts_engine.shutdown()

        # Clear model references
        self._vad_model = None
        self._whisper_model = None
        self._tts_engine = None

        # Force garbage collection
        import gc

        gc.collect()
        if torch.cuda.is_available():
            torch.cuda.empty_cache()

    def detect_voice_activity(
        self,
        audio: np.ndarray,
        sample_rate: int = 16000,
    ) -> list[tuple[float, float]]:
        """
        Detect voice activity segments in audio.

        Args:
            audio: Audio samples as numpy array (float32, normalized)
            sample_rate: Audio sample rate (default 16000)

        Returns:
            List of (start_time, end_time) tuples for speech segments
        """
        if self._vad_model is None:
            raise RuntimeError("VAD model not initialized")

        # Convert to torch tensor
        audio_tensor = torch.from_numpy(audio).float()

        # Get speech timestamps
        speech_timestamps = self._vad_model(
            audio_tensor,
            sample_rate,
            threshold=self.vad_threshold,
            min_speech_duration_ms=250,
            min_silence_duration_ms=500,
        )

        # Convert to time segments
        segments = []
        for segment in speech_timestamps:
            start = segment["start"] / sample_rate
            end = segment["end"] / sample_rate
            segments.append((start, end))

        return segments

    async def transcribe(
        self,
        audio: np.ndarray,
        sample_rate: int = 16000,
        language: Optional[str] = None,
    ) -> TranscriptionResult:
        """
        Transcribe audio to text using faster-whisper.

        Args:
            audio: Audio samples as numpy array (float32, normalized)
            sample_rate: Audio sample rate
            language: Optional language code (e.g., "en")

        Returns:
            TranscriptionResult with text and metadata
        """
        if self._whisper_model is None:
            raise RuntimeError("Whisper model not initialized")

        def do_transcribe():
            segments, info = self._whisper_model.transcribe(
                audio,
                language=language,
                beam_size=5,
                vad_filter=True,
                vad_parameters=dict(
                    threshold=self.vad_threshold,
                    min_speech_duration_ms=250,
                    min_silence_duration_ms=500,
                ),
            )

            # Collect all segment text
            text_parts = []
            total_confidence = 0.0
            segment_count = 0

            for segment in segments:
                text_parts.append(segment.text.strip())
                total_confidence += segment.avg_logprob
                segment_count += 1

            text = " ".join(text_parts)
            avg_confidence = (
                np.exp(total_confidence / segment_count) if segment_count > 0 else 0.0
            )

            return TranscriptionResult(
                text=text,
                language=info.language,
                confidence=float(avg_confidence),
                duration=info.duration,
            )

        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, do_transcribe)

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
        if self._tts_engine is None:
            raise RuntimeError("TTS engine not initialized")

        return await self._tts_engine.synthesize(text, voice)

    async def process_audio_stream(
        self,
        audio_chunks: AsyncIterator[bytes],
        sample_rate: int = 16000,
    ) -> AsyncIterator[TranscriptionResult]:
        """
        Process streaming audio with VAD and transcription.

        Yields transcription results as speech segments are detected
        and processed.

        Args:
            audio_chunks: Async iterator of audio chunk bytes (PCM int16)
            sample_rate: Audio sample rate

        Yields:
            TranscriptionResult for each detected speech segment
        """
        audio_buffer = np.array([], dtype=np.float32)
        speech_buffer = np.array([], dtype=np.float32)
        is_speaking = False
        silence_samples = 0
        silence_threshold = int(sample_rate * 0.5)  # 500ms silence

        async for chunk in audio_chunks:
            # Convert PCM int16 to float32
            samples = np.frombuffer(chunk, dtype=np.int16).astype(np.float32) / 32768.0
            audio_buffer = np.concatenate([audio_buffer, samples])

            # Check VAD every 100ms of audio
            if len(audio_buffer) >= sample_rate // 10:
                speech_probs = self._get_speech_probability(audio_buffer[-sample_rate // 10 :])

                if speech_probs > self.vad_threshold:
                    if not is_speaking:
                        is_speaking = True
                        silence_samples = 0
                    speech_buffer = np.concatenate([speech_buffer, audio_buffer])
                    audio_buffer = np.array([], dtype=np.float32)
                else:
                    if is_speaking:
                        silence_samples += len(audio_buffer)
                        speech_buffer = np.concatenate([speech_buffer, audio_buffer])
                        audio_buffer = np.array([], dtype=np.float32)

                        if silence_samples >= silence_threshold:
                            # End of speech detected, transcribe
                            if len(speech_buffer) > sample_rate // 4:  # >250ms
                                result = await self.transcribe(speech_buffer, sample_rate)
                                if result.text.strip():
                                    yield result

                            speech_buffer = np.array([], dtype=np.float32)
                            is_speaking = False
                            silence_samples = 0

        # Process any remaining audio
        if len(speech_buffer) > sample_rate // 4:
            result = await self.transcribe(speech_buffer, sample_rate)
            if result.text.strip():
                yield result

    def _get_speech_probability(self, audio: np.ndarray) -> float:
        """Get speech probability for audio chunk using VAD."""
        if self._vad_model is None:
            return 0.0

        audio_tensor = torch.from_numpy(audio).float()

        # Silero VAD expects 512 samples at 16kHz
        if len(audio_tensor) < 512:
            audio_tensor = torch.nn.functional.pad(audio_tensor, (0, 512 - len(audio_tensor)))

        with torch.no_grad():
            speech_prob = self._vad_model(audio_tensor[:512], 16000).item()

        return speech_prob
