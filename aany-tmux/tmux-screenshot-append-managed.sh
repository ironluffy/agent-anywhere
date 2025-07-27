#!/bin/bash
# Managed append-mode tmux screenshot logger

set -e

# Configuration
SESSION_NAME="${1:-agent}"
AGENT_ID="${2:-unknown}"
DURATION="${3:-180}" # Total duration in seconds (default 3 minutes)
INTERVAL="${4:-5}"   # Capture interval in seconds (default 5)

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROCESS_MANAGER="${SCRIPT_DIR}/process-manager.sh"

# Function to run the actual logger
run_logger() {
    local session="$1"
    local agent="$2"
    local duration="$3"
    local interval="$4"
    
    # Setup
    AGENT_DIR="${HOME}/.aany/agents/${agent}"
    SCREENSHOT_DIR="${AGENT_DIR}/screenshots"
    mkdir -p "$SCREENSHOT_DIR"
    
    # Create output file with timestamp
    OUTPUT_FILE="${SCREENSHOT_DIR}/session_log_$(date +%Y%m%d_%H%M%S).txt"
    
    echo "Starting append-mode screenshot logger"
    echo "Session: $session (Agent: $agent)"
    echo "Output file: $OUTPUT_FILE"
    echo "Duration: ${duration}s, Interval: ${interval}s"
    echo "----------------------------------------" > "$OUTPUT_FILE"
    
    # Calculate end time
    END_TIME=$(($(date +%s) + duration))
    
    # Main loop
    while [ $(date +%s) -lt $END_TIME ]; do
        # Check if session exists
        if ! tmux has-session -t "$session" 2>/dev/null; then
            echo "Session $session no longer exists. Stopping."
            break
        fi
        
        # Add timestamp separator
        echo -e "\n========== $(date '+%Y-%m-%d %H:%M:%S') ==========" >> "$OUTPUT_FILE"
        
        # Capture and append pane content
        tmux capture-pane -t "$session" -e -p >> "$OUTPUT_FILE" 2>/dev/null || true
        
        echo "[$(date)] Captured screenshot ($(wc -l < "$OUTPUT_FILE") total lines)"
        
        # Wait for next capture
        sleep $interval
    done
    
    echo -e "\n========== END OF LOG ==========" >> "$OUTPUT_FILE"
    echo "Screenshot logging completed. Output saved to: $OUTPUT_FILE"
}

# Export function for background execution
export -f run_logger

# Check if we should run in foreground or background
if [ "${5:-background}" == "foreground" ]; then
    # Run directly
    run_logger "$SESSION_NAME" "$AGENT_ID" "$DURATION" "$INTERVAL"
else
    # Use process manager to start in background
    COMMAND="bash -c 'run_logger \"$SESSION_NAME\" \"$AGENT_ID\" \"$DURATION\" \"$INTERVAL\"'"
    
    # Start managed process
    "$PROCESS_MANAGER" start "screenshot-append" "$COMMAND" "$AGENT_ID" "$SESSION_NAME"
    
    echo "Logger started in background. Use 'process-manager.sh list' to view status"
    echo "To stop: process-manager.sh stop screenshot-append"
fi