#!/bin/bash
# TODO_ULTIMATE_TEST script - Test complete interaction logging

set -e

echo "=== TODO_ULTIMATE_TEST: Testing tmux-agent interaction logging ==="
echo ""

# Make sure aany-hub is running
echo "1. Checking if aany-hub is running..."
if ! pgrep -f "aany-hub" > /dev/null; then
    echo "   Starting aany-hub..."
    cd target/release
    ./aany-hub &
    HUB_PID=$!
    cd ../..
    sleep 2
else
    echo "   aany-hub is already running"
    HUB_PID=""
fi

# Test 1: Create agent from aany pool UI
echo ""
echo "2. Creating agent from aany pool UI..."
echo "   Running: ./target/release/aany pool"
echo ""
echo "   Instructions:"
echo "   - Press 'n' to create new agent"
echo "   - Enter name: 'ultimate-test'"
echo "   - Press 'q' to quit"
echo ""
echo "Press ENTER to continue..."
read

./target/release/aany pool

# Test 2: Attach and run commands
echo ""
echo "3. Attaching to agent and running test commands..."
echo "   This will:"
echo "   - Attach to the agent"
echo "   - Type 'nvm use 22'"
echo "   - Type 'claude'"
echo "   - Type 'tell me about this directory'"
echo ""
echo "Press ENTER to continue..."
read

# Send commands to the agent
tmux send-keys -t agent-ultimate-test 'nvm use 22' C-m
sleep 2
tmux send-keys -t agent-ultimate-test 'claude' C-m
sleep 2
tmux send-keys -t agent-ultimate-test 'tell me about this directory' C-m

echo ""
echo "4. Waiting for interactions to be logged..."
sleep 5

# Test 3: Verify logs
echo ""
echo "5. Verifying interaction logs..."
echo ""

# Find the log file
LOG_FILE=$(ls -t /tmp/tmux-agent-logs/tmux_interaction_ultimate-test_*.log 2>/dev/null | head -1)

if [ -z "$LOG_FILE" ]; then
    echo "ERROR: No log file found for ultimate-test agent!"
    exit 1
fi

echo "Found log file: $LOG_FILE"
echo ""
echo "=== LOG CONTENTS ==="
cat "$LOG_FILE"
echo "===================="
echo ""

# Verify required elements
echo "Checking for required elements in log:"
echo -n "✓ Agent ID (ultimate-test): "
if grep -q "Agent ID: ultimate-test" "$LOG_FILE"; then
    echo "FOUND"
else
    echo "NOT FOUND"
fi

echo -n "✓ Session name: "
if grep -q "Session: agent-ultimate-test" "$LOG_FILE"; then
    echo "FOUND"
else
    echo "NOT FOUND"
fi

echo -n "✓ User typing 'nvm use 22': "
if grep -q "nvm use 22" "$LOG_FILE"; then
    echo "FOUND"
else
    echo "NOT FOUND"
fi

echo -n "✓ User typing 'claude': "
if grep -q "claude" "$LOG_FILE"; then
    echo "FOUND"
else
    echo "NOT FOUND"
fi

echo -n "✓ User typing 'tell me about this directory': "
if grep -q "tell me about this directory" "$LOG_FILE"; then
    echo "FOUND"
else
    echo "NOT FOUND"
fi

echo -n "✓ Screen history updates: "
if grep -q "screen_update" "$LOG_FILE"; then
    echo "FOUND"
else
    echo "NOT FOUND"
fi

echo ""
echo "=== TODO_ULTIMATE_TEST COMPLETE ==="
echo ""
echo "To view the full log:"
echo "cat $LOG_FILE"
echo ""
echo "To monitor the agent:"
echo "tmux attach -r -t agent-ultimate-test"

# Cleanup
if [ -n "$HUB_PID" ]; then
    echo ""
    echo "Stopping aany-hub..."
    kill $HUB_PID 2>/dev/null || true
fi