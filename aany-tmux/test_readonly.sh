#!/bin/bash

echo "Testing tmux read-only behavior..."

# Create a test session
tmux new-session -d -s test-ro -c /tmp
tmux send-keys -t test-ro "echo 'Test session created'" Enter

# Test different attach methods
echo ""
echo "According to tmux manual:"
echo "  -r is an alias for -f read-only,ignore-size"
echo ""

# Show the flags
echo "Client flags format:"
echo "  #{client_flags} shows client flags"
echo "  #{client_readonly} shows 1 if read-only"
echo ""

echo "To test:"
echo "1. In terminal 1: tmux attach -t test-ro"
echo "2. In terminal 2: tmux attach -r -t test-ro"
echo "3. In terminal 3: Run this to see clients:"
echo "   tmux list-clients -t test-ro -F '#{client_tty}: flags=#{client_flags} readonly=#{client_readonly}'"
echo ""
echo "Session 'test-ro' created. Clean up with: tmux kill-session -t test-ro"