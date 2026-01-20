"""
ENGRAM Voice Service - FastAPI + WebSocket entry point.

This service handles voice-based flashcard review sessions using:
- Silero VAD for voice activity detection
- faster-whisper for speech-to-text
- Chatterbox (or fallback TTS) for text-to-speech
"""

import asyncio
import logging
from contextlib import asynccontextmanager
from typing import AsyncGenerator

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware

from .config import settings
from .pipeline import VoicePipeline
from .session import VoiceSession, SessionState

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)

# Global pipeline instance (initialized at startup)
voice_pipeline: VoicePipeline | None = None


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    """Initialize and cleanup voice pipeline on app lifecycle."""
    global voice_pipeline

    logger.info("Initializing voice pipeline...")
    voice_pipeline = VoicePipeline(
        whisper_model=settings.whisper_model,
        whisper_device=settings.whisper_device,
        vad_threshold=settings.vad_threshold,
    )
    await voice_pipeline.initialize()
    logger.info("Voice pipeline ready")

    yield

    logger.info("Shutting down voice pipeline...")
    if voice_pipeline:
        await voice_pipeline.shutdown()
    logger.info("Voice pipeline shutdown complete")


app = FastAPI(
    title="ENGRAM Voice Service",
    description="Voice processing for flashcard review sessions",
    version="0.1.0",
    lifespan=lifespan,
)

# CORS configuration
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/health")
async def health_check() -> dict:
    """Health check endpoint."""
    return {
        "status": "healthy",
        "pipeline_ready": voice_pipeline is not None and voice_pipeline.is_ready,
    }


@app.websocket("/ws/voice")
async def voice_websocket(websocket: WebSocket) -> None:
    """
    WebSocket endpoint for voice sessions.

    Protocol:
    - Client connects and sends {"type": "start_session", "deck_id": "..."}
    - Server sends card fronts as TTS audio
    - Client sends audio chunks for answer
    - Server transcribes and evaluates, sends next card
    - Session ends on client disconnect or {"type": "end_session"}
    """
    await websocket.accept()

    if not voice_pipeline or not voice_pipeline.is_ready:
        await websocket.send_json({
            "type": "error",
            "message": "Voice pipeline not ready",
        })
        await websocket.close(code=1011)
        return

    session = VoiceSession(
        websocket=websocket,
        pipeline=voice_pipeline,
        api_base_url=settings.api_base_url,
    )

    try:
        logger.info(f"Voice session started: {session.session_id}")
        await session.run()
    except WebSocketDisconnect:
        logger.info(f"Voice session disconnected: {session.session_id}")
    except Exception as e:
        logger.error(f"Voice session error: {e}", exc_info=True)
        try:
            await websocket.send_json({
                "type": "error",
                "message": str(e),
            })
        except Exception:
            pass
    finally:
        await session.cleanup()
        logger.info(f"Voice session ended: {session.session_id}")


@app.websocket("/ws/voice/stream")
async def voice_stream_websocket(websocket: WebSocket) -> None:
    """
    Streaming WebSocket for real-time voice processing.

    This endpoint handles continuous audio streaming with VAD-based
    turn detection, suitable for conversational interactions.
    """
    await websocket.accept()

    if not voice_pipeline or not voice_pipeline.is_ready:
        await websocket.send_json({
            "type": "error",
            "message": "Voice pipeline not ready",
        })
        await websocket.close(code=1011)
        return

    session = VoiceSession(
        websocket=websocket,
        pipeline=voice_pipeline,
        api_base_url=settings.api_base_url,
        streaming=True,
    )

    try:
        logger.info(f"Streaming voice session started: {session.session_id}")
        await session.run_streaming()
    except WebSocketDisconnect:
        logger.info(f"Streaming session disconnected: {session.session_id}")
    except Exception as e:
        logger.error(f"Streaming session error: {e}", exc_info=True)
    finally:
        await session.cleanup()


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        "src.main:app",
        host=settings.host,
        port=settings.port,
        reload=settings.debug,
        log_level="info",
    )
