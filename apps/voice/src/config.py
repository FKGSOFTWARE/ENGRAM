"""
Configuration settings for the ENGRAM Voice Service.
"""

from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """Voice service configuration loaded from environment variables."""

    # Server settings
    host: str = "0.0.0.0"
    port: int = 8001
    debug: bool = False

    # CORS settings
    cors_origins: list[str] = ["http://localhost:5173", "http://localhost:3000"]

    # Rust API settings
    api_base_url: str = "http://localhost:3000"

    # Whisper STT settings
    whisper_model: str = "base.en"  # Options: tiny, base, small, medium, large-v3
    whisper_device: str = "cpu"  # Options: cpu, cuda, auto
    whisper_compute_type: str = "int8"  # Options: int8, float16, float32

    # VAD settings
    vad_threshold: float = 0.5  # Voice activity detection threshold
    vad_min_speech_duration: float = 0.25  # Minimum speech duration in seconds
    vad_min_silence_duration: float = 0.5  # Silence duration to end speech

    # TTS settings
    tts_provider: str = "chatterbox"  # Options: chatterbox, pyttsx3, edge-tts
    tts_voice: str = "default"
    tts_sample_rate: int = 24000

    # Audio settings
    audio_sample_rate: int = 16000  # Input audio sample rate
    audio_channels: int = 1  # Mono audio
    audio_chunk_size: int = 4096  # Samples per chunk

    # Session settings
    session_timeout: int = 300  # Session timeout in seconds
    max_audio_buffer_size: int = 1024 * 1024 * 10  # 10MB max audio buffer

    class Config:
        env_prefix = "ENGRAM_VOICE_"
        env_file = ".env"
        env_file_encoding = "utf-8"


settings = Settings()
