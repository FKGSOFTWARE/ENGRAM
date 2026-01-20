# ENGRAM Codebase Audit v2 - Comprehensive Review

**Date**: 2026-01-20
**Auditor**: Claude Opus 4.5
**Scope**: Full codebase comparison against `plan_001.md`

---

## Executive Summary

The previous audit (`plan_001--IMP-ISSUES.md`) contained **significant inaccuracies**. After thorough re-examination:

| Previous Claim | Actual Status |
|----------------|---------------|
| "No Python Voice Service" | **FULLY IMPLEMENTED** - 9 Python files, 1,800+ lines |
| "SM-2 instead of FSRS" | **BOTH EXIST** - FSRS implemented but SM-2 wrapper used |
| "Missing config.rs, error.rs" | **ALL PRESENT** - Complete implementations |
| "Missing PDF processor" | **PRESENT** - 274 lines with tests |
| "Missing migrations/" | **PRESENT** - 4 SQL migration files |

**Overall Implementation Status: ~85-90% complete**

---

## 1. Directory Structure Deviation

### Expected (plan_001.md)
```
apps/api/    → Rust backend
apps/web/    → SvelteKit frontend
apps/voice/  → Python voice service
```

### Actual
```
apps/backend/   → Rust backend (RENAMED)
apps/frontend/  → SvelteKit frontend (RENAMED)
apps/voice/     → Python voice service (EXISTS)
```

**Impact**: Low - naming convention difference only.

---

## 2. Python Voice Service - COMPLETE

**Location**: `/home/goose/ENGRAM/apps/voice/`

### File Structure (All Expected Files Present)

| Expected | Actual | Lines | Status |
|----------|--------|-------|--------|
| `pyproject.toml` | ✓ | 54 | Complete with hatchling |
| `requirements.txt` | ✓ | 28 | All dependencies |
| `src/main.py` | ✓ | 173 | FastAPI + 2 WebSocket endpoints |
| `src/pipeline.py` | ✓ | 346 | Silero VAD + faster-whisper |
| `src/session.py` | ✓ | 445 | 10-state machine |
| `src/api_client.py` | ✓ | 190 | Async httpx REST client |
| `src/prompts.py` | ✓ | 176 | 4 LLM prompt builders |

### Bonus Files (Beyond Plan)
| File | Lines | Purpose |
|------|-------|---------|
| `src/config.py` | 53 | Pydantic settings management |
| `src/tts_chatterbox.py` | 164 | High-quality TTS wrapper |
| `src/tts_fallback.py` | 212 | edge-tts + pyttsx3 fallback |
| `src/__init__.py` | 8 | Module initialization |

### Voice Pipeline Implementation

```
Audio Input → Silero VAD → faster-whisper STT → LLM Eval → Chatterbox TTS → Audio Output
```

**State Machine States** (session.py):
- `IDLE` → `STARTING` → `PRESENTING_CARD` → `LISTENING` → `PROCESSING`
- `EVALUATING` → `PRESENTING_FEEDBACK` → `ENDING` → `ENDED` | `ERROR`

**WebSocket Endpoints**:
- `/ws/voice` - Command-based interaction
- `/ws/voice/stream` - Real-time streaming with VAD turn detection

---

## 3. Spaced Repetition Algorithm

### Actual State (Complex)

**File**: `/home/goose/ENGRAM/apps/backend/src/services/spaced_repetition.rs`

| Implementation | Status | Notes |
|----------------|--------|-------|
| FSRS Algorithm | ✓ Complete | 17-parameter FSRS-4.5 weights |
| SM-2 Wrapper | ✓ Complete | Converts SM-2 params to FSRS internally |
| **In Use** | SM-2 wrapper | Routes call `calculate_sm2()` |

### FSRS Implementation Details

```rust
pub struct FSRSState {
    pub stability: f64,      // Days until 90% retention
    pub difficulty: f64,     // 1-10 scale
    pub reps: i32,          // Consecutive correct
    pub lapses: i32,        // Times rated "Again"
    pub last_review: Option<DateTime<Utc>>,
}

pub fn calculate_fsrs(state: &FSRSState, rating: ReviewRating, params: Option<&FSRSParameters>)
    -> (i32, f64, f64, i32, i32, DateTime<Utc>)
```

