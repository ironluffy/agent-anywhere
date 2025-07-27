#!/bin/bash
# Tmux interaction logger - captures all screen content and user input

set -e

# Configuration
SESSION_NAME="${1:-agent}"
AGENT_ID="${2:-unknown}"
HUB_URL="${AANY_HUB_URL:-localhost:50052}"
# Use flat structure - logs under agent directory
AGENT_DIR="${HOME}/.aany/agents/${AGENT_ID}"
LOG_DIR="${AANY_LOG_DIR:-${AGENT_DIR}/logs}"
LOG_FILE="${LOG_DIR}/tmux_interaction_$(date +%Y%m%d_%H%M%S).log"

# Create log directory
mkdir -p "$LOG_DIR"

# Write log header
cat > "$LOG_FILE" << EOF
=== TMUX INTERACTION LOG ===
Agent ID: $AGENT_ID
Session: $SESSION_NAME
Started: $(date '+%Y-%m-%d %H:%M:%S')
Hub URL: $HUB_URL
===========================

EOF

# Check if logging is disabled
if [ "$AANY_LOGGING_DISABLED" = "true" ]; then
    echo "Logging is disabled (AANY_LOGGING_DISABLED=true)"
    exit 0
fi

echo "Starting interaction logger for session: $SESSION_NAME (Agent: $AGENT_ID)"
echo "Log file: $LOG_FILE"

# Function to log interaction
log_interaction() {
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    local interaction_type="$1"
    local content="$2"
    
    echo "[$timestamp] [$interaction_type] $AGENT_ID: $content" >> "$LOG_FILE"
    echo "  Agent ID: $AGENT_ID" >> "$LOG_FILE"
    echo "  Session: $SESSION_NAME" >> "$LOG_FILE"
    echo "  Hub URL: $HUB_URL" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"
}

# Function to send log to hub
send_to_hub() {
    local interaction_type="$1"
    local content="$2"
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    if command -v grpcurl &> /dev/null && [ "$AANY_LOGGING_DISABLED" != "true" ]; then
        grpcurl -plaintext \
            -d "{
                \"logs\": [{
                    \"agent_id\": \"$AGENT_ID\",
                    \"timestamp\": \"$timestamp\",
                    \"level\": \"INFO\",
                    \"message\": \"$content\",
                    \"component\": \"interaction\",
                    \"metadata\": {
                        \"interaction_type\": \"$interaction_type\",
                        \"session_name\": \"$SESSION_NAME\",
                        \"agent_id\": \"$AGENT_ID\",
                        \"hub_url\": \"$HUB_URL\"
                    }
                }]
            }" \
            "$HUB_URL" aany.hub.v1.AgentHub/SendLogs 2>/dev/null || true
    fi
}

# Log session start
log_interaction "lifecycle" "Interaction logger started for session $SESSION_NAME"
send_to_hub "lifecycle" "Interaction logger started"

# Function to detect user input by analyzing content changes
detect_user_input() {
    local old_content="$1"
    local new_content="$2"
    
    # Find the common prefix length
    local common_len=0
    local min_len=$((${#old_content} < ${#new_content} ? ${#old_content} : ${#new_content}))
    
    while [ $common_len -lt $min_len ]; do
        if [ "${old_content:$common_len:1}" != "${new_content:$common_len:1}" ]; then
            break
        fi
        ((common_len++))
    done
    
    # Extract the new part
    if [ ${#new_content} -gt $common_len ]; then
        echo "${new_content:$common_len}"
    fi
}

# Main monitoring loop
LAST_CONTENT=""
LAST_LINE_COUNT=0
CURSOR_LINE=0

while true; do
    # Check if session still exists
    if ! tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
        log_interaction "lifecycle" "Session $SESSION_NAME ended"
        send_to_hub "lifecycle" "Session ended"
        echo "Session $SESSION_NAME no longer exists. Stopping logger."
        break
    fi
    
    # Capture current pane content with full history
    CURRENT_CONTENT=$(tmux capture-pane -t "$SESSION_NAME" -p -S - 2>/dev/null || echo "")
    
    # Get cursor position for better keystroke detection
    CURSOR_INFO=$(tmux display-message -t "$SESSION_NAME" -p '#{cursor_x},#{cursor_y}' 2>/dev/null || echo "0,0")
    
    # Check if content has changed
    if [ "$CURRENT_CONTENT" != "$LAST_CONTENT" ]; then
        if [ -n "$LAST_CONTENT" ]; then
            # Detect user input
            USER_INPUT=$(detect_user_input "$LAST_CONTENT" "$CURRENT_CONTENT")
            
            if [ -n "$USER_INPUT" ]; then
                # Check if it's actual typing (not just output)
                if [[ "$USER_INPUT" =~ ^[[:print:]]+$ ]] && [ ${#USER_INPUT} -lt 200 ]; then
                    log_interaction "user_typing" "$USER_INPUT"
                    send_to_hub "user_typing" "User typed: $USER_INPUT"
                fi
            fi
            
            # Log screen updates
            log_interaction "screen_update" "$(echo "$CURRENT_CONTENT" | tail -20)"
            send_to_hub "screen_update" "Screen updated"
        else
            # First capture - log everything
            log_interaction "screen_history" "$CURRENT_CONTENT"
            send_to_hub "screen_history" "Initial screen content captured"
        fi
        
        LAST_CONTENT="$CURRENT_CONTENT"
    fi
    
    # Sleep briefly for responsive monitoring
    sleep 0.2
done

# Log session end
echo "" >> "$LOG_FILE"
echo "=== SESSION ENDED ===" >> "$LOG_FILE"
echo "Ended: $(date '+%Y-%m-%d %H:%M:%S')" >> "$LOG_FILE"

echo "Interaction logger stopped. Log saved to: $LOG_FILE"