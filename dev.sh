#!/bin/bash

# ENGRAM Development Script
# Starts backend, voice service, and frontend in a tmux session

SESSION_NAME="engram"
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Check if tmux is installed
if ! command -v tmux &> /dev/null; then
    echo "tmux is not installed. Please install it first."
    exit 1
fi

# Kill existing session if it exists
tmux kill-session -t "$SESSION_NAME" 2>/dev/null

# Create new tmux session with backend pane
tmux new-session -d -s "$SESSION_NAME" -n "dev" -c "$PROJECT_DIR"

# Split window horizontally (top: backend, bottom: voice+frontend)
tmux split-window -v -t "$SESSION_NAME:dev" -c "$PROJECT_DIR"

# Split bottom pane vertically (left: voice, right: frontend)
tmux split-window -h -t "$SESSION_NAME:dev.1" -c "$PROJECT_DIR"

# Pane layout:
# ┌─────────────────────────┐
# │      0: Backend         │
# ├────────────┬────────────┤
# │  1: Voice  │ 2: Frontend│
# └────────────┴────────────┘

# Select pane 0 and run backend
tmux select-pane -t "$SESSION_NAME:dev.0"
tmux send-keys -t "$SESSION_NAME:dev.0" "cd $PROJECT_DIR && echo '=== ENGRAM Backend (Rust) ===' && cargo run -p engram-backend" Enter

# Select pane 1 and run voice service
tmux select-pane -t "$SESSION_NAME:dev.1"
tmux send-keys -t "$SESSION_NAME:dev.1" "cd $PROJECT_DIR/apps/voice && echo '=== ENGRAM Voice Service (Python) ===' && python3 -m src.main" Enter

# Select pane 2 and run frontend
tmux select-pane -t "$SESSION_NAME:dev.2"
tmux send-keys -t "$SESSION_NAME:dev.2" "cd $PROJECT_DIR/apps/frontend && echo '=== ENGRAM Frontend (SvelteKit) ===' && pnpm dev" Enter

# Select the backend pane by default
tmux select-pane -t "$SESSION_NAME:dev.0"

# Attach to the session
tmux attach-session -t "$SESSION_NAME"
