# ENGRAM Voice Flashcard App - Implementation State

**Document**: state_001.md
**Date**: 2026-01-20
**Status**: MVP 100% Complete
**Supersedes**: plan_001.md (original implementation plan)

---

## Executive Summary

The ENGRAM voice flashcard application has achieved **complete MVP status**. All core functionality is implemented, tested, and operational. This document reflects the **verified current state** of the codebase.

| Component | Status | Notes |
|-----------|--------|-------|
| Rust Backend (Axum) | 100% | All endpoints, FSRS, LLM integration |
| Python Voice Service | 100% | Pipecat, VAD, STT, TTS pipeline |
| SvelteKit Frontend | 100% | PWA, offline support, voice controls, PDF upload |
| Infrastructure | 100% | Docker, README, configuration |
| Code Quality | 100% | No compiler warnings, TypeScript clean |

---

## Tech Stack (Implemented)

| Layer | Technology | Status |
|-------|------------|--------|
| **Frontend** | SvelteKit 5 PWA + vanilla SCSS | Implemented |
| **Voice Service** | Python + FastAPI + Pipecat | Implemented |
| **Data API** | Rust + Axum | Implemented |
| **Database** | SQLite (server), IndexedDB (client) | Implemented |
| **STT** | faster-whisper (self-hosted) | Implemented |
| **TTS** | Chatterbox + edge-tts fallback | Implemented |
| **LLM** | Gemini, OpenAI, Anthropic (multi-provider) | Implemented |
| **VAD** | Silero VAD | Implemented |
| **SR Algorithm** | FSRS (Free Spaced Repetition Scheduling) | Implemented |

---

## Architecture (Actual)

```
┌────────────────────────────────────────────────────────────┐
│                    SvelteKit PWA                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Review   │  │ Card     │  │ Ingest   │  │ Settings │  │
│  │ (Voice)  │  │ Library  │  │ Content  │  │          │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
└───────┼─────────────┼─────────────┼─────────────┼─────────┘
        │WebSocket    │REST         │REST         │REST
        │(configurable)
        ▼             ▼             ▼             ▼
┌────────────────┐  ┌──────────────────────────────────────┐
│  Pipecat       │  │         Rust API (Axum)              │
│  Voice Service │  │                                      │
│  Port: 8001    │  │  GET  /api/cards       - list/filter │
│  ┌──────────┐  │  │  POST /api/cards       - create      │
│  │ Silero   │  │  │  PATCH/api/cards/:id   - update      │
│  │ VAD      │  │  │  GET  /api/review/next - due cards   │
│  └────┬─────┘  │  │  POST /api/review/submit - record    │
│  ┌────▼─────┐  │  │  POST /api/review/evaluate - LLM     │
│  │ faster-  │──┼──│  POST /api/ingest/text               │
│  │ whisper  │  │  │  POST /api/ingest/url                │
│  └────┬─────┘  │  │  POST /api/ingest/pdf                │
│  ┌────▼─────┐  │  │  POST /api/ingest/confirm            │
│  │ Gemini   │  │  │  WS   /api/ws          - text session│
│  │ Flash    │  │  │                                      │
│  │ (eval)   │  │  └─────────────────┬────────────────────┘
│  └────┬─────┘  │                    │
│  ┌────▼─────┐  │                    ▼
│  │Chatterbox│  │            ┌──────────────┐
│  │  TTS     │  │            │   SQLite     │
│  └──────────┘  │            │ (engram.db)  │
└────────────────┘            └──────────────┘
```

### Service Communication
- **Frontend ↔ Rust API**: REST over HTTPS for all data operations
- **Frontend ↔ Voice Service**: WebSocket (configurable via `VITE_USE_PYTHON_VOICE`)
  - Python voice service: `ws://host:8001/ws/voice/stream`
  - Rust backend (text-only): `ws://host/api/ws`
- **Voice Service ↔ Rust API**: REST calls for card data

---

## Project Structure (Actual)

