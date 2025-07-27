#!/bin/bash
# Automated TODO_ULTIMATE_TEST - No manual intervention required

set -e

echo "=== AUTOMATED TODO_ULTIMATE_TEST ==="
echo ""

# Clean up
echo "1. Cleaning up existing sessions..."
tmux kill-server 2>/dev/null || true
pkill -f tmux-interaction-logger 2>/dev/null || true
pkill -f tmux-keystroke-logger 2>/dev/null || true
sleep 1

# Create agent directly
echo ""
echo "2. Creating agent 'ultimate-test' with full logging..."

# Use aany command to create agent
./target/release/aany tmux new ultimate-test

# Wait for initialization
echo ""
echo "3. Waiting for agent initialization..."
sleep 5

# Verify agent exists
if ! tmux has-session -t "agent-ultimate-test" 2>/dev/null; then
    echo "ERROR: Failed to create agent session!"
    exit 1
fi

echo "✓ Agent created successfully"

# Send test commands
echo ""
echo "4. Sending test commands..."

echo "   - Command 1: nvm use 22"
tmux send-keys -t "agent-ultimate-test" "nvm use 22" C-m
sleep 3

echo "   - Command 2: claude"
tmux send-keys -t "agent-ultimate-test" "claude" C-m
sleep 3

echo "   - Command 3: tell me about this directory"
tmux send-keys -t "agent-ultimate-test" "tell me about this directory" C-m
sleep 3

# Wait for logging
echo ""
echo "5. Waiting for logs to be written..."
sleep 5

# Find logs
echo ""
echo "6. Locating log files..."

LOG_FILE=$(ls -t /tmp/tmux-agent-logs/tmux_interaction_ultimate-test_*.log 2>/dev/null | head -1)

if [ -z "$LOG_FILE" ]; then
    echo "ERROR: No log file found!"
    echo "Checking for running loggers..."
    ps aux | grep -E "(tmux-interaction-logger|ultimate-test)" | grep -v grep
    echo ""
    echo "Checking log directory:"
    ls -la /tmp/tmux-agent-logs/ 2>/dev/null || echo "Log directory doesn't exist"
    exit 1
fi

echo "Found log: $LOG_FILE"
echo ""

# Display full log
echo "=== FULL LOG CONTENTS ==="
cat "$LOG_FILE"
echo "========================="
echo ""

# Verification
echo "7. Verifying requirements..."
echo ""

PASS=0
TOTAL=0

# Function to check requirement
check_requirement() {
    local description="$1"
    local pattern="$2"
    ((TOTAL++))
    
    echo -n "✓ $description: "
    if grep -q "$pattern" "$LOG_FILE"; then
        echo "PASS"
        ((PASS++))
        return 0
    else
        echo "FAIL (pattern: $pattern)"
        return 1
    fi
}

# Check all requirements
check_requirement "Agent ID present" "Agent ID: ultimate-test"
check_requirement "Session name present" "Session: agent-ultimate-test"
check_requirement "Hub URL present" "Hub URL:"
check_requirement "Screen history captured" "screen_"
check_requirement "nvm command logged" "nvm use 22"
check_requirement "claude command logged" "claude"
check_requirement "directory query logged" "tell me about this directory"

# Also check for user typing detection
echo ""
echo "Additional checks:"
if grep -q "user_typing" "$LOG_FILE"; then
    echo "✓ User typing detection: FOUND"
    grep "user_typing" "$LOG_FILE" | head -5
else
    echo "✗ User typing detection: NOT FOUND"
fi

# Final result
echo ""
echo "================================"
echo "FINAL RESULT: $PASS/$TOTAL tests passed"
echo ""

if [ $PASS -eq $TOTAL ]; then
    echo "✅ TODO_ULTIMATE_TEST PASSED!"
    echo ""
    echo "The tmux-agent successfully logged:"
    echo "- All user input (commands typed)"
    echo "- Complete screen history"
    echo "- Agent identifier and metadata"
    echo "- Hub connection information"
else
    echo "❌ TODO_ULTIMATE_TEST FAILED!"
    echo ""
    echo "Missing $(($TOTAL - $PASS)) required elements."
fi

echo ""
echo "Resources:"
echo "- Log file: $LOG_FILE"
echo "- Live session: tmux attach -r -t agent-ultimate-test"
echo "- Kill session: tmux kill-session -t agent-ultimate-test"

exit $(($TOTAL - $PASS))