### Database Schema Supports FSRS

```sql
-- From 002_create_cards.sql
CREATE TABLE cards (
    stability REAL DEFAULT 0.0,     -- FSRS field
    difficulty REAL DEFAULT 0.0,    -- FSRS field
    -- ...
);
```

### Issue: Card Struct Doesn't Expose FSRS Fields

```rust
// models/card.rs - Only SM-2 fields exposed
pub struct Card {
    pub ease_factor: f64,    // SM-2
    pub interval: i32,       // SM-2
    pub repetitions: i32,    // SM-2
    // Missing: stability, difficulty, lapses
}
```

**Recommendation**: Add FSRS fields to Card struct or create migration to use FSRS exclusively.

---

## 4. Rust Backend - Complete

### File Structure (All Expected Files Present)

```
apps/backend/src/
├── main.rs              ✓ 82 lines
├── config.rs            ✓ 161 lines (CLAIMED MISSING - EXISTS)
├── error.rs             ✓ 191 lines (CLAIMED MISSING - EXISTS)
├── db/
│   ├── mod.rs           ✓ Migration runner
│   └── migrations/      ✓ 4 SQL files (CLAIMED MISSING - EXISTS)
│       ├── 000_migrations_table.sql
│       ├── 001_create_sources.sql
│       ├── 002_create_cards.sql
│       └── 003_create_reviews.sql
├── models/
│   ├── card.rs          ✓ Card, CreateCard, UpdateCard
│   ├── review.rs        ✓ Review, ReviewRating enum
│   └── source.rs        ✓ Source, SourceType enum
├── routes/
│   ├── cards.rs         ✓ Full CRUD
│   ├── review.rs        ✓ /next, /submit, /evaluate
│   ├── ingest.rs        ✓ /text, /url, /pdf, /confirm
│   └── ws.rs            ✓ WebSocket voice handler
├── services/
│   ├── spaced_repetition.rs  ✓ FSRS + SM-2 (with 8 tests)
│   └── pdf_processor.rs      ✓ 274 lines (CLAIMED MISSING - EXISTS)
└── llm/
    ├── provider.rs      ✓ Trait + types
    ├── manager.rs       ✓ Fallback chain
    ├── gemini.rs        ✓ Google implementation
    ├── openai.rs        ✓ OpenAI implementation
    └── anthropic.rs     ✓ Anthropic implementation
```

### API Endpoints (All Implemented)

| Method | Endpoint | Handler | Status |
|--------|----------|---------|--------|
| GET | `/api/cards` | list_cards | ✓ |
| POST | `/api/cards` | create_card | ✓ |
| GET | `/api/cards/:id` | get_card | ✓ |
| PATCH | `/api/cards/:id` | update_card | ✓ |
| DELETE | `/api/cards/:id` | delete_card | ✓ |
| GET | `/api/review/next` | get_next_cards | ✓ |
| POST | `/api/review/submit` | submit_review | ✓ |
| POST | `/api/review/evaluate` | evaluate_answer | ✓ |
| POST | `/api/ingest/text` | ingest_text | ✓ |
| POST | `/api/ingest/url` | ingest_url | ✓ |
| POST | `/api/ingest/pdf` | ingest_pdf | ✓ |
| POST | `/api/ingest/confirm` | confirm_cards | ✓ |
| WS | `/api/ws` | ws_handler | ✓ |

---

## 5. SvelteKit Frontend - Mostly Complete

### File Structure

