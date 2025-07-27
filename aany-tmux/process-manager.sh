#!/bin/bash
# Process management utilities for Agent Anywhere background tasks

AANY_HOME="${HOME}/.aany"
PROCESS_DIR="${AANY_HOME}/processes"
mkdir -p "$PROCESS_DIR"

# Function to start a managed process
start_process() {
    local name="$1"
    local command="$2"
    local agent_id="${3:-unknown}"
    local session_name="${4:-unknown}"
    
    # Generate unique process ID
    local proc_id="${name}_$(date +%s)_$$"
    local pid_file="${PROCESS_DIR}/${proc_id}.pid"
    local meta_file="${PROCESS_DIR}/${proc_id}.json"
    
    # Start the process in background
    eval "$command" &
    local pid=$!
    
    # Save PID
    echo $pid > "$pid_file"
    
    # Save metadata
    cat > "$meta_file" << EOF
{
  "process_id": "$proc_id",
  "name": "$name",
  "pid": $pid,
  "command": "$command",
  "agent_id": "$agent_id",
  "session_name": "$session_name",
  "started_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "status": "running"
}
EOF
    
    echo "Started process: $name (PID: $pid, ID: $proc_id)"
    return 0
}

# Function to stop a process
stop_process() {
    local identifier="$1"  # Can be PID, process_id, or name pattern
    
    # Find matching processes
    for meta_file in "$PROCESS_DIR"/*.json; do
        [ -f "$meta_file" ] || continue
        
        local matches=0
        local pid=$(jq -r '.pid' "$meta_file" 2>/dev/null)
        local name=$(jq -r '.name' "$meta_file" 2>/dev/null)
        local proc_id=$(jq -r '.process_id' "$meta_file" 2>/dev/null)
        
        # Check if identifier matches
        if [[ "$pid" == "$identifier" ]] || [[ "$proc_id" == "$identifier" ]] || [[ "$name" =~ "$identifier" ]]; then
            matches=1
        fi
        
        if [ $matches -eq 1 ] && [ -n "$pid" ]; then
            if kill -0 "$pid" 2>/dev/null; then
                kill "$pid"
                echo "Stopped process: $name (PID: $pid)"
                
                # Update status
                local temp_file=$(mktemp)
                jq '.status = "stopped" | .stopped_at = "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'"' "$meta_file" > "$temp_file"
                mv "$temp_file" "$meta_file"
            else
                echo "Process $name (PID: $pid) is not running"
            fi
        fi
    done
}

# Function to list processes
list_processes() {
    local filter="${1:-all}"  # all, running, stopped, agent:ID, session:NAME
    
    echo "Agent Anywhere Managed Processes:"
    echo "================================"
    
    for meta_file in "$PROCESS_DIR"/*.json; do
        [ -f "$meta_file" ] || continue
        
        local name=$(jq -r '.name' "$meta_file" 2>/dev/null)
        local pid=$(jq -r '.pid' "$meta_file" 2>/dev/null)
        local agent_id=$(jq -r '.agent_id' "$meta_file" 2>/dev/null)
        local session=$(jq -r '.session_name' "$meta_file" 2>/dev/null)
        local status="stopped"
        
        # Check if process is running
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            status="running"
        fi
        
        # Apply filter
        case "$filter" in
            "running")
                [ "$status" != "running" ] && continue
                ;;
            "stopped")
                [ "$status" != "stopped" ] && continue
                ;;
            agent:*)
                local filter_agent="${filter#agent:}"
                [ "$agent_id" != "$filter_agent" ] && continue
                ;;
            session:*)
                local filter_session="${filter#session:}"
                [ "$session" != "$filter_session" ] && continue
                ;;
        esac
        
        printf "%-20s PID: %-8s Agent: %-10s Session: %-15s Status: %s\n" \
            "$name" "$pid" "$agent_id" "$session" "$status"
    done
}

# Function to clean up dead processes
cleanup_processes() {
    echo "Cleaning up dead processes..."
    local cleaned=0
    
    for meta_file in "$PROCESS_DIR"/*.json; do
        [ -f "$meta_file" ] || continue
        
        local pid=$(jq -r '.pid' "$meta_file" 2>/dev/null)
        local pid_file="${meta_file%.json}.pid"
        
        if [ -n "$pid" ] && ! kill -0 "$pid" 2>/dev/null; then
            # Process is dead, archive it
            local archive_dir="${PROCESS_DIR}/archive"
            mkdir -p "$archive_dir"
            
            mv "$meta_file" "$archive_dir/" 2>/dev/null
            [ -f "$pid_file" ] && mv "$pid_file" "$archive_dir/" 2>/dev/null
            
            ((cleaned++))
        fi
    done
    
    echo "Cleaned up $cleaned dead process entries"
}

# Function to stop all processes for an agent or session
stop_all() {
    local type="$1"  # agent or session
    local identifier="$2"
    
    echo "Stopping all processes for $type: $identifier"
    
    for meta_file in "$PROCESS_DIR"/*.json; do
        [ -f "$meta_file" ] || continue
        
        local matches=0
        if [ "$type" == "agent" ]; then
            local agent_id=$(jq -r '.agent_id' "$meta_file" 2>/dev/null)
            [ "$agent_id" == "$identifier" ] && matches=1
        elif [ "$type" == "session" ]; then
            local session=$(jq -r '.session_name' "$meta_file" 2>/dev/null)
            [ "$session" == "$identifier" ] && matches=1
        fi
        
        if [ $matches -eq 1 ]; then
            local pid=$(jq -r '.pid' "$meta_file" 2>/dev/null)
            stop_process "$pid"
        fi
    done
}

# Main command handler
case "${1:-help}" in
    start)
        shift
        start_process "$@"
        ;;
    stop)
        stop_process "$2"
        ;;
    list)
        list_processes "${2:-all}"
        ;;
    cleanup)
        cleanup_processes
        ;;
    stop-all)
        stop_all "$2" "$3"
        ;;
    *)
        echo "Usage: $0 {start|stop|list|cleanup|stop-all}"
        echo ""
        echo "Commands:"
        echo "  start <name> <command> [agent_id] [session]  - Start a managed process"
        echo "  stop <pid|name|id>                           - Stop a process"
        echo "  list [all|running|stopped|agent:ID|session:NAME] - List processes"
        echo "  cleanup                                      - Clean up dead process entries"
        echo "  stop-all <agent|session> <identifier>        - Stop all processes for agent/session"
        ;;
esac