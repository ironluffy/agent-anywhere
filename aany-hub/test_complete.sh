#!/bin/bash

# Complete test script for aany-hub terminal broadcaster
# This script demonstrates the full workflow

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== AANY-HUB TERMINAL BROADCASTER TEST ===${NC}"
echo

# Configuration
AGENT_NAME="demo-agent"
HUB_URL="localhost:50052"
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
AANY_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"

# Function to check if a command succeeded
check_status() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ $1${NC}"
    else
        echo -e "${RED}✗ $1 failed${NC}"
        exit 1
    fi
}

# Step 1: Ensure hub is running
echo -e "${YELLOW}Step 1: Checking hub server...${NC}"
if ! curl -s http://localhost:8090 > /dev/null; then
    echo "Starting hub server..."
    cd "$SCRIPT_DIR"
    ./start_hub_background.sh
    sleep 3
fi
check_status "Hub server running"

# Step 2: Build aany if needed
echo -e "${YELLOW}Step 2: Preparing aany command...${NC}"
cd "$AANY_DIR"
if [ ! -f "target/debug/aany" ]; then
    cargo build --bin aany > /dev/null 2>&1
fi
check_status "aany command ready"

# Step 3: Clean up any existing test agent
echo -e "${YELLOW}Step 3: Cleaning up...${NC}"
echo "yes" | ./target/debug/aany pool delete "$AGENT_NAME" 2>/dev/null || true
check_status "Cleanup complete"

# Step 4: Create agent
echo -e "${YELLOW}Step 4: Creating agent '$AGENT_NAME'...${NC}"
./target/debug/aany pool create "$AGENT_NAME"
check_status "Agent created"

# Step 5: Start agent
echo -e "${YELLOW}Step 5: Starting agent...${NC}"
./target/debug/aany pool start "$AGENT_NAME"
sleep 2
check_status "Agent started"

# Step 6: Connect agent to hub
echo -e "${YELLOW}Step 6: Connecting agent to hub...${NC}"
tmux send-keys -t "agent-$AGENT_NAME" "bash $SCRIPT_DIR/connect_agent_to_hub.sh" Enter
sleep 3

# Verify connection
if curl -s http://localhost:8090/api/agents | grep -q "$AGENT_NAME"; then
    check_status "Agent connected to hub"
else
    echo -e "${RED}✗ Agent not connected${NC}"
    echo "Checking tmux output:"
    tmux capture-pane -t "agent-$AGENT_NAME" -p | tail -20
    exit 1
fi

# Step 7: Verify terminal session
echo -e "${YELLOW}Step 7: Verifying terminal session...${NC}"
TERMINALS=$(curl -s http://localhost:8090/api/terminals)
if echo "$TERMINALS" | grep -q "$AGENT_NAME"; then
    check_status "Terminal session created"
    SESSION_COUNT=$(echo "$TERMINALS" | jq '.terminals | length')
    echo "  Found $SESSION_COUNT terminal session(s)"
else
    echo -e "${RED}✗ No terminal session found${NC}"
    exit 1
fi

# Step 8: Send test commands
echo -e "${YELLOW}Step 8: Sending test commands...${NC}"
tmux send-keys -t "agent-$AGENT_NAME" "clear" Enter
sleep 0.5
tmux send-keys -t "agent-$AGENT_NAME" "echo '🎉 Welcome to aany-hub terminal broadcaster!'" Enter
sleep 0.5
tmux send-keys -t "agent-$AGENT_NAME" "echo 'This terminal is being broadcast to the web in real-time.'" Enter
sleep 0.5
tmux send-keys -t "agent-$AGENT_NAME" "date" Enter
sleep 0.5
tmux send-keys -t "agent-$AGENT_NAME" "pwd" Enter
sleep 0.5
tmux send-keys -t "agent-$AGENT_NAME" "echo 'Try typing commands here and watch them appear in the browser!'" Enter
check_status "Test commands sent"

# Final summary
echo
echo -e "${BLUE}=== TEST COMPLETE ===${NC}"
echo
echo -e "${GREEN}All components are working correctly!${NC}"
echo
echo "Access points:"
echo "  • Dashboard: ${BLUE}http://localhost:8090${NC}"
echo "  • Terminal UI: ${BLUE}http://localhost:8090/terminal${NC}"
echo
echo "What you can do now:"
echo "  1. Open the Terminal UI in your browser"
echo "  2. Click on the '$AGENT_NAME' session"
echo "  3. Watch the live terminal output"
echo "  4. Attach to the agent: ${YELLOW}./target/debug/aany pool attach $AGENT_NAME${NC}"
echo "  5. Type commands and see them in the web terminal!"
echo
echo "Cleanup commands:"
echo "  • Stop agent: ${YELLOW}./target/debug/aany pool stop $AGENT_NAME${NC}"
echo "  • Delete agent: ${YELLOW}echo yes | ./target/debug/aany pool delete $AGENT_NAME${NC}"
echo "  • Stop hub: ${YELLOW}cd $SCRIPT_DIR && ./stop_hub.sh${NC}"
echo

# Ask to open browser
read -p "Open Terminal UI in browser? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if command -v open >/dev/null 2>&1; then
        open "http://localhost:8090/terminal"
    elif command -v xdg-open >/dev/null 2>&1; then
        xdg-open "http://localhost:8090/terminal"
    else
        echo "Please open http://localhost:8090/terminal in your browser"
    fi
fi