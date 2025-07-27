# Getting Started with Agent Anywhere

## Prerequisites

- Python 3.10+ or Homebrew
- tmux (required for agent management)
- Git (for cloning agent repositories)

## Installation

Choose one of the following methods:

### Option 1: pip (Recommended)
```bash
pip install agent-anywhere
```

### Option 2: Homebrew
```bash
brew install agent-anywhere
```

### Option 3: From Source
```bash
git clone https://github.com/your-org/agent-anywhere.git
cd agent-anywhere
cargo build --release
```

## Quick Start

### 1. Launch the Agent Pool Manager

```bash
aany pool
```

This opens an interactive UI where you can:
- Create new agents
- Start/stop existing agents
- Monitor agent status
- Configure environment variables

### 2. Create Your First Agent

In the pool manager UI:

1. Press `n` to create a new agent
2. Enter a name (or press Enter for auto-generated name)
3. Enter a description for your agent
4. Configure environment variables:
   - Press `e` to add/edit variables
   - Common example: `GIT_REPO=https://github.com/user/project.git`
5. Press `c` to create the agent

### 3. Start the Agent

1. Select your agent with arrow keys
2. Press `s` to start the agent
3. Press `a` to attach to the agent's tmux session

### 4. Working with Agents

Once attached to an agent:
- The agent runs in a tmux session
- You can interact with it like a normal terminal
- Press `Ctrl-b d` to detach from the session
- The agent continues running in the background

## Configuration

### Global Configuration

Create a `.env` file in your working directory:

```bash
# Optional: Specify agent-anywhere repo path if scripts aren't in PATH
# AANY_REPO_PATH=/path/to/agent-anywhere

# Optional: Configure logging hub
# AANY_HUB_URL=localhost:50052

# Optional: Disable logging
# AANY_LOGGING_DISABLED=true
```

### Agent-Specific Configuration

Each agent can have its own environment variables:
- Set them during creation in the UI
- Or edit `agents/<agent-name>/.env` directly

Example agent `.env`:
```bash
GIT_REPO=https://github.com/myorg/myproject.git
API_KEY=your-api-key
DATABASE_URL=postgresql://localhost/mydb
```

## Common Workflows

### Clone and Setup a Project

1. Create an agent with `GIT_REPO` environment variable
2. Start the agent - it will automatically clone the repository
3. The agent's workspace will contain your project

### Persistent Environment Variables

The system remembers your last used environment variables:
- They appear as defaults when creating new agents
- Edit them with `e` in the creation UI
- Remove with `r` to reset

### Managing Multiple Agents

- Run multiple agents simultaneously
- Each agent has isolated workspace and environment
- Monitor all agents from the pool manager UI
- Agents persist across system restarts

## Troubleshooting

### Agent Won't Start

1. Check tmux is installed: `which tmux`
2. Verify no existing tmux session: `tmux ls`
3. Check logs in `agents/<agent-name>/logs/`

### Scripts Not Found

If you see warnings about missing scripts:
1. Set `AANY_REPO_PATH` in your `.env` file
2. Or ensure scripts are in your PATH
3. See [Environment Variables Guide](ENVIRONMENT_VARIABLES.md) for details

### Can't Clone Repository

1. Verify Git is installed: `which git`
2. Check repository URL is correct
3. Ensure you have access (SSH keys, tokens, etc.)

## Next Steps

- Explore [agent templates](TEMPLATES.md) for pre-configured setups
- Read the [Environment Variables Guide](ENVIRONMENT_VARIABLES.md) for advanced configuration
- Join our community for support and sharing agent templates