```
/home/goose/ENGRAM/
├── README.md                         # Project documentation (221 lines)
├── docker-compose.yml                # Full orchestration (95 lines)
├── dev.sh                            # tmux development script
├── .env.example                      # Environment template
│
├── apps/
│   ├── backend/                      # Rust Axum data API
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── main.rs               # Entry point (82 lines)
│   │       ├── config.rs             # Configuration (190 lines)
│   │       ├── error.rs              # Error types (190 lines)
│   │       ├── routes/
│   │       │   ├── mod.rs            # Route registration
│   │       │   ├── cards.rs          # Card CRUD
│   │       │   ├── review.rs         # Review + FSRS scheduling
│   │       │   ├── ingest.rs         # Content ingestion
│   │       │   └── ws.rs             # WebSocket voice handler
│   │       ├── models/
│   │       │   ├── mod.rs
│   │       │   ├── card.rs           # Card with FSRS fields
│   │       │   ├── review.rs         # Review + ratings
│   │       │   └── source.rs         # Content sources
│   │       ├── db/
│   │       │   ├── mod.rs            # Migration runner
│   │       │   └── migrations/
│   │       │       ├── 000_migrations_table.sql
│   │       │       ├── 001_create_sources.sql
│   │       │       ├── 002_create_cards.sql
│   │       │       └── 003_create_reviews.sql
│   │       ├── services/
│   │       │   ├── mod.rs
│   │       │   ├── spaced_repetition.rs  # FSRS-4.5 (378 lines, 8 tests)
│   │       │   └── pdf_processor.rs      # PDF extraction (273 lines)
│   │       └── llm/
│   │           ├── mod.rs
│   │           ├── provider.rs       # Trait + types
│   │           ├── manager.rs        # Fallback chain
│   │           ├── gemini.rs         # Google Gemini
│   │           ├── openai.rs         # OpenAI
│   │           └── anthropic.rs      # Anthropic
│   │
│   ├── voice/                        # Python Pipecat voice service
│   │   ├── pyproject.toml            # Package config (53 lines)
│   │   ├── requirements.txt          # Dependencies (28 lines)
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── __init__.py
│   │       ├── main.py               # FastAPI + WebSocket (172 lines)
│   │       ├── config.py             # Pydantic settings (52 lines)
│   │       ├── pipeline.py           # Pipecat pipeline (345 lines)
│   │       ├── session.py            # 10-state machine (444 lines)
│   │       ├── api_client.py         # REST client (189 lines)
│   │       ├── prompts.py            # LLM prompts (175 lines)
│   │       ├── tts_chatterbox.py     # Chatterbox TTS (163 lines)
│   │       └── tts_fallback.py       # edge-tts fallback (211 lines)
│   │
│   └── frontend/                     # SvelteKit PWA
│       ├── package.json
│       ├── svelte.config.js
│       ├── vite.config.ts
│       ├── tsconfig.json
│       ├── Dockerfile
│       ├── static/
│       │   ├── manifest.json         # PWA manifest
│       │   ├── service-worker.js     # Offline support (238 lines)
│       │   ├── audio-processor.worklet.js  # AudioWorklet (152 lines)
│       │   ├── offline.html          # Offline fallback
│       │   ├── favicon.svg
│       │   ├── icon-192.svg
│       │   └── icon-512.svg
│       └── src/
│           ├── app.html              # SW registration
│           ├── routes/
│           │   ├── +layout.svelte    # Theme management
│           │   ├── +page.svelte      # Dashboard
│           │   ├── review/+page.svelte   # Voice review
│           │   ├── cards/+page.svelte    # Card library
│           │   ├── ingest/+page.svelte   # Content ingestion
│           │   └── settings/+page.svelte # Preferences
│           └── lib/
│               ├── components/
│               │   ├── VoiceControls.svelte  # Audio capture
│               │   ├── ReviewCard.svelte     # Card display
│               │   └── CardItem.svelte       # List item
│               ├── stores/
│               │   ├── cards.ts      # Card management
│               │   ├── review.ts     # Review session
│               │   └── settings.ts   # User prefs (sessionStorage for API key)
│               ├── db/
│               │   └── index.ts      # Dexie IndexedDB
│               └── api/
│                   ├── client.ts     # REST client
│                   └── websocket.ts  # Voice WebSocket (configurable)
│
├── packages/
│   └── shared/                       # Shared TypeScript types
│       ├── package.json
│       └── src/
│           └── index.ts
│
└── docs/
    └── MVP/
        ├── state_001.md              # This document
        ├── plan_001.md               # Original plan (reference)
        ├── plan_001--AUDIT-v2.md     # Audit (contains some inaccuracies)
        └── plan_001--IMP-ISSUES--OBSOLETE.md  # Archived
```

---

## Implementation Phases - Status

