#!/bin/bash
# TODO_ULTIMATE_TEST - Comprehensive test of tmux-agent logging

set -e

echo "=== TODO_ULTIMATE_TEST - Full Interaction Logging Test ==="
echo ""

# Clean up any existing sessions
echo "1. Cleaning up existing sessions..."
tmux kill-server 2>/dev/null || true
sleep 1

# Create test agent using aany pool
echo ""
echo "2. Creating test agent via aany pool UI..."
echo ""
echo "IMPORTANT: When the UI opens:"
echo "  1. Press 'n' to create new agent"
echo "  2. Name it: ultimate-test"
echo "  3. Press 'q' to quit"
echo ""
read -p "Press ENTER to open aany pool UI..."

# Run aany pool
./target/release/aany pool

# Wait for agent to fully initialize
echo ""
echo "3. Waiting for agent to initialize with logging..."
sleep 3

# Verify agent was created
if ! tmux has-session -t "agent-ultimate-test" 2>/dev/null; then
    echo "ERROR: Agent session 'agent-ultimate-test' was not created!"
    exit 1
fi

echo "✓ Agent created successfully"

# Attach to agent and send test commands
echo ""
echo "4. Sending test commands to agent..."

# Send nvm command
echo "   - Sending: nvm use 22"
tmux send-keys -t "agent-ultimate-test" "nvm use 22" C-m
sleep 2

# Send claude command
echo "   - Sending: claude"
tmux send-keys -t "agent-ultimate-test" "claude" C-m
sleep 3

# Send directory query
echo "   - Sending: tell me about this directory"
tmux send-keys -t "agent-ultimate-test" "tell me about this directory" C-m
sleep 3

# Wait for all logging to complete
echo ""
echo "5. Waiting for logs to be written..."
sleep 5

# Find and verify logs
echo ""
echo "6. Checking logs..."
echo ""

# Find the interaction log
INTERACTION_LOG=$(ls -t /tmp/tmux-agent-logs/tmux_interaction_ultimate-test_*.log 2>/dev/null | head -1)
KEYSTROKE_LOG=$(ls -t /tmp/tmux-agent-logs/tmux_keystrokes_ultimate-test_*.log 2>/dev/null | head -1)

if [ -z "$INTERACTION_LOG" ]; then
    echo "ERROR: No interaction log found!"
    echo "Expected: /tmp/tmux-agent-logs/tmux_interaction_ultimate-test_*.log"
    exit 1
fi

echo "Found interaction log: $INTERACTION_LOG"
echo ""

# Display log contents
echo "=== INTERACTION LOG CONTENTS ==="
echo ""
cat "$INTERACTION_LOG"
echo ""
echo "================================"
echo ""

# Perform verification checks
echo "7. Verifying required elements..."
echo ""

PASS_COUNT=0
FAIL_COUNT=0

# Check for agent ID
echo -n "✓ Checking for Agent ID 'ultimate-test': "
if grep -q "Agent ID: ultimate-test" "$INTERACTION_LOG"; then
    echo "PASS"
    ((PASS_COUNT++))
else
    echo "FAIL"
    ((FAIL_COUNT++))
fi

# Check for session name
echo -n "✓ Checking for Session 'agent-ultimate-test': "
if grep -q "Session: agent-ultimate-test" "$INTERACTION_LOG"; then
    echo "PASS"
    ((PASS_COUNT++))
else
    echo "FAIL"
    ((FAIL_COUNT++))
fi

# Check for nvm command
echo -n "✓ Checking for 'nvm use 22' command: "
if grep -qE "(user_typing|screen_update).*nvm use 22" "$INTERACTION_LOG"; then
    echo "PASS"
    ((PASS_COUNT++))
else
    echo "FAIL"
    ((FAIL_COUNT++))
fi

# Check for claude command
echo -n "✓ Checking for 'claude' command: "
if grep -qE "(user_typing|screen_update).*claude" "$INTERACTION_LOG"; then
    echo "PASS"
    ((PASS_COUNT++))
else
    echo "FAIL"
    ((FAIL_COUNT++))
fi

# Check for directory query
echo -n "✓ Checking for 'tell me about this directory': "
if grep -qE "(user_typing|screen_update).*tell me about this directory" "$INTERACTION_LOG"; then
    echo "PASS"
    ((PASS_COUNT++))
else
    echo "FAIL"
    ((FAIL_COUNT++))
fi

# Check for screen history
echo -n "✓ Checking for screen history updates: "
if grep -q "screen_update" "$INTERACTION_LOG"; then
    echo "PASS"
    ((PASS_COUNT++))
else
    echo "FAIL"
    ((FAIL_COUNT++))
fi

# Check for metadata
echo -n "✓ Checking for Hub URL metadata: "
if grep -q "Hub URL:" "$INTERACTION_LOG"; then
    echo "PASS"
    ((PASS_COUNT++))
else
    echo "FAIL"
    ((FAIL_COUNT++))
fi

# Summary
echo ""
echo "================================"
echo "TEST SUMMARY:"
echo "  Passed: $PASS_COUNT/7"
echo "  Failed: $FAIL_COUNT/7"
echo ""

if [ $FAIL_COUNT -eq 0 ]; then
    echo "✅ TODO_ULTIMATE_TEST PASSED! All requirements met."
else
    echo "❌ TODO_ULTIMATE_TEST FAILED! Missing required elements."
fi

echo ""
echo "Additional info:"
echo "  - Full log: $INTERACTION_LOG"
echo "  - Monitor session: tmux attach -r -t agent-ultimate-test"
echo "  - Agent workspace: ~/.aany/agents/ultimate-test/"
echo ""

# Show sample of captured screen
echo "Sample of captured screen content:"
echo "---"
grep -A 2 "screen_update" "$INTERACTION_LOG" | head -20
echo "---"

exit $FAIL_COUNT