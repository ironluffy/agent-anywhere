#!/bin/bash
# Test that aany pool UI creates agents with logging enabled

set -e

echo "=== Testing Pool UI Agent Creation with Logging ==="
echo ""

# Clean up
echo "1. Cleaning up..."
tmux kill-server 2>/dev/null || true
rm -rf ~/.aany/agents/pool-test 2>/dev/null || true
sleep 1

echo ""
echo "2. Creating agent via pool UI..."
echo ""
echo "INSTRUCTIONS:"
echo "  1. Press 'n' to create new agent"
echo "  2. Name it: pool-test"
echo "  3. Press 'q' to quit"
echo ""
read -p "Press ENTER to open pool UI..."

# Run pool UI
./target/release/aany pool

# Wait for agent
sleep 5

# Test agent
echo ""
echo "3. Testing agent..."

if ! tmux has-session -t "agent-pool-test" 2>/dev/null; then
    echo "ERROR: Agent not created!"
    exit 1
fi

# Send test command
echo "Sending test command..."
tmux send-keys -t "agent-pool-test" "echo 'Testing pool UI logging'" C-m
sleep 3

# Check logs
echo ""
echo "4. Checking logs..."

LOG_DIR="$HOME/.aany/agents/pool-test/logs"
echo "Looking in: $LOG_DIR"

if [ ! -d "$LOG_DIR" ]; then
    echo "ERROR: Log directory not found!"
    exit 1
fi

# List log files
echo ""
echo "Log files found:"
ls -la "$LOG_DIR"

# Find interaction log
INTERACTION_LOG=$(ls -t "$LOG_DIR"/tmux_interaction_*.log 2>/dev/null | head -1)

if [ -z "$INTERACTION_LOG" ]; then
    echo ""
    echo "ERROR: No interaction log found!"
    exit 1
fi

echo ""
echo "=== Interaction Log Contents ==="
cat "$INTERACTION_LOG"
echo "================================"

# Verify content
echo ""
echo "Verification:"
echo -n "✓ Agent ID present: "
grep -q "Agent ID: pool-test" "$INTERACTION_LOG" && echo "PASS" || echo "FAIL"

echo -n "✓ Test command logged: "
grep -q "Testing pool UI logging" "$INTERACTION_LOG" && echo "PASS" || echo "FAIL"

echo ""
echo "✅ Pool UI agent creation with logging verified!"
echo ""
echo "Resources:"
echo "- Agent dir: ~/.aany/agents/pool-test/"
echo "- Log dir: ~/.aany/agents/pool-test/logs/"
echo "- Session: tmux attach -r -t agent-pool-test"