```
apps/frontend/
├── src/
│   ├── app.html                  ✓ SW registration
│   ├── routes/
│   │   ├── +layout.svelte        ✓ Theme management
│   │   ├── +page.svelte          ✓ Dashboard
│   │   ├── review/+page.svelte   ✓ Voice review
│   │   ├── cards/+page.svelte    ✓ Card CRUD
│   │   ├── ingest/+page.svelte   ✓ Content import
│   │   └── settings/+page.svelte ✓ Preferences
│   └── lib/
│       ├── api/
│       │   ├── client.ts         ✓ REST client
│       │   └── websocket.ts      ✓ WS voice client
│       ├── components/
│       │   ├── VoiceControls.svelte  ✓ AudioWorklet + fallback
│       │   ├── ReviewCard.svelte     ✓ Rating UI
│       │   └── CardItem.svelte       ✓ List item
│       ├── db/
│       │   └── index.ts          ✓ Dexie IndexedDB
│       └── stores/
│           ├── cards.ts          ✓ Card management
│           ├── review.ts         ✓ Review session
│           └── settings.ts       ✓ User prefs
├── static/
│   ├── manifest.json             ✓ PWA manifest
│   ├── service-worker.js         ✓ Offline support
│   ├── audio-processor.worklet.js ✓ Modern audio API
│   ├── offline.html              ✓ Fallback page
│   ├── favicon.svg               ✓ Icon
│   ├── icon-192.svg              ✓ PWA icon
│   └── icon-512.svg              ✓ PWA icon
```

---

## 6. Issues Found

### CRITICAL

| ID | Location | Issue | Impact |
|----|----------|-------|--------|
| C1 | `service-worker.js:19-22` | Precache references `.png` files but icons are `.svg` | PWA won't cache icons offline |
| C2 | `audio-processor.worklet.js:126` | `sampleRate` is undefined | Runtime error breaks VAD |
| C3 | `settings.ts:30` | API key stored in localStorage | XSS vulnerability |

### HIGH

| ID | Location | Issue | Impact |
|----|----------|-------|--------|
| H1 | `spaced_repetition.rs` | FSRS implemented but not used | Suboptimal retention |
| H2 | `VoiceControls.svelte:114-145` | Falls back to deprecated `createScriptProcessor()` | Audio glitches, main thread blocking |
| H3 | `ws.rs:300-310` | Audio chunks collected but never processed | Voice transcription non-functional |
| H4 | `cards.ts:85-89` | Race condition in card ID sync | Potential data inconsistency |

### MEDIUM

| ID | Location | Issue | Impact |
|----|----------|-------|--------|
| M1 | `ingest.rs:496` | `in_script`, `in_style` declared but unused | Dead code |
| M2 | `ingest.rs:492-523` | Primitive HTML parser | Won't handle complex HTML |
| M3 | `ws.rs:77` | Session state not persisted | Progress lost on disconnect |
| M4 | `ingest.rs` | No content size validation for text | Memory exhaustion possible |
| M5 | `client.ts` | No request timeouts | Requests can hang indefinitely |
| M6 | `cards.ts:91` | Silent catch on sync failure | User unaware of offline state |
| M7 | Routes | Inconsistent error handling | Not using AppError consistently |

### LOW

| ID | Location | Issue | Impact |
|----|----------|-------|--------|
| L1 | `ingest/+page.svelte:129` | Uses `alert()` for success | Poor UX |
| L2 | `cards/+page.svelte:47` | Uses `window.confirm()` | Dated UX pattern |
| L3 | `ingest/+page.svelte:143-145` | Unused `$effect` block | Dead code |
| L4 | Frontend | No pagination for cards | Performance with large datasets |
| L5 | Frontend | CSS variable naming inconsistent | Maintainability |
| L6 | `db/index.ts` | Missing compound index on reviews | Query performance |
| L7 | LLM providers | `with_model()` method never used | Dead code (see diagnostics) |
| L8 | `error.rs:154` | `ok_or_not_found` method unused | Dead code |

---

## 7. Missing Infrastructure

| Item | Expected | Status | Notes |
|------|----------|--------|-------|
| `docker-compose.yml` | Yes | **MISSING** | Uses `dev.sh` + tmux instead |
| `README.md` | Yes | **MISSING** | No root documentation |
| `.env` | Template exists | `.env.example` only | Correct for VCS |

---

## 8. Code Quality Summary

### Backend (Rust)

