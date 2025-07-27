# Agent Directory Structure

## Flat Structure Implementation

All agent-related files are now organized under a single directory per agent:

```
~/.aany/
└── agents/
    └── {agent-name}/
        ├── workspace/         # Working directory (agent starts here)
        ├── logs/              # All log files
        │   ├── tmux_interaction_*.log
        │   ├── session_*.log
        │   └── commands_*.log
        ├── metadata/          # Agent metadata
        │   ├── agent.yaml     # Agent configuration
        │   └── claude.json    # Claude AI settings
        └── config/            # Agent-specific configs
            ├── .env           # Environment variables (optional)
            ├── init.sh        # Initialization script (optional)
            └── system.prompt  # System prompt (optional)
```

## Benefits

1. **Simple Management**: Delete agent = `rm -rf ~/.aany/agents/{name}`
2. **Clear Organization**: Everything in one place
3. **No Orphaned Files**: No scattered directories
4. **Easy Backup**: Copy one directory to backup entire agent
5. **Intuitive Paths**: Workspace is clearly a subdirectory

## Path Examples

- **Agent Root**: `~/.aany/agents/my-agent/`
- **Workspace**: `~/.aany/agents/my-agent/workspace/`
- **Logs**: `~/.aany/agents/my-agent/logs/`
- **Metadata**: `~/.aany/agents/my-agent/metadata/agent.yaml`
- **Claude Config**: `~/.aany/agents/my-agent/metadata/claude.json`

## Migration

The system automatically handles old structures:
- Old: `~/.aany/agents/{name}/.agent.yaml`
- New: `~/.aany/agents/{name}/metadata/agent.yaml`

## Logging

Logs are organized by type in the logs directory:
- `tmux_interaction_*.log` - User input and screen content
- `session_*.log` - Full session transcript
- `commands_*.log` - Command history

All logs include timestamps and are rotated daily.