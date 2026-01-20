#!/bin/bash

# ENGRAM Kill Script
# Safely stops all services started by dev.sh

SESSION_NAME="engram"

# Ports used by ENGRAM services
PORTS=(3000 3001 8001)  # Frontend, Backend, Voice

kill_port() {
    local port=$1
    local pid
    pid=$(lsof -ti :"$port" 2>/dev/null)
    if [ -n "$pid" ]; then
        echo "Killing process on port $port (PID: $pid)"
        kill "$pid" 2>/dev/null
        sleep 0.5
        # Force kill if still running
        if kill -0 "$pid" 2>/dev/null; then
            kill -9 "$pid" 2>/dev/null
        fi
    fi
}

echo "Stopping ENGRAM services..."

# Check if the session exists and stop gracefully
if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    # Send Ctrl+C to each pane to allow graceful shutdown
    for pane in 0 1 2; do
        tmux send-keys -t "$SESSION_NAME:dev.$pane" C-c 2>/dev/null
    done

    # Wait briefly for graceful shutdown
    sleep 2

    # Kill the tmux session
    tmux kill-session -t "$SESSION_NAME" 2>/dev/null
    echo "Tmux session '$SESSION_NAME' terminated."
else
    echo "No '$SESSION_NAME' tmux session found."
fi

# Kill any remaining processes on the ports
echo "Checking for processes on ports: ${PORTS[*]}"
for port in "${PORTS[@]}"; do
    kill_port "$port"
done

echo "ENGRAM services stopped."
