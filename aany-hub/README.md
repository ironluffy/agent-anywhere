# aany-hub

Centralized logging and management platform for Agent Anywhere.

## MVP Features

- Agent connection via gRPC
- Log collection from tmux agents
- Basic storage (local file / S3)
- Simple web dashboard

## Quick Start

```bash
# Start the hub
./hub.sh start

# Check status
./hub.sh status

# Stop the hub
./hub.sh stop

# Restart the hub
./hub.sh restart

# View logs (attach to tmux session)
./hub.sh attach
```

## Development

```bash
# Activate virtual environment
source .venv/bin/activate

# Install new dependencies
pip install <package>
pip freeze > requirements.txt

# Run tests
pytest

# Format code
black src/
```

## Agent Connection

### Default Behavior: Logging Enabled

By default, all agents created with Agent Anywhere will automatically log to the hub:

```bash
# Create agent with logging enabled (default)
./create-agent.sh my-agent

# Create agent without logging (opt-out)
./create-agent.sh my-agent --no-logging

# Create agent with custom hub URL
./create-agent.sh my-agent --hub-url remote-hub:50052
```

### Using the CLI

```bash
# Create modular agent with logging (default)
aany tmux agent my-agent

# Create without logging
aany tmux agent my-agent --no-logging

# Monitor agent logs in real-time
aany tmux monitor my-agent
```

### Manual Agent Creation

Agents connect using gRPC and send logs with their identifier:

```bash
# Agent connects with identifier
aany pool start my-agent --hub-url localhost:50052 --agent-id my-agent-001
```

## Configuration

```yaml
# config.yaml
server:
  grpc_port: 50052
  http_port: 8080

storage:
  type: local  # or 's3'
  local_path: ./logs
  s3_bucket: aany-hub-logs

logging:
  level: INFO
  format: json
```