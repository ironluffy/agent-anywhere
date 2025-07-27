#!/bin/bash
# Append-mode tmux screenshot logger - appends to single file for specified duration

set -e

# Configuration
SESSION_NAME="${1:-agent}"
AGENT_ID="${2:-unknown}"
DURATION="${3:-180}" # Total duration in seconds (default 3 minutes)
INTERVAL="${4:-5}"   # Capture interval in seconds (default 5)

# Setup
AGENT_DIR="${HOME}/.aany/agents/${AGENT_ID}"
SCREENSHOT_DIR="${AGENT_DIR}/screenshots"
mkdir -p "$SCREENSHOT_DIR"

# Create output file with timestamp
OUTPUT_FILE="${SCREENSHOT_DIR}/session_log_$(date +%Y%m%d_%H%M%S).txt"

echo "Starting append-mode screenshot logger"
echo "Session: $SESSION_NAME (Agent: $AGENT_ID)"
echo "Output file: $OUTPUT_FILE"
echo "Duration: ${DURATION}s, Interval: ${INTERVAL}s"
echo "----------------------------------------" > "$OUTPUT_FILE"

# Calculate end time
END_TIME=$(($(date +%s) + DURATION))

# Main loop
while [ $(date +%s) -lt $END_TIME ]; do
    # Check if session exists
    if ! tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        echo "Session $SESSION_NAME no longer exists. Stopping."
        break
    fi
    
    # Add timestamp separator
    echo -e "\n========== $(date '+%Y-%m-%d %H:%M:%S') ==========" >> "$OUTPUT_FILE"
    
    # Capture and append pane content
    tmux capture-pane -t "$SESSION_NAME" -e -p >> "$OUTPUT_FILE" 2>/dev/null || true
    
    echo "[$(date)] Captured screenshot ($(wc -l < "$OUTPUT_FILE") total lines)"
    
    # Wait for next capture
    sleep $INTERVAL
done

echo -e "\n========== END OF LOG ==========" >> "$OUTPUT_FILE"
echo "Screenshot logging completed. Output saved to: $OUTPUT_FILE"