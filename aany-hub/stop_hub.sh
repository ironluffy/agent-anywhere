#!/bin/bash

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

if [ -f .hub.pid ]; then
    PID=$(cat .hub.pid)
    if kill -0 $PID 2>/dev/null; then
        echo -e "${YELLOW}Stopping aany-hub (PID: $PID)...${NC}"
        kill $PID
        sleep 1
        
        # Force kill if still running
        if kill -0 $PID 2>/dev/null; then
            kill -9 $PID
        fi
        
        echo -e "${GREEN}✓ aany-hub stopped${NC}"
        rm .hub.pid
    else
        echo -e "${YELLOW}aany-hub is not running (stale PID file)${NC}"
        rm .hub.pid
    fi
else
    # Try to find process by name
    PIDS=$(pgrep -f "src.aany_hub.main")
    if [ ! -z "$PIDS" ]; then
        echo -e "${YELLOW}Found aany-hub process(es): $PIDS${NC}"
        echo "$PIDS" | xargs kill
        sleep 1
        echo -e "${GREEN}✓ aany-hub stopped${NC}"
    else
        echo -e "${YELLOW}aany-hub is not running${NC}"
    fi
fi