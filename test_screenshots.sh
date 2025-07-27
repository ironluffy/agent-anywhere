#!/bin/bash
# Test tmux screenshot logging

set -e

echo "=== Testing Tmux Screenshot Logging ==="
echo ""

# Clean up
echo "1. Cleaning up..."
tmux kill-server 2>/dev/null || true
rm -rf ~/.aany/agents/screenshot-test 2>/dev/null || true
sleep 1

# Create test agent
echo ""
echo "2. Creating test agent..."
./target/release/aany tmux new screenshot-test -d

# Wait for initialization
echo "Waiting for agent to initialize..."
sleep 5

# Send some test content
echo ""
echo "3. Sending test content to create visual variety..."

tmux send-keys -t agent-screenshot-test "clear" C-m
sleep 1

tmux send-keys -t agent-screenshot-test "echo '═══════════════════════════════════════'" C-m
tmux send-keys -t agent-screenshot-test "echo '   SCREENSHOT TEST AGENT'" C-m  
tmux send-keys -t agent-screenshot-test "echo '═══════════════════════════════════════'" C-m
sleep 2

tmux send-keys -t agent-screenshot-test "date" C-m
sleep 1

tmux send-keys -t agent-screenshot-test "ls -la" C-m
sleep 2

tmux send-keys -t agent-screenshot-test "echo 'Testing screenshot capture...'" C-m
sleep 1

# Wait for screenshots
echo ""
echo "4. Waiting for screenshots to be captured (30s interval)..."
echo "   First screenshot should appear in ~30 seconds..."

# Check for screenshots
SCREENSHOT_DIR="$HOME/.aany/agents/screenshot-test/logs/screenshots"
echo "   Watching: $SCREENSHOT_DIR"

# Wait and check
for i in {1..40}; do
    if ls "$SCREENSHOT_DIR"/pane_*.txt >/dev/null 2>&1; then
        echo ""
        echo "✓ Screenshots being captured!"
        break
    fi
    echo -n "."
    sleep 1
done

echo ""
echo ""
echo "5. Screenshot status:"

if [ -d "$SCREENSHOT_DIR" ]; then
    COUNT=$(ls -1 "$SCREENSHOT_DIR"/pane_*.txt 2>/dev/null | wc -l || echo 0)
    echo "   Screenshots captured: $COUNT"
    
    if [ $COUNT -gt 0 ]; then
        echo ""
        echo "   Latest screenshots:"
        ls -1t "$SCREENSHOT_DIR"/pane_*.txt 2>/dev/null | head -5 | while read f; do
            echo "   - $(basename "$f")"
        done
        
        echo ""
        echo "6. Viewing a screenshot:"
        echo ""
        
        # Show the latest screenshot
        LATEST=$(ls -1t "$SCREENSHOT_DIR"/pane_*_plain.txt 2>/dev/null | head -1)
        if [ -n "$LATEST" ]; then
            echo "Content of latest screenshot:"
            echo "───────────────────────────────────────────"
            cat "$LATEST"
            echo "───────────────────────────────────────────"
        fi
    fi
else
    echo "   Screenshot directory not created!"
fi

echo ""
echo "7. Usage:"
echo "   View screenshots: ./aany-tmux/view-screenshots.sh screenshot-test"
echo "   List all: ./aany-tmux/view-screenshots.sh screenshot-test"
echo "   View specific: ./aany-tmux/view-screenshots.sh screenshot-test 1"
echo ""
echo "   Screenshot location: ~/.aany/agents/screenshot-test/logs/screenshots/"
echo "   Monitor session: tmux attach -r -t agent-screenshot-test"
echo ""
echo "✅ Screenshot logging test complete!"