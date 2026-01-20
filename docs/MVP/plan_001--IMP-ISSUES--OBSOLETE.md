# ENGRAM Codebase Audit: Deviations from plan_001.md

## Executive Summary

The codebase has significant deviations from the implementation plan, most critically:

1. **No Python Voice Service** - The entire `apps/voice/` directory is missing. Voice functionality was integrated into the Rust backend as a WebSocket handler, but without actual speech processing.
2. **SM-2 instead of FSRS** - The spaced repetition algorithm is SM-2, not the planned FSRS.
3. **Directory naming differs** - `apps/api/` → `apps/backend/`, `apps/web/` → `apps/frontend/`
4. **Voice is text-only** - Audio recording UI exists but audio is never processed. The "voice flashcard" app is currently text-only with LLM grading.

---

## Critical Issues (Blockers)

### 1. Missing Python Voice Service (CRITICAL)

**Expected:**
```
apps/voice/
├── pyproject.toml
├── requirements.txt
└── src/
    ├── main.py           # FastAPI + WebSocket
    ├── pipeline.py       # Pipecat: Silero VAD → faster-whisper → Chatterbox
    ├── session.py        # State machine
    ├── api_client.py     # REST calls to Rust API
    └── prompts.py        # LLM evaluation prompts
```

**Actual:** Directory does not exist. Zero Python files.

**Impact:**
- No speech-to-text (faster-whisper not integrated)
- No text-to-speech (Chatterbox not integrated)
- No voice activity detection (Silero VAD not used)
- Users cannot speak answers - must type everything
- **The "Voice Flashcard" concept is non-functional**

**Current State:** Voice features stubbed in Rust WebSocket handler (`apps/backend/src/routes/ws.rs:305-310`):
```rust
ClientMessage::EndAudio => {
    // Placeholder for Gemini Live API integration
    audio_buffer.clear();
    // No actual speech processing occurs
}
```

---

### 2. Wrong Spaced Repetition Algorithm (HIGH)

**Expected:** FSRS (Free Spaced Repetition Scheduling)
```rust
struct FSRSState {
    stability: f32,      // Days until 90% retention
    difficulty: f32,     // 1-10 scale
    retrievability: f32, // Current recall probability
}
```

**Actual:** SM-2 (SuperMemo 2) at `apps/backend/src/services/spaced_repetition.rs`
```rust
pub ease_factor: f64,    // Default 2.5
pub interval: i32,       // Days between reviews
pub repetitions: i32,    // Consecutive correct responses
```

**Impact:** FSRS is 15-20% more efficient than SM-2 according to the plan. Users will need more reviews for the same retention.

**Note:** SM-2 implementation is well-tested (6 unit tests) and functional.

---

## High Priority Issues

### 3. Missing Files (Rust Backend)

| Expected | Status | Impact |
|----------|--------|--------|
| `src/config.rs` | Missing | Config scattered in main.rs |
| `src/error.rs` | Missing | Using anyhow, inconsistent error types |
| `src/services/pdf_processor.rs` | Missing | Cannot ingest PDF documents |
| `src/db/migrations/` | Inline | Migrations hardcoded in db/mod.rs |

### 4. Incomplete Audio Pipeline (Frontend)

**Location:** `apps/frontend/src/lib/components/VoiceControls.svelte`

**Issues:**
- Uses deprecated `createScriptProcessor()` API
- Comment: "Note: This is a placeholder - actual worklet implementation needed"
- Audio is recorded but never actually sent to backend for processing
- No TTS playback for card fronts

### 5. No Service Worker (PWA Incomplete)

**Expected:** Offline-capable PWA with service worker
**Actual:** No service worker implementation

**Missing:**
- `static/service-worker.ts`
- Service worker registration in `app.html`
- Offline fallback page
- Cache-first strategy for assets

**Impact:** App will not work offline despite being designed as PWA.

### 6. Missing PWA Assets

**Referenced but not present:**
- `static/favicon.png`
- `static/icon-192.png`
- `static/icon-512.png`

---

## Medium Priority Issues

### 7. Primitive HTML Parser

**Location:** `apps/backend/src/routes/ingest.rs:267-297`

Simple character-by-character tag stripping that won't handle:
- CDATA sections
- Nested structures
- JavaScript content

**Code smell:** Variables `in_script`, `in_style` declared but never used.

**Recommendation:** Use `html2text` or `scraper` crate.

### 8. WebSocket Session State Not Persisted

**Location:** `apps/backend/src/routes/ws.rs:77`

`cards_reviewed` counter is in-memory only. Review session data lost if connection drops.

### 9. No Input Validation on Ingest

**Location:** `apps/backend/src/routes/ingest.rs:46-112`

No payload size limits. Large content could cause memory issues.

### 10. Error Handling Inconsistencies (Frontend)

- Generic error handling in stores
- WebSocket silently fails after 5 reconnection attempts
- No user-visible retry mechanisms
- `alert()` used for success messages

---

## Code Smells

### Backend
- No global error type hierarchy
- LLM provider priority (Gemini > OpenAI > Anthropic) undocumented
- Database migrations inline instead of separate .sql files

### Frontend
- Multiple independent stores with overlapping concerns
- Settings in localStorage vs cards in IndexedDB (inconsistent)
- No pagination for card listing (loads all into memory)
- Hardcoded colors in ReviewCard instead of CSS variables

---

## Positive Findings

| Feature | Status | Notes |
|---------|--------|-------|
| Card CRUD endpoints | 100% | All endpoints implemented |
| LLM evaluation prompt | 100% | Well-structured in Rust |
| Multi-provider LLM support | Bonus | Gemini, OpenAI, Anthropic |
| WebSocket session state machine | 90% | Renamed states but functional |
| Dexie IndexedDB setup | 100% | Excellent sync patterns |
| All 6 frontend routes | 100% | Dashboard, Review, Cards, Ingest, Settings + Layout |
| REST API client | 100% | Clean implementation |
| Theme system | 100% | Light/dark/system with persistence |
| SM-2 algorithm tests | 100% | 6 comprehensive unit tests |

---

## Summary by Component

| Component | Alignment | Critical Gap |
|-----------|-----------|--------------|
| Rust Backend | 75% | Missing FSRS, PDF processor, error.rs |
| Python Voice | 0% | **Entire service missing** |
| SvelteKit PWA | 73% | No service worker, deprecated audio APIs |
| Shared Types | 90% | Present and functional |

---

## Recommendations (Priority Order)

1. **Implement Python Voice Service** - This is the core differentiator
   - Add Pipecat with Silero VAD → faster-whisper → Chatterbox pipeline
   - Or implement Gemini Live API integration as planned Phase 6

2. **Add FSRS Algorithm** - Replace SM-2 for better retention efficiency

3. **Implement Service Worker** - Required for offline PWA functionality

4. **Add PDF Processor** - Required for content ingestion per plan

5. **Modernize Audio APIs** - Replace ScriptProcessor with AudioWorklet

6. **Add Missing Files** - config.rs, error.rs, migration files, PWA icons
