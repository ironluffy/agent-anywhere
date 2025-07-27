#!/bin/bash

echo "Testing aany-hub terminal broadcaster setup"
echo "=========================================="

# Check if virtual environment is activated
if [[ "$VIRTUAL_ENV" != *".venv"* ]]; then
    echo "Activating virtual environment..."
    source .venv/bin/activate
fi

# Install/update dependencies
echo "Installing dependencies..."
pip install -r requirements.txt -q

# Recompile protobuf
echo "Compiling protobuf..."
python compile_proto.py

# Start the hub server
echo "Starting aany-hub server..."
python -m src.aany_hub.main &
HUB_PID=$!

# Wait for server to start
sleep 3

echo ""
echo "Hub server started with PID: $HUB_PID"
echo "Dashboard available at: http://localhost:8090"
echo "Terminal UI available at: http://localhost:8090/terminal"
echo ""
echo "To test the terminal broadcaster:"
echo "1. Create a tmux session: tmux new -s test-session"
echo "2. Run tmux-agent with terminal broadcaster module"
echo "3. Open http://localhost:8090/terminal in your browser"
echo ""
echo "Press Ctrl+C to stop the server"

# Wait for interrupt
trap "kill $HUB_PID; exit" INT
wait $HUB_PID