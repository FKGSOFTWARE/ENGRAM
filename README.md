# ENGRAM - Voice-First Flashcard Application

ENGRAM is a voice-enabled spaced repetition flashcard application that uses AI to evaluate your spoken answers. Built with a modern tech stack featuring Rust, Python, and SvelteKit.

## Features

- **Voice-First Learning**: Speak your answers and get AI-powered feedback
- **FSRS Algorithm**: Uses the Free Spaced Repetition Scheduling algorithm for optimal retention
- **AI Content Ingestion**: Generate flashcards from text, URLs, or PDFs
- **Multi-Provider LLM Support**: Works with Gemini, OpenAI, or Anthropic
- **Offline-First PWA**: Review cards offline with automatic sync
- **Self-Hosted**: Keep your data private with local deployment

## Architecture

```
                    ┌─────────────────────────────────────────────┐
                    │             SvelteKit PWA                   │
                    │  (Review, Cards, Ingest, Settings)          │
                    └─────────────────┬───────────────────────────┘
                                      │
              ┌───────────────────────┼───────────────────────┐
              │ REST                  │ WebSocket             │ REST
              ▼                       ▼                       ▼
    ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
    │   Rust Backend  │     │  Python Voice   │     │   Rust Backend  │
    │   (Axum API)    │◄────│    (Pipecat)    │────►│   (Axum API)    │
    │                 │     │                 │     │                 │
    │  - Cards CRUD   │     │  - Silero VAD   │     │  - LLM Eval     │
    │  - FSRS         │     │  - Whisper STT  │     │  - Reviews      │
    │  - PDF Ingest   │     │  - Chatterbox   │     │                 │
    └────────┬────────┘     └─────────────────┘     └────────┬────────┘
             │                                                │
             └──────────────────────┬─────────────────────────┘
                                    ▼
                           ┌─────────────────┐
                           │     SQLite      │
                           │   (engram.db)   │
                           └─────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | SvelteKit 5, TypeScript, Dexie (IndexedDB) |
| Backend API | Rust, Axum, SQLx |
| Voice Service | Python, FastAPI, Pipecat |
| STT | faster-whisper (self-hosted) |
| TTS | Chatterbox / edge-tts (fallback) |
| VAD | Silero VAD |
| LLM | Gemini Flash, OpenAI, Anthropic |
| Database | SQLite |
| SR Algorithm | FSRS (Free Spaced Repetition Scheduling) |

## Quick Start

### Prerequisites

- Node.js 20+ and pnpm 9+
- Rust 1.75+ and Cargo
- Python 3.10+ and pip
- (Optional) CUDA for GPU-accelerated Whisper

### Development Setup

1. **Clone and install dependencies**

```bash
git clone https://github.com/yourusername/ENGRAM.git
cd ENGRAM

# Install frontend dependencies
pnpm install

# Install Python voice service dependencies
cd apps/voice
pip install -e .
cd ../..
```

2. **Configure environment**

```bash
cp .env.example .env
# Edit .env with your API keys (at least one LLM provider required)
```

3. **Start development servers**

```bash
# Option 1: Use the dev script (requires tmux)
./dev.sh

# Option 2: Run services separately
# Terminal 1 - Backend
cargo run -p engram-backend

# Terminal 2 - Voice Service
cd apps/voice && python -m src.main

# Terminal 3 - Frontend
pnpm dev:frontend
```

4. **Access the app**

- Frontend: http://localhost:3000
- Backend API: http://localhost:3001
- Voice Service: http://localhost:8001

### Docker Setup

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f
```

## Project Structure

```
ENGRAM/
├── apps/
│   ├── backend/          # Rust Axum API
│   │   └── src/
│   │       ├── routes/   # HTTP endpoints
│   │       ├── models/   # Data models
│   │       ├── services/ # FSRS, PDF processing
│   │       └── llm/      # LLM provider integrations
│   │
│   ├── voice/            # Python voice service
│   │   └── src/
│   │       ├── main.py       # FastAPI entry
│   │       ├── pipeline.py   # Pipecat voice pipeline
│   │       ├── session.py    # Voice session state machine
│   │       └── prompts.py    # LLM evaluation prompts
│   │
│   └── frontend/         # SvelteKit PWA
│       └── src/
│           ├── routes/   # Pages
│           ├── lib/
│           │   ├── api/       # REST & WebSocket clients
│           │   ├── components/
│           │   ├── stores/    # Svelte stores
│           │   └── db/        # IndexedDB (Dexie)
│           └── static/   # PWA assets
│
├── packages/
│   └── shared/           # Shared TypeScript types
│
└── docs/
    └── MVP/              # Planning documents
```

## API Endpoints

### Cards
- `GET /api/cards` - List all cards
- `POST /api/cards` - Create card
- `GET /api/cards/:id` - Get card
- `PATCH /api/cards/:id` - Update card
- `DELETE /api/cards/:id` - Delete card

### Review
- `GET /api/review/next?limit=10` - Get due cards
- `POST /api/review/submit` - Submit review with rating
- `POST /api/review/evaluate` - Get LLM evaluation only

### Ingest
- `POST /api/ingest/text` - Generate cards from text
- `POST /api/ingest/url` - Generate cards from URL
- `POST /api/ingest/pdf` - Generate cards from PDF
- `POST /api/ingest/confirm` - Confirm staged cards

### WebSocket
- `WS /api/ws` - Rust backend (text-only)
- `WS /ws/voice/stream` - Python voice service (STT/TTS)

## Configuration

### Environment Variables

```bash
# Backend
DATABASE_URL=sqlite:engram.db?mode=rwc
RUST_LOG=engram_backend=debug
HOST=0.0.0.0
PORT=3001

# LLM Providers (at least one required)
GEMINI_API_KEY=your-key
OPENAI_API_KEY=your-key
ANTHROPIC_API_KEY=your-key

# Voice Service
WHISPER_MODEL=base          # tiny, base, small, medium, large
WHISPER_DEVICE=cpu          # cpu or cuda
TTS_PROVIDER=fallback       # chatterbox or fallback
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [FSRS](https://github.com/open-spaced-repetition/fsrs4anki) - Spaced repetition algorithm
- [Pipecat](https://github.com/pipecat-ai/pipecat) - Voice AI framework
- [faster-whisper](https://github.com/guillaumekln/faster-whisper) - Fast Whisper implementation
- [Chatterbox](https://github.com/resemble-ai/chatterbox) - Open-source TTS
