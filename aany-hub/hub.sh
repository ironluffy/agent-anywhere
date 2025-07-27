#!/bin/bash
# Agent Anywhere Hub Management

set -e

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Configuration
VENV_PATH="${AANY_HUB_VENV:-.venv}"
TMUX_SESSION="aany-hub"
GRPC_PORT="${AANY_HUB_GRPC_PORT:-50052}"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

start_server() {
    local WITH_DASHBOARD="$1"
    echo -e "${GREEN}Starting Agent Anywhere Hub...${NC}"
    
    # Check if already running
    if tmux has-session -t $TMUX_SESSION 2>/dev/null; then
        echo -e "${YELLOW}Hub is already running in tmux session: $TMUX_SESSION${NC}"
        echo "Use: $0 restart"
        exit 1
    fi
    
    # Check virtual environment
    if [ ! -d "$VENV_PATH" ]; then
        echo -e "${YELLOW}Creating virtual environment...${NC}"
        python3 -m venv "$VENV_PATH"
        source "$VENV_PATH/bin/activate"
        pip install -r requirements.txt
    fi
    
    # Create tmux session
    tmux new-session -d -s $TMUX_SESSION
    
    # Start server
    tmux send-keys -t $TMUX_SESSION "cd $(pwd)" C-m
    tmux send-keys -t $TMUX_SESSION "source $VENV_PATH/bin/activate" C-m
    # Use main server with dashboard by default, or simple server if --no-dashboard flag is used
    if [[ "$WITH_DASHBOARD" == "--no-dashboard" ]]; then
        tmux send-keys -t $TMUX_SESSION "cd src && python -m aany_hub.simple_server" C-m
    else
        tmux send-keys -t $TMUX_SESSION "python -m src.aany_hub.main" C-m
        DASHBOARD_PORT="${AANY_HUB_HTTP_PORT:-8080}"
    fi
    
    # Wait and check if server started
    sleep 3
    if lsof -i :$GRPC_PORT > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Hub started successfully!${NC}"
        echo -e "   gRPC endpoint: localhost:$GRPC_PORT"
        echo -e "   Logs stored in: ${AANY_HUB_STORAGE_PATH:-./logs}"
        if [[ "$WITH_DASHBOARD" != "--no-dashboard" ]]; then
            echo -e "   Web dashboard: http://localhost:$DASHBOARD_PORT"
        fi
        echo -e "   View logs: tmux attach -t $TMUX_SESSION"
    else
        echo -e "${RED}❌ Failed to start hub${NC}"
        echo "Check logs: tmux attach -t $TMUX_SESSION"
        exit 1
    fi
}

stop_server() {
    echo -e "${YELLOW}Stopping Agent Anywhere Hub...${NC}"
    
    # Kill tmux session
    if tmux has-session -t $TMUX_SESSION 2>/dev/null; then
        tmux kill-session -t $TMUX_SESSION
    fi
    
    # Kill any remaining processes on the port
    if lsof -i :$GRPC_PORT > /dev/null 2>&1; then
        echo "Cleaning up processes on port $GRPC_PORT..."
        lsof -ti :$GRPC_PORT | xargs kill -9 2>/dev/null || true
    fi
    
    echo -e "${GREEN}✅ Hub stopped${NC}"
}

status_server() {
    echo -e "${GREEN}Agent Anywhere Hub Status${NC}"
    echo "========================"
    
    # Check tmux session
    if tmux has-session -t $TMUX_SESSION 2>/dev/null; then
        echo -e "Tmux session: ${GREEN}Running${NC}"
    else
        echo -e "Tmux session: ${RED}Not running${NC}"
    fi
    
    # Check port
    if lsof -i :$GRPC_PORT > /dev/null 2>&1; then
        echo -e "gRPC server: ${GREEN}Listening on port $GRPC_PORT${NC}"
        echo -e "Process:"
        lsof -i :$GRPC_PORT | grep LISTEN | head -1
    else
        echo -e "gRPC server: ${RED}Not listening${NC}"
    fi
    
    # Check logs directory
    LOG_DIR="${AANY_HUB_STORAGE_PATH:-./logs}"
    if [ -d "$LOG_DIR" ]; then
        AGENT_COUNT=$(find "$LOG_DIR" -type d -mindepth 1 -maxdepth 1 | wc -l | tr -d ' ')
        echo -e "\nLog storage: $LOG_DIR"
        echo -e "Agents logged: $AGENT_COUNT"
    fi
}

case "${1:-}" in
    start)
        start_server "$2"
        ;;
    stop)
        stop_server
        ;;
    restart)
        stop_server
        sleep 1
        start_server "$2"
        ;;
    status)
        status_server
        ;;
    attach)
        if tmux has-session -t $TMUX_SESSION 2>/dev/null; then
            tmux attach -t $TMUX_SESSION
        else
            echo -e "${RED}Hub is not running${NC}"
            exit 1
        fi
        ;;
    *)
        echo "Agent Anywhere Hub Management"
        echo ""
        echo "Usage: $0 {start|stop|restart|status|attach} [options]"
        echo ""
        echo "Commands:"
        echo "  start [--no-dashboard]   - Start the hub server (dashboard on by default)"
        echo "  stop                     - Stop the hub server"
        echo "  restart [--no-dashboard] - Restart the hub server"
        echo "  status                   - Show hub status"
        echo "  attach                   - Attach to hub tmux session"
        echo ""
        echo "Options:"
        echo "  --no-dashboard - Start without web dashboard (gRPC only)"
        echo ""
        echo "Default: Hub starts with web dashboard at http://localhost:8080"
        exit 1
        ;;
esac