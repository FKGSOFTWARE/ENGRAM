# ENGRAM Voice Service

Python-based voice processing service using Pipecat for real-time voice flashcard review.

## Features

- Silero VAD (Voice Activity Detection)
- faster-whisper STT (Speech-to-Text)
- Chatterbox/edge-tts TTS (Text-to-Speech)
- WebSocket streaming for real-time audio

## Installation

```bash
pip install -e .
```

## Running

```bash
python -m src.main
```

Service runs on port 8001 by default.
