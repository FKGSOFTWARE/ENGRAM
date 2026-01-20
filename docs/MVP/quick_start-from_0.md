# ENGRAM Quick Start Guide - From Zero to Running

**Time Required**: ~15 minutes
**Difficulty**: Beginner-friendly

---

## Prerequisites

Before starting, ensure you have installed:

| Requirement | Version | Check Command |
|-------------|---------|---------------|
| Node.js | 20+ | `node --version` |
| pnpm | 9+ | `pnpm --version` |
| Rust | 1.75+ | `rustc --version` |
| Python | 3.10+ | `python3 --version` (use `python3`, not `python`) |

**Optional but recommended**:
- CUDA toolkit (for GPU-accelerated Whisper STT)
- tmux (for the dev script)

---

## Step 1: Clone and Install Dependencies

```bash
# Clone the repository
cd /home/goose
git clone <your-repo-url> ENGRAM  # or use existing
cd ENGRAM

# Install frontend dependencies
pnpm install

# Install Python voice service dependencies
cd apps/voice
pip install pydantic-settings fastapi uvicorn httpx faster-whisper edge-tts
cd ../..
```

**Expected output**: Dependencies install without errors. You may see some warnings - these are normal.

---

## Step 2: Configure Environment

```bash
# Copy environment template
cp .env.example .env

# Edit with your API keys
nano .env  # or use your preferred editor
```

**Required**: At least ONE LLM provider API key:

```bash
# .env contents - uncomment and fill at least one:
GEMINI_API_KEY=your-gemini-key      # Recommended (cheapest)
# OPENAI_API_KEY=your-openai-key
# ANTHROPIC_API_KEY=your-anthropic-key

# Database (uses default if not set)
DATABASE_URL=sqlite:data/engram.db?mode=rwc
```

**Get API keys**:
- Gemini: https://makersuite.google.com/app/apikey
- OpenAI: https://platform.openai.com/api-keys
- Anthropic: https://console.anthropic.com/

---

## Step 3: Start the Services

### Option A: Using tmux (Recommended)

```bash
./dev.sh
```

This opens a tmux session with all three services in separate panes.

**tmux controls**:
- `Ctrl+B` then `n` - Next pane
- `Ctrl+B` then `p` - Previous pane
- `Ctrl+B` then `d` - Detach (services keep running)
- `tmux attach` - Reattach to session

### Option B: Manual Start (3 terminals)

**Terminal 1 - Backend**:
```bash
cd /home/goose/ENGRAM
cargo run -p engram-backend
```
Expected: `Listening on 0.0.0.0:3001`

**Terminal 2 - Voice Service**:
```bash
cd /home/goose/ENGRAM/apps/voice
python3 -m src.main
```
Expected: `Uvicorn running on http://0.0.0.0:8001`

**Note**: First run downloads models (~150MB Whisper, ~20MB Silero VAD).
- First run: 2-5 minutes (downloading)
- Subsequent runs: ~40 seconds (loading from cache)
- Health check: `curl http://localhost:8001/health` → `{"status":"healthy","pipeline_ready":true}`

**Terminal 3 - Frontend**:
```bash
cd /home/goose/ENGRAM
pnpm dev:frontend
```
Expected: `Local: http://localhost:3000/`

### Option C: Docker

```bash
docker-compose up -d
docker-compose logs -f  # Watch logs
```

---

## Step 4: Access the Application

Open your browser to: **http://localhost:3000**

You should see the ENGRAM dashboard with:
- Navigation: Dashboard, Review, Cards, Import, Settings
- Empty card count (0 cards)
- "Start Review" button (disabled until you have cards)

---

## Step 5: Test Core Features

### Test 1: Create Your First Card

1. Navigate to **Cards** (sidebar)
2. Click **"Add Card"** button
3. Fill in:
   - **Front**: "What is the capital of France?"
   - **Back**: "Paris"
4. Click **Save**

**Expected**: Card appears in the list with "Due Now" status.

### Test 2: Review a Card (Text Mode)

1. Navigate to **Review** (sidebar)
2. You should see your card's front: "What is the capital of France?"
3. Type your answer in the text box: "Paris"
4. Click **Submit**

**Expected**:
- LLM evaluates your answer
- Shows feedback: "Correct!" with score
- Rating buttons appear (Again, Hard, Good, Easy)

5. Click **Good**

**Expected**: Card scheduled for tomorrow (or later based on FSRS).

### Test 3: Import Content via Text

1. Navigate to **Import** (sidebar)
2. Select **Text** tab
3. Paste this sample content:
   ```
   The mitochondria is the powerhouse of the cell. It produces ATP
   through cellular respiration. The process involves glycolysis,
   the Krebs cycle, and the electron transport chain.
   ```
