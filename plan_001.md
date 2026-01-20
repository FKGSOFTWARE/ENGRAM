# ENGRAM Voice Flashcard App - Implementation Plan

## Tech Stack (Revised)

| Layer | Technology | Notes |
|-------|------------|-------|
| **Frontend** | SvelteKit PWA + vanilla SCSS | Browser-first, PWA for mobile |
| **Voice Service** | Python + Pipecat | Handles VAD, STT, TTS, voice session |
| **Data API** | Rust + Axum | Cards, reviews, sources, user data |
| **Database** | SQLite (server), IndexedDB (client) | Single-file DB, easy dev/deploy |
| **STT** | faster-whisper (self-hosted) | Open source, near-zero cost |
| **TTS** | Chatterbox (self-hosted) | MIT license, emotion control |
| **LLM** | Gemini Flash (eval), flexible providers | Chained for MVP, native audio ready |
| **VAD** | Silero VAD | Industry standard, 1.8MB model |
| **SR Algorithm** | FSRS | 15-20% more efficient than SM-2 |

## Architecture Overview

```
┌────────────────────────────────────────────────────────────┐
│                    SvelteKit PWA                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Review   │  │ Card     │  │ Ingest   │  │ Settings │  │
│  │ (Voice)  │  │ Library  │  │ Content  │  │          │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
└───────┼─────────────┼─────────────┼─────────────┼─────────┘
        │WebSocket    │REST         │REST         │REST
        ▼             ▼             ▼             ▼
┌────────────────┐  ┌──────────────────────────────────────┐
│  Pipecat       │  │           Rust API (Axum)            │
│  Voice Service │  │                                      │
│  ┌──────────┐  │  │  GET  /cards         - list/filter   │
│  │ Silero   │  │  │  POST /cards         - create        │
│  │ VAD      │  │  │  PATCH/cards/:id     - update        │
│  └────┬─────┘  │  │  GET  /review/next   - due cards     │
│  ┌────▼─────┐  │  │  POST /review/submit - record answer │
│  │ faster-  │──┼──│  POST /ingest        - process PDF   │
│  │ whisper  │  │  │  POST /ingest/:id/confirm            │
│  └────┬─────┘  │  │                                      │
│  ┌────▼─────┐  │  └─────────────────┬────────────────────┘
│  │ Gemini   │  │                    │
│  │ Flash    │  │                    ▼
│  │ (eval)   │  │            ┌──────────────┐
│  └────┬─────┘  │            │   SQLite     │
│  ┌────▼─────┐  │            │ (cards.db)   │
│  │Chatterbox│  │            └──────────────┘
│  │  TTS     │  │
│  └──────────┘  │
└────────────────┘
```

### Service Communication
- **Frontend ↔ Rust API**: REST over HTTPS for all data operations
- **Frontend ↔ Pipecat**: WebSocket for voice sessions (audio streaming)
- **Pipecat ↔ Rust API**: REST calls for card data during voice sessions
- **Future-ready**: API interfaces designed for gRPC migration if needed

## Project Structure

```
/home/goose/ENGRAM/
├── README.md
├── docker-compose.yml            # Local dev orchestration
│
├── apps/
│   ├── api/                      # Rust Axum data API
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── config.rs
│   │       ├── error.rs
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── cards.rs
│   │       │   ├── review.rs
│   │       │   └── ingest.rs
│   │       ├── models/
│   │       │   ├── mod.rs
│   │       │   ├── card.rs
│   │       │   ├── review.rs
│   │       │   └── source.rs
│   │       ├── db/
│   │       │   ├── mod.rs
│   │       │   └── migrations/
│   │       └── services/
│   │           ├── mod.rs
│   │           ├── fsrs.rs           # FSRS algorithm
│   │           ├── pdf_processor.rs
│   │           └── llm_client.rs     # For card generation
│   │
│   ├── voice/                    # Python Pipecat voice service
│   │   ├── pyproject.toml
│   │   ├── requirements.txt
│   │   └── src/
│   │       ├── __init__.py
│   │       ├── main.py               # FastAPI + WebSocket entry
│   │       ├── pipeline.py           # Pipecat pipeline config
│   │       ├── session.py            # Voice session state machine
│   │       ├── api_client.py         # Calls Rust API
│   │       └── prompts.py            # LLM evaluation prompts
│   │
│   └── web/                      # SvelteKit PWA
│       ├── package.json
│       ├── svelte.config.js
│       ├── vite.config.ts
│       ├── static/
│       │   └── manifest.json
│       └── src/
│           ├── app.html
│           ├── routes/
│           │   ├── +layout.svelte
│           │   ├── +page.svelte          # Dashboard
│           │   ├── review/+page.svelte   # Voice review
│           │   ├── cards/+page.svelte    # Card library
│           │   ├── ingest/+page.svelte   # Content ingestion
│           │   └── settings/+page.svelte
│           └── lib/
│               ├── components/
│               ├── stores/
│               ├── db/                   # Dexie IndexedDB
│               └── api/
│                   ├── rest.ts           # Rust API client
│                   └── voice.ts          # Pipecat WebSocket
│
├── packages/
│   └── types/                    # Shared TypeScript types
│       ├── package.json
│       └── src/
│           └── index.ts
│
└── docs/
    └── MVP/
        └── plan_001.md           # This file
```

