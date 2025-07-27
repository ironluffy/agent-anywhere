# Logging Changes Summary

## Key Changes Made

### 1. Clean Agent Startup
- **REMOVED**: All commands sent to new tmux sessions
- **REMOVED**: Welcome messages, environment exports, echo statements
- Agents now start with a clean terminal ready for use

### 2. New Logging Control
- **REMOVED**: `AANY_LOGGING_ENABLED` variable
- **ADDED**: `AANY_LOGGING_DISABLED` variable
- Logging is now **enabled by default**
- To disable logging: `export AANY_LOGGING_DISABLED=true`

### 3. Directory Structure
```
~/.aany/
├── agents/
│   └── {agent-name}/
│       ├── .agent.yaml      (metadata)
│       ├── .claude/          (settings)
│       └── logs/             (all logs here)
└── workspaces/
    └── {agent-name}/         (working directory)
```

## Usage

### Enable Logging (Default)
```bash
# Just create the agent - logging is automatic
./target/release/aany pool
# or
./target/release/aany tmux new my-agent
```

### Disable Logging
```bash
export AANY_LOGGING_DISABLED=true
./target/release/aany pool
```

### Check Logs
```bash
# Logs are in agent-specific directory
ls ~/.aany/agents/{agent-name}/logs/
```

## What Gets Logged

When logging is enabled (default):
- All tmux screen content
- User keyboard input
- Command execution
- Session lifecycle events
- Agent metadata

## Clean Startup

Agents now start with:
- Working directory set to `~/.aany/workspaces/{agent-name}`
- No welcome messages
- No environment variable exports visible in terminal
- Clean prompt ready for immediate use