4. Set **Maximum Cards**: 3
5. Click **Generate Cards**

**Expected**:
- Loading spinner while LLM generates cards
- Preview shows 3 generated Q&A cards
- Each card has a checkbox (default: selected)

6. Review the cards, uncheck any you don't want
7. Click **Add X Cards**

**Expected**: Success message, cards added to library.

### Test 4: Import from URL

1. Navigate to **Import** (sidebar)
2. Select **URL** tab
3. Enter a Wikipedia article URL (e.g., `https://en.wikipedia.org/wiki/Photosynthesis`)
4. Click **Generate Cards**

**Expected**: Cards generated from the page content.

### Test 5: Import from PDF

1. Navigate to **Import** (sidebar)
2. Select **PDF** tab
3. Click to select a PDF file (any educational PDF)
4. Click **Generate Cards**

**Expected**: PDF text extracted, cards generated.

### Test 6: Voice Review (Optional - Requires Microphone)

1. Navigate to **Settings** (sidebar)
2. Ensure **Voice Mode** is enabled
3. Navigate to **Review**
4. Click the **microphone icon** to enable voice

**Expected**:
- Browser asks for microphone permission
- You hear the card question (TTS)
- Speak your answer
- LLM evaluates your spoken response

**Note**: Voice requires the Python voice service running on port 8001.

---

## What to Expect

### First Run Behavior

| Event | What Happens |
|-------|--------------|
| First card review | FSRS initializes with default parameters |
| First LLM call | May take 2-5 seconds for cold start |
| First voice session | Downloads Whisper model (~150MB for "base") |
| Offline access | App works, reviews queue for sync |

### Performance Expectations

| Operation | Typical Time |
|-----------|--------------|
| Card CRUD | < 100ms |
| LLM evaluation | 1-3 seconds |
| Card generation (10 cards) | 5-15 seconds |
| PDF processing | 2-10 seconds (size dependent) |
| Voice transcription | 0.5-2 seconds |

### Cost Expectations

Using Gemini Flash (cheapest):
- ~$0.0001 per review evaluation
- ~$0.001 per card generation
- 100 reviews/day ≈ $0.30/month

---

## Troubleshooting

### "No LLM provider available"

**Cause**: No API key configured or invalid key.

**Fix**:
1. Check `.env` file has at least one API key
2. Restart the backend after editing `.env`
3. Verify key is valid at provider's dashboard

### "Failed to connect to voice service"

**Cause**: Python voice service not running.

**Fix**:
1. Start voice service: `cd apps/voice && python -m src.main`
2. Check port 8001 is not blocked
3. Verify `VITE_USE_PYTHON_VOICE=true` in frontend env

### "Database error"

**Cause**: SQLite file permission or path issue.

**Fix**:
1. Create data directory: `mkdir -p data`
2. Check write permissions: `chmod 755 data`
3. Delete and recreate: `rm data/engram.db` (loses data)

### Cards not showing as due

**Cause**: FSRS scheduled them for the future.

**Fix**:
1. Check card's `next_review` date in card list
2. For testing, create new cards (always due immediately)
3. Wait for scheduled time or manually edit card

### Voice not working in browser

**Cause**: Browser security restrictions.

**Fix**:
1. Use HTTPS or localhost (not IP address)
2. Grant microphone permission when prompted
3. Try Chrome/Edge (best AudioWorklet support)

---

## Development Tips

### Watch Logs

```bash
# Backend logs
RUST_LOG=debug cargo run -p engram-backend

# Voice service logs
DEBUG=true python -m src.main

# All logs in Docker
docker-compose logs -f
```

### Reset Database

```bash
rm data/engram.db
# Restart backend - migrations run automatically
```

### Test API Directly

```bash
# List cards
curl http://localhost:3001/api/cards

# Create card
curl -X POST http://localhost:3001/api/cards \
  -H "Content-Type: application/json" \
  -d '{"front": "Test Q", "back": "Test A"}'

# Get due cards
curl http://localhost:3001/api/review/next?limit=10
```

### Run Type Checks

```bash
# Rust
cargo check -p engram-backend

# Frontend
cd apps/frontend && pnpm exec svelte-check
```

---

## Next Steps

After completing the quick start:

1. **Add more content**: Import from your study materials
2. **Configure settings**: Adjust daily card limit, theme
3. **Review daily**: FSRS works best with consistent reviews
4. **Try voice mode**: Hands-free review while doing other tasks

---

## File Locations Reference

| What | Where |
|------|-------|
| Database | `data/engram.db` |
| Backend logs | stdout |
| Voice models | `apps/voice/models/` (downloaded on first use) |
| Frontend build | `apps/frontend/.svelte-kit/` |
| Config | `.env` (root) |

---

**Questions?** Check the main README.md or open an issue.
