# aany - Agent Anywhere CLI

Unified command-line interface for the Agent Anywhere system.

## Installation

```bash
# Build and install
cargo install --path .

# Or from the workspace root
cargo install --path aany
```

## Usage

### TMux Commands

Manage individual tmux sessions for agents:

```bash
# Create new session
aany tmux new my-bot
aany tmux new my-bot --detach

# Attach to session
aany tmux attach my-bot
aany tmux attach my-bot --readonly

# Monitor (read-only)
aany tmux monitor my-bot

# List sessions
aany tmux list

# Session management
aany tmux health my-bot
aany tmux cleanup my-bot
aany tmux kill my-bot
```

### Pool Commands

Manage multiple agents in a pool:

```bash
# List all agents
aany pool list
aany pool list --detailed

# Create agent
aany pool create research-bot
aany pool create code-bot --template code-reviewer --start

# Start/stop agents
aany pool start research-bot
aany pool start research-bot --attach
aany pool stop research-bot

# Attach/monitor
aany pool attach research-bot
aany pool monitor research-bot

# Status and logs
aany pool status
aany pool status research-bot
aany pool logs research-bot
aany pool logs research-bot --follow

# Delete agent
aany pool delete research-bot --force

# Interactive TUI
aany pool navigate
```

### Other Commands

```bash
# Initialize agent workspace
aany init my-project --template research

# Configuration
aany config
aany config model claude-3-opus
aany config pool.root ~/my-agents
```

## Environment Variables

- `AGENT_POOL_ROOT`: Root directory for agent pool (default: `~/agent-pool`)
- `RUST_LOG`: Logging level (e.g., `debug`, `info`)

## Aliases

For convenience, add these to your shell:

```bash
alias at='aany tmux'    # Agent Tmux
alias ap='aany pool'    # Agent Pool

# Usage
at new my-bot
ap list
```