### Phase 1: Foundation ✅ COMPLETE
- Rust API with SQLite (sqlx) - implemented
- Card, Review, Source models - implemented
- CRUD endpoints for cards - implemented
- `/review/next` (fetch due cards) - implemented
- `/review/submit` (record review) - implemented
- SvelteKit with static adapter - implemented
- Dexie for IndexedDB - implemented
- Card store with local persistence - implemented
- Card list/create/edit UI - implemented
- Basic text-only review UI - implemented

### Phase 2: FSRS Scheduling ✅ COMPLETE
- FSRS-4.5 algorithm in `services/spaced_repetition.rs` - implemented
  - 17-parameter optimized weights
  - Stability, difficulty, retrievability calculations
  - 8 comprehensive unit tests
- FSRS state per card (stability, difficulty, lapses) - implemented
- `/review/submit` uses FSRS directly - implemented
- WebSocket handler uses FSRS directly - implemented (fixed 2026-01-20)
- Cards ordered by retrievability (lowest first) - implemented

### Phase 3: Voice Pipeline ✅ COMPLETE
- Pipecat project with FastAPI - implemented
- Pipeline: Silero VAD → faster-whisper → LLM → Chatterbox/fallback - implemented
- WebSocket endpoints:
  - `/ws/voice` - Command-based interaction
  - `/ws/voice/stream` - Real-time streaming with VAD
- Session state machine (10 states) - implemented:
  - IDLE → STARTING → PRESENTING_CARD → LISTENING → PROCESSING
  - EVALUATING → PRESENTING_FEEDBACK → ENDING → ENDED | ERROR
- REST client for Rust API integration - implemented
- Frontend WebSocket client - implemented (configurable)
- AudioWorklet for audio capture (24kHz PCM) - implemented
- Audio playback queue - implemented
- Voice review UI with visualizer - implemented

### Phase 4: LLM Evaluation ✅ COMPLETE
- Multi-provider LLM support (Gemini, OpenAI, Anthropic) - implemented
- Evaluation prompt with structured output - implemented
- Transcription → LLM eval → FSRS rating → TTS feedback - implemented
- Edge case handling (silence, "I don't know") - implemented

### Phase 5: Content Ingestion ✅ COMPLETE
- PDF text extraction (`pdf_processor.rs`) - implemented
- URL content fetching with HTML stripping - implemented
- Card generation endpoints:
  - `POST /api/ingest/text` - implemented
  - `POST /api/ingest/url` - implemented
  - `POST /api/ingest/pdf` - implemented (multipart upload)
  - `POST /api/ingest/confirm` - implemented
- Ingestion UI with three tabs:
  - Text: Paste content with optional title
  - URL: Fetch and extract from web pages
  - PDF: Upload PDF files (up to 50MB)
- Staged cards with edit/approve/reject - implemented

### Phase 6: Native Audio Migration 🔄 READY
- Architecture supports provider switching
- Environment variable `VITE_USE_PYTHON_VOICE` controls mode
- Gemini Live API integration point identified in voice service
- Current setup: Chained pipeline (STT → LLM → TTS)

### Phase 7: PWA & Polish ✅ COMPLETE
- Service worker for offline app shell - implemented
- PWA manifest with SVG icons - implemented
- Offline review queue (IndexedDB sync) - implemented
- Error handling, loading states - implemented
- Settings UI - implemented
- Theme support (light/dark/system) - implemented

---

## FSRS Implementation Details

```rust
pub struct FSRSState {
    pub stability: f64,      // Days until 90% retention
    pub difficulty: f64,     // 1.0-10.0 scale
    pub reps: i32,          // Review count
    pub lapses: i32,        // Times rated "Again"
    pub last_review: Option<DateTime<Utc>>,
}

// FSRS-4.5 optimized weights
pub w: [f64; 17] = [
    0.4, 0.6, 2.4, 5.8,     // Initial stability [Again, Hard, Good, Easy]
    4.93, 0.94, 0.86, 0.01, // Difficulty weights
    1.49, 0.14, 0.94,       // Stability after success
    2.18, 0.05, 0.34, 1.26, // Stability after failure
    0.29, 2.61,             // Hard penalty, Easy bonus
];
```

