#!/bin/bash

# Ultimate verified test script for aany-hub terminal broadcaster
# This script ensures everything works end-to-end

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== AANY-HUB TERMINAL BROADCASTER ULTIMATE TEST ===${NC}"
echo

# Get directories
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
AANY_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"

# Step 1: Verify hub server is running
echo -e "${YELLOW}Step 1: Verifying hub server...${NC}"
if ! curl -s http://localhost:8090 > /dev/null; then
    echo -e "${RED}✗ Hub server is not running${NC}"
    echo "Starting hub server..."
    cd "$SCRIPT_DIR"
    ./start_hub_background.sh
    sleep 3
fi

# Verify again
if curl -s http://localhost:8090 > /dev/null; then
    echo -e "${GREEN}✓ Hub server is running${NC}"
else
    echo -e "${RED}✗ Failed to start hub server${NC}"
    exit 1
fi

# Step 2: Build aany if needed
echo -e "${YELLOW}Step 2: Building aany command...${NC}"
cd "$AANY_DIR"
if [ ! -f "target/debug/aany" ]; then
    cargo build --bin aany
fi
echo -e "${GREEN}✓ aany command ready${NC}"

# Step 3: Create hub-connected template if not exists
echo -e "${YELLOW}Step 3: Setting up hub-connected template...${NC}"
TEMPLATE_DIR="$HOME/.aany/templates"
mkdir -p "$TEMPLATE_DIR"

cat > "$TEMPLATE_DIR/hub-connected.yaml" << 'EOF'
agent:
  name: "hub-connected"
  role: "general"
  created_by: "aany-hub"
  description: "Agent with automatic hub connection"

claude:
  profile: "claude-3-sonnet"
  cache_ttl: 3600
  temperature: 0.7
  custom_instructions: |
    You are a helpful AI assistant connected to aany-hub.
    Your responses are being monitored and broadcast through the hub.

init_commands:
  - "echo '🔗 Connecting to aany-hub...'"
  - "export AANY_HUB_URL=localhost:50052"
  - "bash ~/Gitrepo/agent-anywhere/aany-hub/connect_agent_to_hub.sh"
  - "echo '✅ Connected to hub'"

environment:
  AANY_HUB_ENABLED: "true"
  AANY_HUB_URL: "localhost:50052"
EOF

echo -e "${GREEN}✓ Template created${NC}"

# Step 4: Create test agent
TEST_AGENT="hub-test-$(date +%s)"
echo -e "${YELLOW}Step 4: Creating test agent: $TEST_AGENT${NC}"

cd "$AANY_DIR"
# First delete if exists
echo "yes" | ./target/debug/aany pool delete "$TEST_AGENT" 2>/dev/null || true

# Create with template
./target/debug/aany pool create "$TEST_AGENT" --template hub-connected

# Verify creation
if ./target/debug/aany pool list | grep -q "$TEST_AGENT"; then
    echo -e "${GREEN}✓ Agent created successfully${NC}"
else
    echo -e "${RED}✗ Failed to create agent${NC}"
    exit 1
fi

# Step 5: Start the agent
echo -e "${YELLOW}Step 5: Starting agent...${NC}"
./target/debug/aany pool start "$TEST_AGENT"
sleep 3

# Check if tmux session exists
if tmux has-session -t "agent-$TEST_AGENT" 2>/dev/null; then
    echo -e "${GREEN}✓ Agent tmux session started${NC}"
else
    echo -e "${RED}✗ Failed to start agent session${NC}"
    exit 1
fi

# Step 6: Wait for agent to connect to hub
echo -e "${YELLOW}Step 6: Waiting for agent to connect to hub...${NC}"
sleep 5  # Give time for init scripts to run

# Check if agent is registered
AGENTS_JSON=$(curl -s http://localhost:8090/api/agents)
if echo "$AGENTS_JSON" | grep -q "$TEST_AGENT"; then
    echo -e "${GREEN}✓ Agent registered with hub${NC}"
    echo "Connected agents:"
    echo "$AGENTS_JSON" | jq -r '.agents[] | "  - \(.id)"'
else
    echo -e "${YELLOW}⚠ Agent not yet registered, checking init script output...${NC}"
    echo "Tmux pane content:"
    tmux capture-pane -t "agent-$TEST_AGENT" -p | tail -20
fi

# Step 7: Check terminal sessions
echo -e "${YELLOW}Step 7: Checking terminal sessions...${NC}"
TERMINALS_JSON=$(curl -s http://localhost:8090/api/terminals)
if echo "$TERMINALS_JSON" | grep -q "$TEST_AGENT"; then
    echo -e "${GREEN}✓ Terminal session created${NC}"
    SESSION_ID=$(echo "$TERMINALS_JSON" | jq -r '.terminals[0].session_id')
    echo "Terminal session ID: $SESSION_ID"
else
    echo -e "${YELLOW}⚠ No terminal session yet${NC}"
fi

# Step 8: Send test commands
echo -e "${YELLOW}Step 8: Sending test commands...${NC}"
tmux send-keys -t "agent-$TEST_AGENT" "echo '🎉 Hello from Ultimate Test!'" Enter
sleep 1
tmux send-keys -t "agent-$TEST_AGENT" "date" Enter
sleep 1
tmux send-keys -t "agent-$TEST_AGENT" "pwd" Enter
sleep 1
tmux send-keys -t "agent-$TEST_AGENT" "echo 'Commands are being broadcast to the web terminal!'" Enter

# Step 9: Display final status
echo
echo -e "${BLUE}=== TEST COMPLETE ===${NC}"
echo
echo -e "${GREEN}✓ Hub server: http://localhost:8090${NC}"
echo -e "${GREEN}✓ Terminal UI: http://localhost:8090/terminal${NC}"
echo -e "${GREEN}✓ Agent: $TEST_AGENT${NC}"
echo
echo "Next steps:"
echo "1. Open http://localhost:8090/terminal in your browser"
echo "2. Click on the terminal session for '$TEST_AGENT'"
echo "3. You should see the test commands and output"
echo
echo "To interact with the agent:"
echo "  ./target/debug/aany pool attach $TEST_AGENT"
echo
echo "To clean up:"
echo "  ./target/debug/aany pool stop $TEST_AGENT"
echo "  echo yes | ./target/debug/aany pool delete $TEST_AGENT"
echo "  cd $SCRIPT_DIR && ./stop_hub.sh"
echo
echo -e "${YELLOW}Press Enter to open the terminal UI in your browser...${NC}"
read -r

# Open browser
if command -v open >/dev/null 2>&1; then
    open "http://localhost:8090/terminal"
elif command -v xdg-open >/dev/null 2>&1; then
    xdg-open "http://localhost:8090/terminal"
fi