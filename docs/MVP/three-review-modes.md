# Three Review Modes Implementation

## Overview

This document describes the implementation of three distinct flashcard review paths in ENGRAM. Each mode offers a different learning experience optimized for different use cases.

| Mode | Description | Rating Method |
|------|-------------|---------------|
| **Manual** | Text-based, click to reveal, click to rate | User clicks rating buttons |
| **Oral** | Voice answer, LLM evaluates, structured feedback | LLM auto-rates (no user input) |
| **Conversational** | Natural tutoring with Feynman-style teaching | LLM auto-rates (no user input) |

**Key Principle:** In Oral and Conversational modes, the human NEVER affects the rating. The LLM evaluates and rates automatically. The human only provides their answer.

**Rating Scale:** `0 = Again`, `1 = Hard`, `2 = Good`, `3 = Easy`

---

## Mode Details

### 1. Manual Mode (Default)

The existing text-based review flow. No changes required.

**Flow:**
1. Card question is displayed
2. User clicks "Show Answer" to reveal the back
3. User clicks a rating button (Again/Hard/Good/Easy)
4. Next card is presented

**Characteristics:**
- Traditional flashcard experience
- Full user control over rating
- No voice or audio components

### 2. Oral Mode

Efficient voice-based review with structured feedback.

**Flow:**
1. TTS speaks the card question
2. User records their voice answer
3. Audio is transcribed via STT
4. LLM evaluates the answer and assigns a rating (0-3)
5. TTS speaks structured feedback: *"Correct. The answer is X. Rating: Good."*
6. Card is automatically rated and advanced to next
7. Repeat until session complete

**Characteristics:**
- Structured, efficient feedback
- No personality or small talk
- Clear indication of correctness and rating
- Fastest voice-based option
- Rating IS announced to user

**UI Elements:**
- Progress bar showing cards reviewed
- Current question display
- Record button (tap to start/stop)
- Transcription display ("You said: ...")
- Evaluation result (correct/incorrect, rating badge)
- Expected answer shown on incorrect

### 3. Conversational Mode

Natural tutoring experience with a Feynman-style AI teacher.

**Flow:**
1. TTS speaks warm session intro: *"Hey there! Let's work through these cards together."*
2. TTS asks question with personality: *"Alright, here's an interesting one..."*
3. User speaks their answer naturally
4. LLM acts as a Feynman-style teacher:
   - Acknowledges answer naturally
   - Explains why correct/incorrect
   - May provide teaching moments
   - Uses intuitive analogies when helpful
   - Rating determined internally (NOT announced)
5. Natural transition to next card
6. Encouraging outro at session end: *"Nice work! You got through all of them."*

**Characteristics:**
- Warm, engaging personality
- Teaching moments - explains concepts
- Active listening cues ("I see...", "Interesting...")
- Rating happens silently in background
- Feels like conversation with a tutor
- Rating is NOT announced to user

**UI Elements:**
- Conversation bubble UI (chat-like)
- Tutor avatar with initials "FT" (Feynman Tutor)
- Progress dots showing cards completed
- Minimal controls - just mic button and end session
- Stats shown only at session end

---

## Technical Architecture

### Frontend Components

| File | Purpose |
|------|---------|
| `apps/frontend/src/lib/stores/settings.ts` | `ReviewMode` type and `reviewMode` setting with `setReviewMode()` method |
| `apps/frontend/src/routes/settings/+page.svelte` | Mode selector UI in Review Settings section |
| `apps/frontend/src/lib/api/websocket.ts` | `startSession(cardLimit, reviewMode)` passes mode to backend |
| `apps/frontend/src/lib/components/VoiceReview.svelte` | Oral mode component |
| `apps/frontend/src/lib/components/ConversationalReview.svelte` | Conversational mode component |
| `apps/frontend/src/routes/review/+page.svelte` | Routes to appropriate component based on mode |

### Voice Service (Python)

| File | Purpose |
|------|---------|
| `apps/voice/src/session.py` | `ReviewMode` enum, mode-specific evaluation and auto-advance logic |
| `apps/voice/src/prompts.py` | Mode-specific prompts for oral feedback and conversational teaching |
| `apps/voice/src/api_client.py` | `generate_text()` method for conversational text generation |
| `apps/voice/src/audio_components.py` | `AudioSegment` and `AudioAssembler` for modular audio (future pre-generation) |

### Backend (Rust)

| File | Purpose |
|------|---------|
| `apps/backend/src/routes/llm.rs` | `POST /api/llm/generate` endpoint for text generation |

### Shared Types

| File | Purpose |
|------|---------|
| `packages/shared/src/index.ts` | `ReviewMode` type, extended WebSocket message types |

---

## WebSocket Message Flow

### Start Session

**Client sends:**
```json
{
  "type": "start_session",
  "card_limit": 10,
  "review_mode": "oral"
}
```

Valid `review_mode` values: `"manual"`, `"oral"`, `"conversational"`

**Server responds:**
```json
{
  "type": "session_started",
  "total_cards": 10,
  "review_mode": "oral"
}
```

### Conversational Mode Only - Session Intro

**Server sends (conversational only):**
```json
{
  "type": "session_intro",
  "text": "Hey there! Let's work through these cards together.",
  "audio": "<base64 audio>",
  "sample_rate": 24000
}
```

### Card Presentation

**Server sends:**
```json
{
  "type": "card_presented",
  "card_id": "uuid",
  "front": "What is the capital of France?",
  "spoken_text": "Alright, here's a good one: What is the capital of France?",
  "audio": "<base64 audio>",
  "card_number": 1,
  "total_cards": 10
}
```

