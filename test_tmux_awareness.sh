#!/bin/bash

echo "=== Testing tmux awareness in aany pool ==="
echo

# Test 1: Check if we're in tmux
if [ -n "$TMUX" ]; then
    echo "✓ Currently inside tmux session"
    echo "  TMUX=$TMUX"
else
    echo "✗ Not currently in tmux"
    echo "  To test nested tmux handling, run this script from inside a tmux session"
fi
echo

# Test 2: Create a test agent if it doesn't exist
echo "Checking for test agent..."
if ! ./target/debug/aany pool list | grep -q "test-tmux-agent"; then
    echo "Creating test agent..."
    ./target/debug/aany pool create test-tmux-agent --template general
fi

# Test 3: Start the agent if not running
echo "Checking agent status..."
STATUS=$(./target/debug/aany pool status test-tmux-agent 2>/dev/null | grep "Status:" | awk '{print $2}')
if [ "$STATUS" != "Active" ]; then
    echo "Starting test agent..."
    ./target/debug/aany pool start test-tmux-agent
    sleep 2
fi

echo
echo "=== Testing attach functionality ==="
echo

if [ -n "$TMUX" ]; then
    echo "Test: Attaching from inside tmux (should use switch-client)"
    echo "Press 'q' to return from the agent session"
    read -p "Press Enter to continue..." 
    ./target/debug/aany pool attach test-tmux-agent
    echo "✓ Returned from agent session successfully"
else
    echo "Test: Attaching from outside tmux (should use attach-session)"
    echo "Press Ctrl+B then D to detach from the agent session"
    read -p "Press Enter to continue..." 
    ./target/debug/aany pool attach test-tmux-agent
    echo "✓ Detached from agent session successfully"
fi

echo
echo "=== Test complete ==="