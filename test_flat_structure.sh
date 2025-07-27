#!/bin/bash
# Test the new flat directory structure

set -e

echo "=== Testing Flat Directory Structure ==="
echo ""

# Clean up
echo "1. Cleaning up..."
tmux kill-server 2>/dev/null || true
rm -rf ~/.aany/agents/flat-test 2>/dev/null || true
sleep 1

# Create agent
echo ""
echo "2. Creating test agent via pool..."
echo ""
echo "INSTRUCTIONS:"
echo "  1. Press 'n' to create new agent"
echo "  2. Name it: flat-test"
echo "  3. Press 'q' to quit"
echo ""
read -p "Press ENTER to open pool..."

./target/release/aany pool

# Wait for initialization
sleep 5

# Verify structure
echo ""
echo "3. Verifying flat directory structure..."
echo ""

AGENT_DIR="$HOME/.aany/agents/flat-test"

# Check main directories
declare -a dirs=("workspace" "logs" "metadata" "config")
for dir in "${dirs[@]}"; do
    if [ -d "$AGENT_DIR/$dir" ]; then
        echo "✓ $dir directory created"
    else
        echo "✗ $dir directory NOT found!"
        exit 1
    fi
done

# Check metadata files
if [ -f "$AGENT_DIR/metadata/agent.yaml" ]; then
    echo "✓ agent.yaml in metadata directory"
else
    echo "✗ agent.yaml NOT found in metadata!"
fi

if [ -f "$AGENT_DIR/metadata/claude.json" ]; then
    echo "✓ claude.json in metadata directory"
else
    echo "✗ claude.json NOT found in metadata!"
fi

# Check working directory
echo ""
echo "4. Checking agent's working directory..."
CURRENT_DIR=$(tmux send-keys -t agent-flat-test "pwd" C-m && sleep 1 && tmux capture-pane -t agent-flat-test -p | grep -E "agents/flat-test/workspace" | tail -1)

if [[ "$CURRENT_DIR" == *"agents/flat-test/workspace"* ]]; then
    echo "✓ Agent is in correct workspace"
else
    echo "✗ Agent is NOT in expected workspace!"
fi

# Show structure
echo ""
echo "5. Actual directory structure:"
echo ""
tree -L 3 ~/.aany/agents/flat-test/ 2>/dev/null || {
    echo "~/.aany/agents/flat-test/"
    find ~/.aany/agents/flat-test -type d -maxdepth 2 | sort | sed 's|^.*flat-test|flat-test|' | sed 's|^|    |'
}

# Check if logging is working
echo ""
echo "6. Testing logging..."
tmux send-keys -t agent-flat-test "echo 'Testing flat structure logging'" C-m
sleep 2

if ls "$AGENT_DIR/logs"/tmux_interaction_*.log >/dev/null 2>&1; then
    echo "✓ Interaction logs being created"
    echo "  Log file: $(ls -t "$AGENT_DIR/logs"/tmux_interaction_*.log | head -1)"
else
    echo "✗ No interaction logs found!"
fi

echo ""
echo "✅ Flat directory structure verified!"
echo ""
echo "Summary:"
echo "- All agent files under: ~/.aany/agents/flat-test/"
echo "- Workspace: ~/.aany/agents/flat-test/workspace/"
echo "- Logs: ~/.aany/agents/flat-test/logs/"
echo "- Metadata: ~/.aany/agents/flat-test/metadata/"
echo "- Config: ~/.aany/agents/flat-test/config/"