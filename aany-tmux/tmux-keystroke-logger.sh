#!/bin/bash
# Enhanced tmux keystroke and interaction logger

set -e

# Configuration
SESSION_NAME="${1:-agent}"
AGENT_ID="${2:-unknown}"
HUB_URL="${AANY_HUB_URL:-localhost:50052}"
# Use flat structure - logs under agent directory
AGENT_DIR="${HOME}/.aany/agents/${AGENT_ID}"
LOG_DIR="${AANY_LOG_DIR:-${AGENT_DIR}/logs}"
LOG_FILE="${LOG_DIR}/tmux_keystrokes_$(date +%Y%m%d_%H%M%S).log"

# Create log directory
mkdir -p "$LOG_DIR"

# Write log header
cat > "$LOG_FILE" << EOF
=== TMUX KEYSTROKE AND INTERACTION LOG ===
Agent ID: $AGENT_ID
Session: $SESSION_NAME
Started: $(date '+%Y-%m-%d %H:%M:%S')
Hub URL: $HUB_URL
==========================================

EOF

# Check if logging is disabled
if [ "$AANY_LOGGING_DISABLED" = "true" ]; then
    echo "Logging is disabled (AANY_LOGGING_DISABLED=true)"
    exit 0
fi

echo "Starting enhanced keystroke logger for session: $SESSION_NAME (Agent: $AGENT_ID)"
echo "Log file: $LOG_FILE"

# Function to log with full metadata
log_keystroke() {
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local event_type="$1"
    local content="$2"
    
    {
        echo "[$timestamp] [$event_type]"
        echo "  Agent ID: $AGENT_ID"
        echo "  Session: $SESSION_NAME"
        echo "  Content: $content"
        echo "  ---"
    } >> "$LOG_FILE"
}

# Function to capture pane content with full history
capture_full_pane() {
    tmux capture-pane -t "$SESSION_NAME" -p -S - 2>/dev/null || echo ""
}

# Function to monitor for keystrokes using tmux's built-in features
monitor_keystrokes() {
    # Enable key logging in tmux
    tmux set-option -t "$SESSION_NAME" monitor-activity on 2>/dev/null || true
    
    # Monitor pane for any changes
    local last_content=""
    local last_cursor_pos=""
    
    while true; do
        if ! tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
            log_keystroke "session_end" "Session terminated"
            break
        fi
        
        # Get current pane content
        local current_content=$(capture_full_pane)
        
        # Get cursor position and pane info
        local pane_info=$(tmux display-message -t "$SESSION_NAME" -p '#{cursor_x},#{cursor_y},#{pane_current_command}' 2>/dev/null || echo "0,0,unknown")
        
        # Detect changes
        if [ "$current_content" != "$last_content" ]; then
            # Extract the difference to identify keystrokes
            local new_chars=""
            
            # Simple diff to find new characters (this is a heuristic)
            if [ -n "$last_content" ]; then
                # Find the common prefix
                local i=0
                while [ $i -lt ${#last_content} ] && [ $i -lt ${#current_content} ] && [ "${last_content:$i:1}" = "${current_content:$i:1}" ]; do
                    ((i++))
                done
                
                # Extract new content after the common prefix
                if [ $i -lt ${#current_content} ]; then
                    new_chars="${current_content:$i}"
                    # Trim to just the new input (remove trailing content)
                    new_chars=$(echo "$new_chars" | head -n 1)
                    
                    # Log keystroke event
                    if [ -n "$new_chars" ]; then
                        log_keystroke "user_input" "$new_chars"
                    fi
                fi
            fi
            
            # Always log the full screen update
            log_keystroke "screen_update" "$(echo "$current_content" | tail -20)"
            
            last_content="$current_content"
        fi
        
        # Check cursor position changes (indicates typing)
        if [ "$pane_info" != "$last_cursor_pos" ]; then
            last_cursor_pos="$pane_info"
            log_keystroke "cursor_moved" "Position: $pane_info"
        fi
        
        # Brief sleep to avoid excessive CPU usage
        sleep 0.1
    done
}

# Start monitoring
log_keystroke "session_start" "Started monitoring session $SESSION_NAME"

# Main monitoring loop
monitor_keystrokes

# Final log entry
echo "" >> "$LOG_FILE"
echo "=== SESSION ENDED ===" >> "$LOG_FILE"
echo "Ended: $(date '+%Y-%m-%d %H:%M:%S')" >> "$LOG_FILE"

echo "Keystroke logger stopped. Log saved to: $LOG_FILE"