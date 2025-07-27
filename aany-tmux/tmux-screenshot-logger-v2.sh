#!/bin/bash
# Enhanced tmux pane screenshot logger with gap-free capture
# Tracks last capture position to ensure no missing content or overlaps

set -e

# Configuration
SESSION_NAME="${1:-agent}"
AGENT_ID="${2:-unknown}"
INTERVAL="${3:-5}" # Screenshot interval in seconds (default 5)

# Check if logging is disabled
if [ "$AANY_LOGGING_DISABLED" = "true" ]; then
    echo "Screenshot logging is disabled (AANY_LOGGING_DISABLED=true)"
    exit 0
fi

# Setup directories
AGENT_DIR="${HOME}/.aany/agents/${AGENT_ID}"
SCREENSHOT_DIR="${AGENT_DIR}/screenshots"
STATE_FILE="${SCREENSHOT_DIR}/.capture_state"

# Create screenshot directory
mkdir -p "$SCREENSHOT_DIR"

echo "Starting enhanced screenshot logger for session: $SESSION_NAME (Agent: $AGENT_ID)"
echo "Screenshot directory: $SCREENSHOT_DIR"
echo "Capture interval: ${INTERVAL}s"

# Initialize or load state
LAST_CAPTURE_END=0
if [ -f "$STATE_FILE" ]; then
    LAST_CAPTURE_END=$(cat "$STATE_FILE" 2>/dev/null || echo 0)
fi

# Function to get current pane information
get_pane_info() {
    tmux display-message -t "$SESSION_NAME" -p \
        '{"width": #{pane_width}, "height": #{pane_height}, "cursor_x": #{cursor_x}, "cursor_y": #{cursor_y}, "command": "#{pane_current_command}", "history_size": #{history_size}, "scroll_position": #{scroll_position}, "history_bytes": #{history_bytes}}' \
        2>/dev/null || echo '{}'
}

# Function to capture tmux pane with position tracking
capture_pane_screenshot() {
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local screenshot_file="${SCREENSHOT_DIR}/pane_${timestamp}.txt"
    local metadata_file="${SCREENSHOT_DIR}/pane_${timestamp}.json"
    
    # Check if session exists
    if ! tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        echo "Session $SESSION_NAME no longer exists. Stopping screenshot logger."
        return 1
    fi
    
    # Get current pane information including scroll position
    local pane_info=$(get_pane_info)
    local history_size=$(echo "$pane_info" | jq -r '.history_size // 0')
    local pane_height=$(echo "$pane_info" | jq -r '.height // 24')
    
    # Calculate capture range
    local capture_start=$LAST_CAPTURE_END
    local capture_end=$((history_size + pane_height))
    
    # If this is the first capture or we've scrolled back, capture everything visible
    if [ $capture_start -gt $capture_end ] || [ $capture_start -eq 0 ]; then
        # Capture entire visible buffer plus history
        tmux capture-pane -t "$SESSION_NAME" -e -p -S -$history_size > "$screenshot_file" 2>/dev/null || true
        capture_start=0
    else
        # Capture from last position to current end
        local lines_to_capture=$((capture_end - capture_start))
        if [ $lines_to_capture -gt 0 ]; then
            # Calculate start position relative to current view
            local relative_start=$((capture_start - history_size))
            tmux capture-pane -t "$SESSION_NAME" -e -p -S $relative_start -E $pane_height > "$screenshot_file" 2>/dev/null || true
        else
            # No new content
            echo "[$(date)] No new content to capture"
            return 0
        fi
    fi
    
    # Update state
    echo $capture_end > "$STATE_FILE"
    LAST_CAPTURE_END=$capture_end
    
    # Create enhanced metadata
    cat > "$metadata_file" << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "agent_id": "$AGENT_ID",
  "session": "$SESSION_NAME",
  "pane": $pane_info,
  "capture_range": {
    "start": $capture_start,
    "end": $capture_end,
    "lines_captured": $((capture_end - capture_start)),
    "method": $([ $capture_start -eq 0 ] && echo '"full"' || echo '"incremental"')
  },
  "file": "pane_${timestamp}.txt"
}
EOF
    
    echo "[$(date)] Screenshot captured: pane_${timestamp}.txt (lines $capture_start-$capture_end)"
}

# Function to clean old screenshots (keep last 100)
cleanup_old_screenshots() {
    local count=$(ls -1 "$SCREENSHOT_DIR"/pane_*.txt 2>/dev/null | wc -l)
    if [ $count -gt 100 ]; then
        # Remove oldest files
        ls -1t "$SCREENSHOT_DIR"/pane_*.txt | tail -n +101 | while read f; do
            rm -f "$f" "${f%.txt}.json"
        done
        echo "[$(date)] Cleaned up old screenshots (kept last 100)"
    fi
}

# Create index file
create_index() {
    local index_file="${SCREENSHOT_DIR}/index.html"
    cat > "$index_file" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Tmux Screenshot Log - Agent: AGENT_ID</title>
    <style>
        body { font-family: monospace; background: #1e1e1e; color: #ccc; }
        .screenshot { margin: 20px; padding: 10px; border: 1px solid #444; }
        .metadata { color: #888; font-size: 12px; }
        .content { background: #000; padding: 10px; white-space: pre; overflow-x: auto; }
        .capture-info { color: #4a9eff; font-size: 11px; margin-top: 5px; }
        img { max-width: 100%; border: 1px solid #444; }
    </style>
</head>
<body>
    <h1>Tmux Screenshot Log</h1>
    <p>Agent: AGENT_ID | Session: SESSION_NAME</p>
    <p class="capture-info">Gap-free incremental capture enabled</p>
    <div id="screenshots"></div>
    <script>
        // This would be populated by a script that reads the screenshot files
        document.getElementById('screenshots').innerHTML = 
            '<p>Open screenshot files directly to view captures</p>';
    </script>
</body>
</html>
EOF
    sed -i.bak "s/AGENT_ID/$AGENT_ID/g; s/SESSION_NAME/$SESSION_NAME/g" "$index_file" && rm -f "${index_file}.bak"
}

# Initialize
create_index

# Main loop
echo "Starting gap-free screenshot capture loop..."
while true; do
    # Capture screenshot with position tracking
    if ! capture_pane_screenshot; then
        break
    fi
    
    # Cleanup old files periodically
    if [ $(($(date +%s) % 600)) -lt $INTERVAL ]; then  # Every 10 minutes
        cleanup_old_screenshots
    fi
    
    # Wait for next capture
    sleep $INTERVAL
done

echo "Screenshot logger stopped."