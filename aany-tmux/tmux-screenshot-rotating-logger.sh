#!/bin/bash
# Rotating tmux screenshot logger - creates new file every 5 minutes, captures every 5 seconds

set -e

# Configuration
SESSION_NAME="${1:-agent}"
AGENT_ID="${2:-unknown}"
CAPTURE_INTERVAL="${3:-5}"     # Capture interval in seconds (default 5)
FILE_DURATION="${4:-300}"       # Duration per file in seconds (default 300 = 5 minutes)

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROCESS_MANAGER="${SCRIPT_DIR}/process-manager.sh"

# Function to trim trailing newlines from a string
trim_trailing_newlines() {
    local content="$1"
    # Remove all trailing newlines
    echo -n "$content" | sed -e :a -e '/^\n*$/{$d;N;ba' -e '}'
}

# Function to run the rotating logger
run_rotating_logger() {
    local session="$1"
    local agent="$2"
    local interval="$3"
    local file_duration="$4"
    
    # Setup
    AGENT_DIR="${HOME}/.aany/agents/${agent}"
    SCREENSHOT_DIR="${AGENT_DIR}/screenshots"
    mkdir -p "$SCREENSHOT_DIR"
    
    echo "Starting rotating screenshot logger"
    echo "Session: $session (Agent: $agent)"
    echo "Capture interval: ${interval}s, New file every: ${file_duration}s"
    
    while true; do
        # Check if session still exists
        if ! tmux has-session -t "$session" 2>/dev/null; then
            echo "Session $session no longer exists. Stopping logger."
            break
        fi
        
        # Create new file for this rotation
        local file_timestamp=$(date +%Y%m%d_%H%M%S)
        local output_file="${SCREENSHOT_DIR}/session_log_${file_timestamp}.txt"
        local file_end_time=$(($(date +%s) + file_duration))
        
        echo "[$(date)] Starting new log file: $output_file"
        echo "=== Screenshot Log Started at $(date) ===" > "$output_file"
        echo "Session: $session | Agent: $agent" >> "$output_file"
        echo "=========================================" >> "$output_file"
        
        # Capture loop for this file
        while [ $(date +%s) -lt $file_end_time ]; do
            # Check if session still exists
            if ! tmux has-session -t "$session" 2>/dev/null; then
                echo "Session ended during capture"
                break 2
            fi
            
            # Capture pane content
            local raw_content=$(tmux capture-pane -t "$session" -e -p 2>/dev/null || echo "")
            
            # Trim trailing newlines
            local trimmed_content=$(trim_trailing_newlines "$raw_content")
            
            # Only add content if non-empty after trimming
            if [ -n "$trimmed_content" ]; then
                echo -e "\n========== $(date '+%Y-%m-%d %H:%M:%S') ==========" >> "$output_file"
                echo "$trimmed_content" >> "$output_file"
                
                local line_count=$(echo "$trimmed_content" | wc -l | tr -d ' ')
                echo "[$(date)] Captured $line_count lines (trimmed)"
            else
                echo "[$(date)] Skipped capture - empty content"
            fi
            
            # Wait for next capture
            sleep $interval
        done
        
        # End current file
        echo -e "\n=== Log File Ended at $(date) ===" >> "$output_file"
        echo "[$(date)] Completed log file: $output_file"
    done
    
    echo "Rotating logger stopped."
}

# Export function for background execution
export -f run_rotating_logger
export -f trim_trailing_newlines

# Check if we should run in foreground or background
if [ "${5:-background}" == "foreground" ]; then
    # Run directly
    run_rotating_logger "$SESSION_NAME" "$AGENT_ID" "$CAPTURE_INTERVAL" "$FILE_DURATION"
else
    # Use process manager to start in background
    COMMAND="bash -c 'run_rotating_logger \"$SESSION_NAME\" \"$AGENT_ID\" \"$CAPTURE_INTERVAL\" \"$FILE_DURATION\"'"
    
    # Start managed process
    "$PROCESS_MANAGER" start "screenshot-rotating" "$COMMAND" "$AGENT_ID" "$SESSION_NAME"
    
    echo "Rotating logger started in background."
    echo "Files will be created every ${FILE_DURATION}s with captures every ${CAPTURE_INTERVAL}s"
    echo "Use 'process-manager.sh list' to view status"
    echo "To stop: process-manager.sh stop screenshot-rotating"
fi