#!/bin/bash
# Simple demonstration of typing simulation with tmux

echo "🎹 Agent Anywhere Typing Demo"
echo "============================"

# Kill existing demo session
tmux kill-session -t demo-typing 2>/dev/null

# Create new session
echo "Creating tmux session..."
tmux new-session -d -s demo-typing

# Function to type slowly
type_slowly() {
    local text="$1"
    local delay="${2:-0.05}"
    
    for (( i=0; i<${#text}; i++ )); do
        char="${text:$i:1}"
        tmux send-keys -t demo-typing "$char"
        sleep $delay
    done
}

# Start the demo
echo "Starting typing simulation..."

# Clear screen
tmux send-keys -t demo-typing "clear" Enter
sleep 1

# Type header
echo "📝 Phase 1: Slow typing (thinking)..."
type_slowly "# Welcome to Agent Anywhere!" 0.1
tmux send-keys -t demo-typing Enter
sleep 1

type_slowly "# This demonstrates keyboard simulation" 0.08
tmux send-keys -t demo-typing Enter
sleep 1

# Type commands
echo "📝 Phase 2: Normal typing..."
type_slowly "echo 'Agent Anywhere can type like a human!'" 0.05
tmux send-keys -t demo-typing Enter
sleep 1

# Fast typing
echo "📝 Phase 3: Fast typing..."
type_slowly "date && hostname && whoami" 0.02
tmux send-keys -t demo-typing Enter
sleep 1

# Execute some commands
echo "🚀 Running commands..."
commands=(
    "ls -la | head -5"
    "ps aux | grep tmux | head -3"
    "echo 'Current time: '$(date +%H:%M:%S)"
)

for cmd in "${commands[@]}"; do
    echo "   Typing: $cmd"
    type_slowly "$cmd" 0.03
    tmux send-keys -t demo-typing Enter
    sleep 1
done

# Simulate typo and correction
echo "✏️  Simulating typo..."
type_slowly "ehco 'Oops, made a typo!'" 0.05
sleep 0.5

# Delete the typo
echo "   Fixing typo..."
for i in {1..24}; do
    tmux send-keys -t demo-typing BSpace
    sleep 0.02
done

type_slowly "echo 'Fixed the typo!'" 0.05
tmux send-keys -t demo-typing Enter
sleep 1

# Final message
echo "🎯 Final message..."
type_slowly "echo '✅ Demo complete! Agent Anywhere rocks!'" 0.04
tmux send-keys -t demo-typing Enter

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📸 Capturing session output..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Show the output
tmux capture-pane -t demo-typing -p | tail -30

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✨ Demo complete!"
echo ""
echo "To see the live session: tmux attach -t demo-typing"
echo "To cleanup:             tmux kill-session -t demo-typing"