## Implementation Phases

### Phase 1: Foundation
**Goal**: Data layer working end-to-end

**Rust API:**
1. Initialize Axum project with SQLite (sqlx)
2. Define models: Card, Review, Source
3. Implement CRUD endpoints for cards
4. Implement `/review/next` (fetch due cards)
5. Implement `/review/submit` (record review, placeholder scoring)

**SvelteKit:**
1. Initialize SvelteKit with static adapter
2. Setup Dexie for IndexedDB
3. Create card store with local persistence
4. Build card list/create/edit UI
5. Build basic text-only review UI

**Verification**: Create cards, review them with manual scoring, see them scheduled

### Phase 2: FSRS Scheduling
**Goal**: Proper spaced repetition

**Rust API:**
1. Implement FSRS algorithm in `services/fsrs.rs`:
   ```rust
   struct FSRSState {
       stability: f32,      // Days until 90% retention
       difficulty: f32,     // 1-10 scale
       retrievability: f32, // Current recall probability
   }

   fn next_interval(state: &FSRSState, rating: u8) -> (FSRSState, i32)
   ```
2. Store FSRS state per card-user pair
3. Update `/review/submit` to use FSRS
4. Order due cards by retrievability (lowest first)

**Verification**: Review cards multiple times, verify intervals increase appropriately

### Phase 3: Voice Pipeline (Core)
**Goal**: Voice review loop without LLM evaluation

**Python Voice Service:**
1. Initialize Pipecat project with FastAPI
2. Configure pipeline: Silero VAD → faster-whisper → placeholder → Chatterbox
3. Implement WebSocket endpoint for voice sessions
4. Build session state machine:
   - `IDLE` → `PRESENTING` → `LISTENING` → `PROCESSING` → `FEEDBACK` → loop
5. Integrate REST client to fetch cards from Rust API
6. Generate TTS for card fronts

**SvelteKit:**
1. Implement WebSocket client for voice service
2. Build audio capture with AudioWorklet (24kHz PCM)
3. Build audio playback queue
4. Create voice review UI with visualizer

**Verification**: Start voice session, hear card read aloud, speak answer, hear "Processing..." feedback

### Phase 4: LLM Evaluation
**Goal**: AI-powered answer evaluation

**Python Voice Service:**
1. Add Gemini Flash client (for text eval)
2. Implement evaluation prompt:
   ```python
   EVAL_PROMPT = """
   Card front: {front}
   Expected answer: {back}
   User's answer (transcribed): {user_answer}

   Evaluate: {"correct": bool, "score": 0-100, "quality": 0-5, "feedback": "..."}
   """
   ```
3. Wire transcription → LLM eval → FSRS rating → TTS feedback
4. Handle edge cases: silence, "I don't know", interruptions

**Verification**: Complete voice review with AI feedback, verify scheduling reflects performance

### Phase 5: Content Ingestion
**Goal**: Generate cards from content

**Rust API:**
1. Implement PDF text extraction (pdf-extract or pdfium)
2. Implement URL content fetching (readability-style extraction)
3. Add card generation endpoint that calls LLM:
   ```
   POST /ingest { type: "pdf"|"text"|"url", content: ... }
   → Returns staged_cards[]

   POST /ingest/:id/confirm { card_ids: [...], edits: {...} }
   → Moves to card library
   ```

