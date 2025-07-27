#!/bin/bash
# Create a tmux agent with full interaction logging enabled

set -e

# Parse arguments
AGENT_NAME="${1:-test-agent}"
HUB_URL="${AANY_HUB_URL:-localhost:50052}"

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Build the project first
echo "Building tmux-agent modules..."
cd "$SCRIPT_DIR"
cargo build --release --bin tmux-agent-modular 2>/dev/null || true

# Create agent config with interaction logging enabled
CONFIG_FILE="/tmp/agent-${AGENT_NAME}-config.json"
cat > "$CONFIG_FILE" << EOF
{
  "session_name": "agent-$AGENT_NAME",
  "window": 0,
  "pane": 0,
  "agent_id": "$AGENT_NAME",
  "modules_config": {
    "hub_connector": {
      "enabled": true,
      "hub_url": "$HUB_URL"
    },
    "interaction_logger": {
      "enabled": true,
      "hub_url": "$HUB_URL",
      "log_dir": "/tmp/tmux-agent-logs"
    }
  }
}
EOF

echo "Creating tmux agent with interaction logging..."
echo "Agent Name: $AGENT_NAME"
echo "Hub URL: $HUB_URL"
echo "Config: $CONFIG_FILE"

# Create tmux session
SESSION_NAME="agent-$AGENT_NAME"
tmux new-session -d -s "$SESSION_NAME"

# Send initial message
tmux send-keys -t "$SESSION_NAME" "clear" C-m
tmux send-keys -t "$SESSION_NAME" "echo '🤖 Agent $AGENT_NAME starting with interaction logging...'" C-m
tmux send-keys -t "$SESSION_NAME" "echo '📊 Logs will be sent to hub at $HUB_URL'" C-m
tmux send-keys -t "$SESSION_NAME" "echo '📝 Local logs saved to /tmp/tmux-agent-logs/'" C-m

# Start the modular agent with logging
if [ -f "$SCRIPT_DIR/target/release/tmux-agent-modular" ]; then
    tmux send-keys -t "$SESSION_NAME" "$SCRIPT_DIR/target/release/tmux-agent-modular --config $CONFIG_FILE" C-m
else
    echo "Warning: tmux-agent-modular not found, using basic logging"
    # Start basic interaction logger
    "$SCRIPT_DIR/tmux-interaction-logger.sh" "$SESSION_NAME" "$AGENT_NAME" &
fi

echo ""
echo "✅ Agent created with interaction logging!"
echo ""
echo "🔍 Monitor the agent:"
echo "   tmux attach -r -t $SESSION_NAME"
echo ""
echo "📊 View logs:"
echo "   Hub UI: http://localhost:8090"
echo "   Local: tail -f /tmp/tmux-agent-logs/tmux_interaction_${AGENT_NAME}_*.log"
echo ""
echo "🎯 Test the agent:"
echo "   tmux send-keys -t $SESSION_NAME 'echo Hello from agent' C-m"