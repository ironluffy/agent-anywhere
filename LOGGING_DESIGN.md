# Agent Logging Design

## Overview
Capture and manage logs from agent tmux sessions for debugging and monitoring.

## Implementation Options

### 1. **TMux Logging**
```bash
# Enable logging for a tmux pane
tmux pipe-pane -t agent-research-bot -o 'cat >> ~/.aany/pool/agents/research-bot/logs/session.log'
```

### 2. **Structured Logging**
```
logs/
├── session.log          # Raw tmux output
├── agent.log           # Agent activity log
├── commands.log        # Commands executed
├── errors.log          # Error messages only
└── archive/            # Rotated logs
    └── 2024-01-15/
```

### 3. **Log Rotation**
- Daily rotation
- Compress old logs
- Clean up based on `config.yaml` retention settings

### 4. **Real-time Monitoring**
```bash
# Tail logs
aany pool logs research-bot --follow

# Filter logs
aany pool logs research-bot --grep "ERROR"

# View last N lines
aany pool logs research-bot -n 100
```

### 5. **Integration Points**

#### When Starting Agent
```rust
// In agent.rs start() method
Command::new("tmux")
    .args(&["pipe-pane", "-t", &session_name, "-o", 
           &format!("cat >> {}/session.log", log_dir)])
    .output()?;
```

#### Structured Event Logging
```rust
// Log agent events
let event = AgentEvent {
    timestamp: Utc::now(),
    event_type: "task_started",
    details: json!({
        "task": "Research AI safety",
        "agent": "research-bot"
    })
};
append_to_log(&event, "agent.log")?;
```

### 6. **Log Analysis Features**
- Parse session logs for errors
- Extract command history
- Generate activity reports
- Monitor resource usage

### 7. **Claude Integration Logs**
- Log all Claude API calls
- Track token usage
- Monitor response times
- Debug conversation flow

## Benefits
1. **Debugging** - See exactly what happened in a session
2. **Auditing** - Track all agent activities
3. **Monitoring** - Watch for errors or issues
4. **Analytics** - Understand agent behavior patterns
5. **Compliance** - Keep records of AI agent actions