| Aspect | Score | Notes |
|--------|-------|-------|
| Architecture | 9/10 | Clean separation of concerns |
| Error Handling | 7/10 | Types defined, inconsistently applied |
| Tests | 8/10 | Good coverage on SR algorithm |
| Documentation | 5/10 | Limited inline docs |

### Voice Service (Python)

| Aspect | Score | Notes |
|--------|-------|-------|
| Architecture | 9/10 | Pipeline pattern, async throughout |
| Error Handling | 8/10 | Comprehensive try/except |
| Configuration | 9/10 | Pydantic settings |
| Documentation | 6/10 | Docstrings present but sparse |

### Frontend (SvelteKit)

| Aspect | Score | Notes |
|--------|-------|-------|
| Architecture | 8/10 | Offline-first, stores pattern |
| Security | 3/10 | API key in localStorage |
| PWA | 5/10 | Broken precache paths |
| UX | 6/10 | alert/confirm modals |

---

## 9. Compiler Warnings (From IDE Diagnostics)

```
openai.rs:26      ⚠ method `with_model` never used
anthropic.rs:26   ⚠ method `with_model` never used
gemini.rs:26      ⚠ method `with_model` never used
mod.rs:75         ⚠ function `needs_migration` never used
pdf_processor.rs  ⚠ variant `InvalidFormat` never constructed
pdf_processor.rs  ⚠ fields `page_count`, `truncated` never read
error.rs:154      ⚠ method `ok_or_not_found` never used
ws.rs:49          ⚠ variant `AudioChunk` never constructed
```

---

## 10. Recommendations (Priority Order)

### Immediate (Critical)

1. **Fix service-worker.js precache paths**
   ```javascript
   // Change from:
   '/favicon.png', '/icon-192.png', '/icon-512.png'
   // To:
   '/favicon.svg', '/icon-192.svg', '/icon-512.svg'
   ```

2. **Fix audio-processor.worklet.js sampleRate**
   ```javascript
   // Add to constructor or options:
   this.sampleRate = sampleRate || 16000;
   ```

3. **Move API key from localStorage to sessionStorage or secure cookie**

### High Priority

4. **Switch from SM-2 wrapper to direct FSRS usage**
   - Update Card struct to include FSRS fields
   - Change routes to call `calculate_fsrs()` directly

5. **Connect voice pipeline to backend**
   - Frontend should connect to Python voice service, not Rust WS
   - Or integrate Python STT/TTS into Rust via subprocess/FFI

6. **Remove deprecated ScriptProcessor fallback**
   - Show browser incompatibility message instead

### Medium Priority

7. **Add docker-compose.yml** for consistent development
8. **Add README.md** with setup instructions
9. **Implement request timeouts** in API client
10. **Add pagination** for card listing
11. **Clean up dead code** (unused methods, variables)
12. **Standardize error handling** to use AppError throughout

---

## 11. Comparison with Previous Audit

| Previous Claim | Verdict |
|----------------|---------|
| "Python Voice Service 0%" | **WRONG** - 100% implemented |
| "Missing config.rs" | **WRONG** - Present (161 lines) |
| "Missing error.rs" | **WRONG** - Present (191 lines) |
| "Missing pdf_processor.rs" | **WRONG** - Present (274 lines) |
| "Migrations hardcoded" | **WRONG** - 4 separate SQL files |
| "SM-2 instead of FSRS" | **PARTIALLY CORRECT** - Both exist, SM-2 wrapper used |
| "No service worker" | **WRONG** - Present (230 lines) |
| "Uses ScriptProcessor" | **PARTIALLY CORRECT** - AudioWorklet primary, SP fallback |

---

## 12. Overall Assessment

**Project Status**: MVP ~85-90% complete

**Ready for Alpha Testing**: Yes, with critical fixes

**Blocking Issues**:
1. Service worker precache paths (breaks offline)
2. Audio worklet sampleRate bug (breaks VAD)
3. Voice service not connected to frontend (voice review non-functional end-to-end)

**Architecture Quality**: Solid foundation, well-structured monorepo

**Technical Debt**: Low to moderate, mostly dead code and minor inconsistencies
