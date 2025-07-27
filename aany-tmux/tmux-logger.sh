#!/bin/bash
# Tmux Logger - Sends tmux session activity to aany-hub

SESSION_NAME="$1"
AGENT_ID="${2:-$SESSION_NAME}"
HUB_URL="${AANY_HUB_URL:-localhost:50052}"

if [ -z "$SESSION_NAME" ]; then
    echo "Usage: $0 <session-name> [agent-id]"
    exit 1
fi

# Python script to send logs
cat << 'EOF' > /tmp/tmux_logger_${SESSION_NAME}.py
import asyncio
import grpc
import sys
import os
import time
from datetime import datetime

# Add hub path
repo_path = os.environ.get('AANY_REPO_PATH')
if not repo_path:
    print("Error: AANY_REPO_PATH environment variable must be set", file=sys.stderr)
    sys.exit(1)
sys.path.append(f'{repo_path}/aany-hub/src')
from aany_hub import agent_pb2, agent_pb2_grpc

SESSION = sys.argv[1]
AGENT_ID = sys.argv[2]
HUB_URL = sys.argv[3]

async def monitor_and_log():
    channel = grpc.aio.insecure_channel(HUB_URL)
    stub = agent_pb2_grpc.AgentHubStub(channel)
    
    # Register agent
    agent_info = agent_pb2.AgentInfo(
        agent_id=AGENT_ID,
        version="1.0.0",
        hostname=os.uname().nodename,
        capabilities={"type": "tmux-session"}
    )
    
    try:
        await stub.RegisterAgent(agent_info)
        print(f"✅ Agent {AGENT_ID} registered with hub")
    except Exception as e:
        print(f"Failed to register: {e}")
        return
    
    # Monitor tmux pane content
    last_content = ""
    while True:
        try:
            # Capture pane content
            import subprocess
            result = subprocess.run(
                ['tmux', 'capture-pane', '-t', SESSION, '-p'],
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                content = result.stdout
                if content != last_content:
                    # Find what changed
                    new_lines = content[len(last_content):].strip()
                    if new_lines:
                        # Split into individual lines and send each
                        log_entries = []
                        for line in new_lines.split('\n'):
                            if line.strip():
                                # Send as log
                                log_entry = agent_pb2.LogEntry(
                                    agent_id=AGENT_ID,
                                    timestamp=datetime.utcnow().isoformat(),
                                    level="INFO",
                                    message=line.strip(),
                                    component="tmux",
                                    metadata={"session": SESSION}
                                )
                                log_entries.append(log_entry)
                        
                        if log_entries:
                            await stub.SendLogs(iter(log_entries))
                    
                    last_content = content
            
            await asyncio.sleep(1)
            
        except KeyboardInterrupt:
            break
        except Exception as e:
            print(f"Error: {e}")
            await asyncio.sleep(5)

if __name__ == "__main__":
    asyncio.run(monitor_and_log())
EOF

# Run the logger in background
echo "🚀 Starting tmux logger for session: $SESSION_NAME"
echo "📊 Sending logs to: $HUB_URL"
echo "🔌 Agent ID: $AGENT_ID"

# Set environment to suppress gRPC fork warnings
export GRPC_ENABLE_FORK_SUPPORT=1
export GRPC_POLL_STRATEGY=poll

if [ -z "$AANY_REPO_PATH" ]; then
    echo "Error: AANY_REPO_PATH environment variable must be set" >&2
    exit 1
fi
source "$AANY_REPO_PATH/aany-hub/.venv/bin/activate"
python /tmp/tmux_logger_${SESSION_NAME}.py "$SESSION_NAME" "$AGENT_ID" "$HUB_URL" 2>/dev/null &
LOGGER_PID=$!

echo "✅ Logger started with PID: $LOGGER_PID"
echo "To stop: kill $LOGGER_PID"