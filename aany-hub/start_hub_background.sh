#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Check if hub is already running
if pgrep -f "src.aany_hub.main" > /dev/null; then
    echo -e "${YELLOW}aany-hub is already running${NC}"
    echo "To stop it, run: ./stop_hub.sh"
    exit 1
fi

# Activate virtual environment
if [ ! -d ".venv" ]; then
    echo -e "${RED}Virtual environment not found. Creating...${NC}"
    python3 -m venv .venv
fi

source .venv/bin/activate

# Install dependencies
echo "Installing/updating dependencies..."
pip install -r requirements.txt -q

# Compile protobuf
echo "Compiling protobuf definitions..."
python compile_proto.py
python fix_proto_imports.py

# Create logs directory
mkdir -p logs

# Start hub in background
echo -e "${GREEN}Starting aany-hub server in background...${NC}"
nohup python -m src.aany_hub.main > logs/aany-hub.log 2>&1 &
HUB_PID=$!

# Save PID for stop script
echo $HUB_PID > .hub.pid

# Wait a moment for server to start
sleep 2

# Check if server started successfully
if kill -0 $HUB_PID 2>/dev/null; then
    echo -e "${GREEN}✓ aany-hub started successfully (PID: $HUB_PID)${NC}"
    echo ""
    echo "Dashboard: http://localhost:8090"
    echo "Terminal UI: http://localhost:8090/terminal"
    echo "gRPC Port: 50052"
    echo ""
    echo "Logs: tail -f logs/aany-hub.log"
    echo "Stop: ./stop_hub.sh"
else
    echo -e "${RED}✗ Failed to start aany-hub${NC}"
    echo "Check logs/aany-hub.log for details"
    exit 1
fi