**SvelteKit:**
1. Build ingestion UI: paste text, upload PDF, enter URL
2. Display staged cards with edit/approve/reject
3. Bulk confirm flow

**Verification**: Upload PDF, review generated cards, edit one, confirm to library

### Phase 6: Native Audio Migration
**Goal**: Switch to Gemini Native Audio for production quality

**Python Voice Service:**
1. Implement Gemini Live API client
2. Add provider switching (chained vs native)
3. Update pipeline to use native audio when configured
4. A/B test latency and quality

**Verification**: Same voice review flow, faster response, more natural conversation

### Phase 7: PWA & Polish
**Goal**: Production-ready MVP

1. Service worker for offline app shell
2. PWA manifest with icons
3. Offline review queue (sync when back online)
4. Error handling, loading states, user feedback
5. Settings UI for voice speed, batch size, API keys (BYOK)

**Verification**: Install as PWA, review cards offline, changes sync when online

## Critical Files

| File | Purpose |
|------|---------|
| `apps/api/src/services/fsrs.rs` | FSRS spaced repetition algorithm |
| `apps/api/src/routes/review.rs` | Due cards, submit reviews |
| `apps/voice/src/pipeline.py` | Pipecat voice pipeline config |
| `apps/voice/src/session.py` | Voice session state machine |
| `apps/web/src/lib/api/voice.ts` | WebSocket client for voice |
| `apps/web/src/lib/db/index.ts` | Dexie IndexedDB schema |

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Voice framework | Pipecat | Handles VAD, turn-taking, provider switching OOTB |
| STT | faster-whisper | Open source, accurate, self-hosted, near-zero cost |
| TTS | Chatterbox | MIT license, emotion control, voice cloning, self-hosted |
| SR Algorithm | FSRS | 15-20% fewer reviews than SM-2 for same retention |
| Voice → Data comms | REST | Simple, stateless, debuggable; can migrate to gRPC |
| LLM for MVP | Gemini Flash | Cheap, fast, good enough for eval; native audio ready |
| Database | SQLite | Single file, easy dev, sufficient for single-user MVP |

## Dependencies

**Rust (apps/api/Cargo.toml):**
```toml
[dependencies]
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
reqwest = { version = "0.12", features = ["json"] }
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Python (apps/voice/requirements.txt):**
```
pipecat-ai[silero,google]
faster-whisper
chatterbox-tts
google-generativeai
httpx
python-dotenv
```

**Frontend (apps/web/package.json):**
```json
{
  "dependencies": {
    "dexie": "^4.0.0"
  },
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0.0",
    "@sveltejs/kit": "^2.0.0",
    "svelte": "^5.0.0",
    "sass": "^1.70.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0"
  }
}
```

## Verification Plan

| Phase | Test |
|-------|------|
| 1 | CRUD cards via API and UI, basic review loop |
| 2 | Review same card 5 times, verify FSRS intervals |
| 3 | Voice session: hear card, speak, see transcription |
| 4 | Voice review: get AI feedback, verify rating affects schedule |
| 5 | Upload PDF, generate cards, edit, confirm |
| 6 | A/B test chained vs native audio latency |
| 7 | Install PWA, review offline, verify sync |

## Cost Estimates (Self-Hosted)

| Component | Cost |
|-----------|------|
| faster-whisper | $0 (self-hosted, GPU recommended) |
| Chatterbox TTS | $0 (self-hosted, GPU recommended) |
| Gemini Flash (eval) | ~$0.0001/review (500 tokens) |
| **Total per review** | **~$0.0001** |

100 reviews/day = **~$0.30/month** for LLM alone

## Out of Scope (MVP)
- Multi-user / authentication
- Analytics dashboard
- Anki import
- Native mobile apps
- Cloze deletion cards
- Phone number access (future: Telnyx integration)

## Migration Path to Native Audio

When ready to upgrade:
1. Add Gemini Live API client to voice service
2. Feature flag: `USE_NATIVE_AUDIO=true`
3. When enabled, bypass faster-whisper and Chatterbox
4. Audio streams directly to/from Gemini
5. Cost increases to ~$0.01/min but latency drops to ~280ms