Note: In oral mode, `spoken_text` matches `front` verbatim. In conversational mode, `spoken_text` includes personality (e.g., "Ah, here's a good one...").

### Evaluation Response

**Server sends:**
```json
{
  "type": "evaluation",
  "rating": 2,
  "is_correct": true,
  "feedback": "Correct! The answer is Paris. Rating: Good.",
  "expected_answer": "Paris",
  "user_answer": "Paris",
  "audio": "<base64 audio>",
  "sample_rate": 24000,
  "auto_advance": true,
  "review_mode": "oral"
}
```

Note: `auto_advance: true` indicates oral/conversational mode where card is auto-rated.

### Auto-Rating (Oral/Conversational)

**Server sends:**
```json
{
  "type": "card_rated",
  "card_id": "uuid",
  "rating": 2,
  "auto_rated": true
}
```

### Session Complete

**Server sends:**
```json
{
  "type": "session_complete",
  "message": "Nice work! You got through all of them.",
  "audio": "<base64 audio>",
  "sample_rate": 24000,
  "stats": {
    "cards_reviewed": 10,
    "correct_count": 8,
    "incorrect_count": 2,
    "accuracy": 0.8
  }
}
```

---

## LLM Prompts

### Oral Mode Feedback

The oral feedback prompt requests structured output with rating announcement:

```
Evaluate this flashcard answer and provide structured feedback for spoken delivery.

Question: {question}
Expected Answer: {expected_answer}
Student's Answer: {user_answer}

Response format:
{
  "rating": 0-3,
  "is_correct": boolean,
  "spoken_feedback": "Correct. The answer is X. Rating: Good."
}
```

### Conversational Mode - Question Presentation

```
You are Richard Feynman teaching a student with flashcards. Present this question:
"{question}"

Be curious, engaging, maybe add why this is interesting. Keep it brief (1-2 sentences + question).

Output ONLY the text to speak, no JSON.
```

### Conversational Mode - Evaluation

```
You are Richard Feynman responding to a student's answer.

Question: {question}
Expected Answer: {expected_answer}
Student said: "{user_answer}"

Respond as Feynman would:
- Be genuinely curious about their thinking
- If wrong, explain WHY in an intuitive way
- Use simple analogies if helpful
- Be encouraging but honest
- Do NOT announce the rating

Response format:
{
  "rating": 0-3,
  "is_correct": boolean,
  "spoken_feedback": "...",
  "teaching_note": "..."
}
```

### Session Intro/Outro

```
# Intro
Generate a warm 1-2 sentence greeting to start a {total_cards} card review session.

# Outro
Session complete: {correct}/{total} correct ({accuracy}%).
Generate encouraging 1-2 sentence conclusion.
```

---

## Audio Architecture

The implementation includes groundwork for future pre-generated audio:

```
┌────────────────────────────────────────────────────────────┐
│                     Audio Components                       │
├────────────────────────────────────────────────────────────┤
│  1. Question Audio     - Card front spoken aloud           │
│  2. Transition Audio   - "Moving on...", "Next up...", etc │
│  3. Feedback Audio     - Response to user's answer         │
│  4. Session Audio      - Intro/outro for session           │
└────────────────────────────────────────────────────────────┘

Phase 1 (Current): All generated live via LLM + TTS
Phase 2 (Future):  Questions pre-generated at card creation
Phase 3 (Future):  Transitions pre-generated as audio bank
```

The `AudioSegment` dataclass and `AudioAssembler` class in `audio_components.py` provide the abstraction layer for this.

---

## Verification Checklist

### Settings
- [ ] Review mode can be changed in Settings page
- [ ] Selected mode persists across page refresh
- [ ] Default mode is "Manual"

### Manual Mode
- [ ] Existing click-based flow unchanged
- [ ] Rating buttons work as before
- [ ] VoiceControls component appears only if `voiceEnabled` setting is true (separate from reviewMode)

### Oral Mode
- [ ] Session starts with TTS speaking first question
- [ ] User can record voice answer
- [ ] Transcription displays after recording stops
- [ ] Evaluation shows correct/incorrect with rating badge
- [ ] Feedback includes rating announcement ("Rating: Good")
- [ ] Card auto-advances after feedback (no rating buttons)
- [ ] Progress bar updates correctly
- [ ] Session complete shows stats

### Conversational Mode
- [ ] Session starts with warm greeting (TTS)
- [ ] Questions include personality/intro phrases
- [ ] Conversation bubble UI displays messages
- [ ] Tutor responses are natural/teaching-focused
- [ ] Rating is NOT announced in feedback
- [ ] Auto-advances between cards
- [ ] Session complete shows encouraging outro
- [ ] Stats shown at end

### Edge Cases
- [ ] Empty card queue handled gracefully
- [ ] Network errors show appropriate messages
- [ ] Audio playback errors don't crash session
- [ ] LLM generation failures fall back gracefully

---

## Files Changed

### New Files
- `apps/frontend/src/lib/components/VoiceReview.svelte`
- `apps/frontend/src/lib/components/ConversationalReview.svelte`
- `apps/voice/src/audio_components.py`
- `apps/backend/src/routes/llm.rs`

### Modified Files
- `apps/frontend/src/lib/stores/settings.ts`
- `apps/frontend/src/routes/settings/+page.svelte`
- `apps/frontend/src/routes/review/+page.svelte`
- `apps/frontend/src/lib/api/websocket.ts`
- `apps/frontend/src/lib/components/VoiceControls.svelte` (type fixes)
- `apps/voice/src/session.py`
- `apps/voice/src/prompts.py`
- `apps/voice/src/api_client.py`
- `apps/backend/src/routes/mod.rs`
- `packages/shared/src/index.ts`
