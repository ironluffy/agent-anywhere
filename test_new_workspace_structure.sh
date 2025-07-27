#!/bin/bash
# Test new workspace directory structure

set -e

echo "=== Testing New Workspace Structure ==="
echo ""

# Clean up
echo "1. Cleaning up..."
tmux kill-server 2>/dev/null || true
rm -rf ~/.aany/agents/workspace-test 2>/dev/null || true
rm -rf ~/.aany/workspaces/workspace-test 2>/dev/null || true
sleep 1

# Create agent
echo ""
echo "2. Creating agent via pool..."
echo "   Name: workspace-test"
echo ""
echo "INSTRUCTIONS:"
echo "  1. Press 'n' to create new agent"
echo "  2. Name it: workspace-test"
echo "  3. Press 'q' to quit"
echo ""
read -p "Press ENTER to open pool..."

./target/release/aany pool

# Wait for initialization
sleep 5

# Verify structure
echo ""
echo "3. Verifying directory structure..."

# Check workspace
if [ -d "$HOME/.aany/workspaces/workspace-test" ]; then
    echo "✓ Workspace created at: ~/.aany/workspaces/workspace-test"
else
    echo "✗ Workspace NOT found at expected location!"
    exit 1
fi

# Check agent dir (metadata/logs)
if [ -d "$HOME/.aany/agents/workspace-test" ]; then
    echo "✓ Agent dir created at: ~/.aany/agents/workspace-test"
else
    echo "✗ Agent dir NOT found!"
    exit 1
fi

# Check logs dir
if [ -d "$HOME/.aany/agents/workspace-test/logs" ]; then
    echo "✓ Logs dir created at: ~/.aany/agents/workspace-test/logs"
else
    echo "✗ Logs dir NOT found!"
    exit 1
fi

# Check working directory
echo ""
echo "4. Checking agent's working directory..."
CURRENT_DIR=$(tmux send-keys -t agent-workspace-test "pwd" C-m && sleep 1 && tmux capture-pane -t agent-workspace-test -p | grep -E "^/.*/workspaces/workspace-test" | tail -1)

if [[ "$CURRENT_DIR" == *"/.aany/workspaces/workspace-test"* ]]; then
    echo "✓ Agent is in correct workspace: $CURRENT_DIR"
else
    echo "✗ Agent is NOT in expected workspace!"
    echo "Current dir: $(tmux capture-pane -t agent-workspace-test -p | tail -5)"
    exit 1
fi

# Show structure
echo ""
echo "5. Directory structure:"
echo ""
echo "~/.aany/"
echo "├── agents/"
echo "│   └── workspace-test/"
echo "│       ├── .agent.yaml      (metadata)"
echo "│       ├── .claude/          (claude settings)"
echo "│       └── logs/             (interaction logs)"
echo "└── workspaces/"
echo "    └── workspace-test/       (working directory)"
echo ""

# List actual files
echo "Actual structure:"
echo ""
echo "Agent directory:"
ls -la ~/.aany/agents/workspace-test/
echo ""
echo "Workspace directory:"
ls -la ~/.aany/workspaces/workspace-test/
echo ""

echo "✅ New workspace structure verified!"
echo ""
echo "Summary:"
echo "- Workspace: ~/.aany/workspaces/workspace-test"
echo "- Logs: ~/.aany/agents/workspace-test/logs"
echo "- Metadata: ~/.aany/agents/workspace-test/.agent.yaml"