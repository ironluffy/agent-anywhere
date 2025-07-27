#!/bin/bash
# Integration test for aany-hub and tmux-agent with hub-connector module

echo "🚀 Agent Anywhere Integration Test"
echo "================================"

# Check if tmux is running
if ! tmux info &> /dev/null; then
    echo "❌ Error: tmux server is not running. Please start tmux first."
    exit 1
fi

# Start aany-hub in a tmux session
echo "1. Starting aany-hub server..."
tmux new-session -d -s aany-hub-test "cd aany-hub && source .venv/bin/activate && python -m aany_hub.main"
sleep 3

# Check if hub is running
echo "2. Checking hub status..."
if curl -s http://localhost:8000 > /dev/null; then
    echo "✅ Hub is running at http://localhost:8000"
else
    echo "❌ Hub failed to start"
    tmux kill-session -t aany-hub-test
    exit 1
fi

# Run tmux-agent with hub connector
echo ""
echo "3. Starting tmux-agent with hub-connector module..."
cd aany-tmux
source ~/.cargo/env

# Create a test tmux session for the agent
tmux new-session -d -s test-agent "echo 'Test agent session ready'"

# Run the modular agent
echo "4. Running modular agent with hub connection..."
cargo run --example modular_agent

echo ""
echo "5. Integration test complete!"
echo ""
echo "📊 Check the hub dashboard:"
echo "   http://localhost:8000"
echo ""
echo "🔍 View hub logs:"
echo "   tmux attach -t aany-hub-test"
echo ""
echo "🧹 Cleanup:"
echo "   tmux kill-session -t aany-hub-test"
echo "   tmux kill-session -t test-agent"