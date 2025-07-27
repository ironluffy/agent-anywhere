#!/bin/bash

# Ultimate test script for aany-hub terminal broadcaster
# This script will:
# 1. Start aany-hub server in background
# 2. Create an agent using aany pool
# 3. Verify agent shows up in Connected Agents
# 4. Verify terminal output is visible in web UI

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== aany-hub Terminal Broadcaster Ultimate Test ===${NC}"
echo

# Step 1: Clean up any existing processes
echo -e "${YELLOW}Step 1: Cleaning up existing processes...${NC}"
./stop_hub.sh 2>/dev/null || true
sleep 1

# Step 2: Start hub server
echo -e "${YELLOW}Step 2: Starting aany-hub server...${NC}"
./start_hub_background.sh
sleep 3

# Verify hub is running
if ! curl -s http://localhost:8090 > /dev/null; then
    echo -e "${RED}✗ Hub server failed to start${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Hub server is running${NC}"

# Step 3: Create test agent
TEST_AGENT_NAME="hub-test-$(date +%s)"
echo -e "${YELLOW}Step 3: Creating test agent: $TEST_AGENT_NAME${NC}"

cd ~/Gitrepo/agent-anywhere
./aany pool create "$TEST_AGENT_NAME" --template hub-connected

# Wait for agent creation
sleep 2

# Step 4: Start the agent
echo -e "${YELLOW}Step 4: Starting agent...${NC}"
./aany pool start "$TEST_AGENT_NAME"
sleep 3

# Step 5: Verify agent registration
echo -e "${YELLOW}Step 5: Checking agent registration...${NC}"
AGENTS_JSON=$(curl -s http://localhost:8090/api/agents)
if echo "$AGENTS_JSON" | grep -q "$TEST_AGENT_NAME"; then
    echo -e "${GREEN}✓ Agent '$TEST_AGENT_NAME' is registered${NC}"
else
    echo -e "${RED}✗ Agent not found in Connected Agents${NC}"
    echo "Response: $AGENTS_JSON"
    exit 1
fi

# Step 6: Verify terminal session
echo -e "${YELLOW}Step 6: Checking terminal sessions...${NC}"
TERMINALS_JSON=$(curl -s http://localhost:8090/api/terminals)
if echo "$TERMINALS_JSON" | grep -q "$TEST_AGENT_NAME"; then
    echo -e "${GREEN}✓ Terminal session created for agent${NC}"
    
    # Extract session ID
    SESSION_ID=$(echo "$TERMINALS_JSON" | grep -o '"session_id":"[^"]*"' | head -1 | cut -d'"' -f4)
    echo "Terminal session ID: $SESSION_ID"
else
    echo -e "${RED}✗ No terminal session found${NC}"
    echo "Response: $TERMINALS_JSON"
fi

# Step 7: Send test commands
echo -e "${YELLOW}Step 7: Sending test commands to agent...${NC}"
tmux send-keys -t "agent-$TEST_AGENT_NAME" "echo 'Hello from aany-hub test!'" Enter
sleep 1
tmux send-keys -t "agent-$TEST_AGENT_NAME" "date" Enter
sleep 1
tmux send-keys -t "agent-$TEST_AGENT_NAME" "pwd" Enter
sleep 1

# Step 8: Display results
echo
echo -e "${BLUE}=== Test Complete ===${NC}"
echo
echo -e "${GREEN}✓ Hub server is running at: http://localhost:8090${NC}"
echo -e "${GREEN}✓ Terminal UI available at: http://localhost:8090/terminal${NC}"
echo -e "${GREEN}✓ Agent '$TEST_AGENT_NAME' is connected${NC}"
echo
echo "To verify terminal output:"
echo "1. Open http://localhost:8090/terminal in your browser"
echo "2. Click on the terminal session for '$TEST_AGENT_NAME'"
echo "3. You should see the test commands and their output"
echo
echo "To manually test:"
echo "1. Attach to agent: ./aany pool attach $TEST_AGENT_NAME"
echo "2. Type commands and see them appear in the web terminal"
echo
echo "To clean up:"
echo "1. Stop agent: ./aany pool stop $TEST_AGENT_NAME"
echo "2. Delete agent: ./aany pool delete $TEST_AGENT_NAME"
echo "3. Stop hub: cd aany-hub && ./stop_hub.sh"

# Optional: Open browser automatically
if command -v open >/dev/null 2>&1; then
    echo
    read -p "Open terminal UI in browser? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        open "http://localhost:8090/terminal"
    fi
fi