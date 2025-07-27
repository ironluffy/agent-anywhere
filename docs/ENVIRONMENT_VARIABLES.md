# Agent Anywhere Environment Variables

This document provides a comprehensive reference for all environment variables used in the Agent Anywhere system.

## Overview

Agent Anywhere uses environment variables at two levels:
1. **System-level variables**: Configure the Agent Anywhere infrastructure
2. **Agent-level variables**: Configure individual agents and their workspaces

## System-Level Environment Variables

These variables configure the Agent Anywhere system itself:

### Core Configuration

#### `AANY_REPO_PATH`
- **Type**: String (path)
- **Default**: None (auto-discovery)
- **Description**: Path to the agent-anywhere repository
- **Usage**: Only needed if scripts are not installed in standard locations
- **Example**: `/home/user/agent-anywhere`

The system searches for scripts in this order:
1. Environment variable override (e.g., `AANY_CLAUDE_WRAPPER`)
2. `$AANY_REPO_PATH/aany-tmux/` directory
3. System PATH (using `which` command)
4. Common installation paths:
   - `/usr/local/bin/`
   - `/opt/homebrew/bin/`
   - `~/.local/bin/`
   - `~/bin/`
5. Relative to current executable

### Logging Configuration

#### `AANY_HUB_URL`
- **Type**: String (URL)
- **Default**: `localhost:50052`
- **Description**: gRPC hub URL for centralized logging
- **Usage**: Where tmux loggers send their data
- **Example**: `hub.example.com:50052`

#### `AANY_LOGGING_DISABLED`
- **Type**: Boolean
- **Default**: `false`
- **Description**: Disable all logging functionality
- **Usage**: Set to `true` to disable tmux, interaction, and screenshot logging
- **Example**: `true`

### Script Path Overrides

These variables allow you to specify custom locations for Agent Anywhere scripts:

#### `AANY_CLAUDE_WRAPPER`
- **Type**: String (path)
- **Default**: Auto-discovered
- **Description**: Path to claude-with-return.sh wrapper script
- **Example**: `/opt/custom/claude-with-return.sh`

#### `AANY_TMUX_LOGGER`
- **Type**: String (path)
- **Default**: Auto-discovered
- **Description**: Path to tmux-logger.sh script
- **Example**: `/opt/custom/tmux-logger.sh`

#### `AANY_INTERACTION_LOGGER`
- **Type**: String (path)
- **Default**: Auto-discovered
- **Description**: Path to tmux-interaction-logger.sh script
- **Example**: `/opt/custom/tmux-interaction-logger.sh`

#### `AANY_SCREENSHOT_LOGGER`
- **Type**: String (path)
- **Default**: Auto-discovered
- **Description**: Path to tmux-screenshot-rotating-logger.sh script
- **Example**: `/opt/custom/tmux-screenshot-rotating-logger.sh`

### Internal Variables

These are set automatically by the system:

#### `GRPC_ENABLE_FORK_SUPPORT`
- **Type**: String
- **Default**: `1`
- **Description**: Enable gRPC fork support for logger processes

#### `GRPC_POLL_STRATEGY`
- **Type**: String
- **Default**: `poll`
- **Description**: gRPC polling strategy for logger processes

## Agent-Level Environment Variables

These variables are typically set in individual agent `.env` files located at `agents/<agent-name>/.env`:

### Repository Configuration

#### `GIT_REPO`
- **Type**: String (URL)
- **Default**: None
- **Description**: Git repository to clone when starting the agent
- **Usage**: If set, the repository is cloned into the agent's workspace on startup
- **Example**: `https://github.com/username/project.git`

### Custom Variables

You can add any custom environment variables to an agent's `.env` file, and they will be available in the agent's tmux session.

## Configuration Files

### Global Configuration

1. **System .env file**: Place a `.env` file in the directory where you run `aany` commands
2. **Example**: Copy `.env.example` to `.env` and modify as needed

### Agent Configuration

1. **Agent .env file**: Each agent can have its own `.env` file at `agents/<agent-name>/.env`
2. **Persistence**: Environment variables set in the Create Agent UI are saved to the pool configuration and reused for new agents

## Usage Examples

### Basic Setup

```bash
# Copy the example file
cp .env.example .env

# Edit with your configuration
vim .env

# No AANY_REPO_PATH needed if scripts are in PATH
# Just run aany commands normally
aany pool
```

### Custom Installation

```bash
# .env file
AANY_REPO_PATH=/opt/agent-anywhere
AANY_HUB_URL=logging.internal:50052
AANY_LOGGING_DISABLED=false
```

### Agent with Git Repository

```bash
# agents/my-agent/.env
GIT_REPO=https://github.com/myorg/myproject.git
API_KEY=secret-key
ENVIRONMENT=development
```

### Disable Logging

```bash
# .env file
AANY_LOGGING_DISABLED=true
```

## Troubleshooting

### Scripts Not Found

If you see warnings about scripts not being found:

1. Check if scripts are in your PATH: `which tmux-logger.sh`
2. Set `AANY_REPO_PATH` to your agent-anywhere directory
3. Use specific script overrides (e.g., `AANY_TMUX_LOGGER=/path/to/script`)

### Logging Issues

If logging isn't working:

1. Check `AANY_HUB_URL` is accessible
2. Ensure `AANY_LOGGING_DISABLED` is not set to `true`
3. Verify tmux logger scripts have execute permissions

### Environment Variables Not Available

If agent environment variables aren't working:

1. Ensure the `.env` file is in the correct location
2. Check file permissions
3. Verify syntax (KEY=value format, no spaces around =)

## Best Practices

1. **Use .env.example**: Always keep `.env.example` updated as documentation
2. **Don't commit .env**: Add `.env` to `.gitignore` to avoid exposing secrets
3. **Agent isolation**: Use agent-specific `.env` files to isolate configurations
4. **Path independence**: Prefer auto-discovery over hard-coded paths
5. **Logging in production**: Configure `AANY_HUB_URL` for centralized log collection