Database schema supports both FSRS and legacy SM-2 fields:
```sql
CREATE TABLE cards (
    -- Legacy SM-2 (backward compatibility)
    ease_factor REAL DEFAULT 2.5,
    interval INTEGER DEFAULT 0,
    repetitions INTEGER DEFAULT 0,
    -- FSRS fields
    stability REAL DEFAULT 0.0,
    difficulty REAL DEFAULT 5.0,
    lapses INTEGER DEFAULT 0,
    -- Scheduling
    next_review TEXT NOT NULL,
    last_review TEXT,
    ...
);
```

---

## Security Measures

| Concern | Implementation |
|---------|----------------|
| API Key Storage | `sessionStorage` (not localStorage) - cleared on browser close |
| XSS Prevention | Svelte's built-in escaping |
| CORS | tower-http CORS middleware |
| Input Validation | Rust type system + serde |

---

## Known Minor Issues (Non-Blocking)

### Accessibility Warnings (a11y)
The following accessibility improvements are recommended but not required for MVP:
- Settings page: Some labels need explicit control associations
- Review page: Minor keyboard navigation improvements

### Reserved Code (Intentionally Unused)
The following code is intentionally kept for future features and marked with `#[allow(dead_code)]`:
- `calculate_sm2()` - SM-2 compatibility wrapper
- `needs_migration()` - Database migration check utility
- `ok_or_not_found()` - Error handling helper
- `AudioChunk` variant - Reserved for TTS audio streaming
- `with_model()` methods - Reserved for model selection feature

---

## Environment Configuration

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

# Frontend
VITE_API_URL=http://localhost:3001
VITE_VOICE_SERVICE_PORT=8001
VITE_USE_PYTHON_VOICE=true  # Enable Python voice service
```

---

## Verification Checklist

| Phase | Test | Status |
|-------|------|--------|
| 1 | CRUD cards via API and UI | ✅ |
| 2 | Review card 5 times, verify FSRS intervals | ✅ |
| 3 | Voice session: hear card, speak, see transcription | ✅ |
| 4 | Voice review: get AI feedback, verify rating affects schedule | ✅ |
| 5 | Upload PDF, generate cards, edit, confirm | ✅ |
| 6 | A/B test chained vs native audio | 🔄 Ready |
| 7 | Install PWA, review offline, verify sync | ✅ |

---

## Cost Estimates (Self-Hosted)

| Component | Cost |
|-----------|------|
| faster-whisper | $0 (self-hosted) |
| Chatterbox/edge-tts | $0 (self-hosted) |
| Gemini Flash (eval) | ~$0.0001/review |
| **Total per review** | **~$0.0001** |

100 reviews/day = **~$0.30/month** for LLM alone

---

## Quick Start

```bash
# Clone and install
git clone <repo>
cd ENGRAM
pnpm install
cd apps/voice && pip install -e . && cd ../..

# Configure
cp .env.example .env
# Edit .env with API keys

# Run (Option 1: tmux)
./dev.sh

# Run (Option 2: Docker)
docker-compose up -d

# Access
# Frontend: http://localhost:5173
# Backend:  http://localhost:3001
# Voice:    http://localhost:8001
```

---

## Out of Scope (Future Work)

- Multi-user / authentication
- Analytics dashboard
- Anki import
- Native mobile apps
- Cloze deletion cards
- Phone number access (Telnyx integration)
- Gemini Native Audio (architecture ready)

---

## Changelog

### 2026-01-20 (MVP 100%)
- **PDF Upload**: Added PDF file upload UI to ingest page (was missing despite backend support)
- **Type Fixes**: Updated shared TypeScript types for voice session messages
- **Dead Code**: Added `#[allow(dead_code)]` annotations for intentionally reserved code
- **Clean Build**: Achieved zero compiler warnings in Rust backend
- **TypeScript Clean**: Achieved zero TypeScript errors in frontend

### 2026-01-20 (Initial)
- Archived obsolete `plan_001--IMP-ISSUES.md`
- Fixed WebSocket handler to use FSRS directly (was using SM-2 wrapper)
- Verified all critical issues from AUDIT-v2:
  - Service worker: Already uses correct SVG paths
  - AudioWorklet: Already handles sampleRate correctly
  - Settings store: Already uses sessionStorage for API keys
  - Voice service: Frontend connection properly configurable
  - docker-compose.yml: Exists and complete
  - README.md: Exists and comprehensive
- Created state_001.md (this document)

---

**Document Status**: MVP COMPLETE. This document reflects the verified implementation state as of 2026-01-20 and supersedes plan_001.md for current status reference.
