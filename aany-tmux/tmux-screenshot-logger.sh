#!/bin/bash
# Tmux pane screenshot logger - captures visual snapshots of tmux sessions

set -e

# Configuration
SESSION_NAME="${1:-agent}"
AGENT_ID="${2:-unknown}"
INTERVAL="${3:-30}" # Screenshot interval in seconds (default 30)

# Check if logging is disabled
if [ "$AANY_LOGGING_DISABLED" = "true" ]; then
    echo "Screenshot logging is disabled (AANY_LOGGING_DISABLED=true)"
    exit 0
fi

# Setup directories
AGENT_DIR="${HOME}/.aany/agents/${AGENT_ID}"
SCREENSHOT_DIR="${AGENT_DIR}/screenshots"

# Create screenshot directory
mkdir -p "$SCREENSHOT_DIR"

echo "Starting screenshot logger for session: $SESSION_NAME (Agent: $AGENT_ID)"
echo "Screenshot directory: $SCREENSHOT_DIR"
echo "Capture interval: ${INTERVAL}s"

# Function to capture tmux pane as text-based screenshot
capture_pane_screenshot() {
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local screenshot_file="${SCREENSHOT_DIR}/pane_${timestamp}.txt"
    local metadata_file="${SCREENSHOT_DIR}/pane_${timestamp}.json"
    
    # Check if session exists
    if ! tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        echo "Session $SESSION_NAME no longer exists. Stopping screenshot logger."
        return 1
    fi
    
    # Capture pane content with ANSI colors and formatting
    tmux capture-pane -t "$SESSION_NAME" -e -p > "$screenshot_file" 2>/dev/null || true
    
    # Get pane information
    local pane_info=$(tmux display-message -t "$SESSION_NAME" -p \
        '{"width": #{pane_width}, "height": #{pane_height}, "cursor_x": #{cursor_x}, "cursor_y": #{cursor_y}, "command": "#{pane_current_command}"}' \
        2>/dev/null || echo '{}')
    
    # Create metadata
    cat > "$metadata_file" << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "agent_id": "$AGENT_ID",
  "session": "$SESSION_NAME",
  "pane": $pane_info,
  "file": "pane_${timestamp}.txt"
}
EOF
    
    echo "[$(date)] Screenshot captured: pane_${timestamp}.txt"
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
        img { max-width: 100%; border: 1px solid #444; }
    </style>
</head>
<body>
    <h1>Tmux Screenshot Log</h1>
    <p>Agent: AGENT_ID | Session: SESSION_NAME</p>
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
echo "Starting screenshot capture loop..."
while true; do
    # Capture text-based screenshot
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