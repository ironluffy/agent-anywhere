# Agent Integration Guide

## Default Behavior: Logging is Enabled

**As of the latest version, all agents created with Agent Anywhere have logging enabled by default.** This ensures comprehensive monitoring and debugging capabilities out of the box.

## How to Connect Agents to aany-hub

### 1. Quick Start (Logging Enabled by Default)

```bash
# Create agent - logging is automatic
./create-agent.sh my-agent

# Or using the CLI
aany tmux agent my-agent
```

To opt-out of logging:
```bash
# Create agent without logging
./create-agent.sh my-agent --no-logging
```

### 2. Manual Agent Configuration

If creating agents programmatically:

```bash
# Start agent with hub connection (default behavior)
aany pool start my-agent \
  --hub-url localhost:50052 \
  --agent-id my-agent-001
```

### 2. What Gets Logged

When connected to hub, agents automatically send:
- Tmux session events (create, destroy, command execution)
- Agent lifecycle events (start, stop, restart)
- Task execution logs
- Error messages and stack traces

### 3. Log Format

Each log entry contains:
```json
{
  "agent_id": "my-agent-001",
  "timestamp": "2025-01-26T12:34:56Z",
  "level": "INFO",
  "component": "tmux",
  "message": "Executed command: python script.py",
  "metadata": {
    "session": "agent-main",
    "window": "0",
    "pane": "1"
  }
}
```

### 4. Hub Dashboard

View logs in real-time at: http://localhost:8080

### 5. Storage

Logs are stored in:
- Local: `./logs/{agent_id}/{date}.jsonl`
- S3: `s3://bucket/agent-logs/{agent_id}/{year}/{month}/{day}/`