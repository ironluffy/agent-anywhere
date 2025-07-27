#!/bin/bash
# Create a tmux agent with logging enabled by default

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
HUB_URL="localhost:50052"
LOGGING_ENABLED=true

# Parse arguments
AGENT_NAME="$1"
if [ -z "$AGENT_NAME" ]; then
    echo -e "${RED}Error: Agent name required${NC}"
    echo "Usage: $0 <agent-name> [--no-logging] [--hub-url URL]"
    exit 1
fi

# Process additional arguments
shift
while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        --no-logging)
            LOGGING_ENABLED=false
            shift
            ;;
        --hub-url)
            HUB_URL="$2"
            shift
            shift
            ;;
        *)
            echo -e "${YELLOW}Unknown option: $1${NC}"
            shift
            ;;
    esac
done

SESSION_NAME="agent-${AGENT_NAME}"

# Check if session already exists
if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    echo -e "${RED}Error: Session '$SESSION_NAME' already exists${NC}"
    exit 1
fi

# Check if hub is running (only if logging is enabled)
if [ "$LOGGING_ENABLED" = true ]; then
    echo -e "${YELLOW}Checking hub connectivity...${NC}"
    if ! nc -z localhost 50052 2>/dev/null; then
        echo -e "${YELLOW}Warning: Hub not reachable at $HUB_URL${NC}"
        echo -e "${YELLOW}Start the hub with: cd aany-hub && ./run_server.sh${NC}"
        read -p "Continue anyway? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
fi

# Create tmux session
echo -e "${GREEN}Creating agent session: $SESSION_NAME${NC}"
tmux new-session -d -s "$SESSION_NAME"

# Add protection warning
WARNING="clear && echo -e '\033[1;33m
╔══════════════════════════════════════╗
║  ⚠️  AGENT CONTROLLED SESSION  ⚠️   ║
║                                      ║
║  Session: $SESSION_NAME
║  Agent ID: $AGENT_NAME
║  Logging: $LOGGING_ENABLED
║  Hub URL: $HUB_URL
║                                      ║
║  This tmux session is managed by     ║
║  an automated agent.                 ║
║                                      ║
║  Manual changes may disrupt agent    ║
║  operations!                         ║
╚══════════════════════════════════════╝
\033[0m'"

tmux send-keys -t "$SESSION_NAME" "$WARNING" Enter

# Add a marker for agent configuration
if [ "$LOGGING_ENABLED" = true ]; then
    CONFIG="# Agent configuration
export AANY_AGENT_ID='$AGENT_NAME'
export AANY_HUB_URL='$HUB_URL'
export AANY_LOGGING_ENABLED=true
echo 'Agent $AGENT_NAME initialized with logging to $HUB_URL'"
else
    CONFIG="# Agent configuration
export AANY_AGENT_ID='$AGENT_NAME'
export AANY_LOGGING_ENABLED=false
echo 'Agent $AGENT_NAME initialized without logging'"
fi

tmux send-keys -t "$SESSION_NAME" "$CONFIG" Enter

# Success message
echo -e "${GREEN}✅ Agent created successfully!${NC}"
echo -e "${GREEN}Session: $SESSION_NAME${NC}"
echo -e "${GREEN}Agent ID: $AGENT_NAME${NC}"

if [ "$LOGGING_ENABLED" = true ]; then
    echo -e "${GREEN}📊 Logging enabled to: $HUB_URL${NC}"
else
    echo -e "${YELLOW}📊 Logging disabled${NC}"
fi

echo
echo -e "${GREEN}Commands:${NC}"
echo "  Monitor (read-only): tmux attach -r -t $SESSION_NAME"
echo "  Attach (interactive): tmux attach -t $SESSION_NAME"
echo "  Kill: tmux kill-session -t $SESSION_NAME"
echo
echo -e "${GREEN}Or use aany CLI:${NC}"
echo "  Monitor: aany tmux monitor $AGENT_NAME"
echo "  Health: aany tmux health $AGENT_NAME"
echo "  Kill: aany tmux kill $AGENT_NAME"