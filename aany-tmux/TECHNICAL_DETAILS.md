# Technical Details

This document covers the technical implementation details of tmux-agent, including safety features, monitoring capabilities, and architecture decisions.

## Table of Contents
- [Human Interference Handling](#human-interference-handling)
- [Read-Only Monitoring](#read-only-monitoring)
- [Safety Features](#safety-features)
- [TMux Compatibility](#tmux-compatibility)

---

## Human Interference Handling

### What Happens When Humans Enter Agent-Controlled Sessions?

When a human manually interacts with a tmux window managed by the agent proxy, several issues can arise:

#### 1. Structural Changes
- **Pane Splits**: Human creates new panes, shrinking existing ones
- **Pane Kills**: Human closes panes the agent is using
- **Window Changes**: Human renames windows or creates new ones

#### 2. State Disruption
- **Active Pane Switch**: Human switches to different pane
- **Unexpected Input**: Human types commands in agent's pane
- **Content Modification**: Human clears or modifies pane content

#### 3. Session Attachment
- Multiple clients attached simultaneously
- Human watching agent operations in real-time
- Potential for accidental interference

### Detection Mechanisms

The `InterferenceDetector` monitors for:

```rust
pub enum InterferenceEvent {
    PaneCountChanged { before: u32, after: u32 },
    ActivePaneChanged { before: u32, after: u32 },
    PaneDimensionsChanged { pane: u32, before: (u32, u32), after: (u32, u32) },
    UnexpectedInput { content_snippet: String },
    PaneKilled { pane: u32 },
    WindowRenamed { before: String, after: String },
    HumanAttached { client_info: String },
}
```

### Recovery Strategies

#### Automatic Recovery
1. **Cleanup Extra Panes**: Remove all but the primary pane
2. **Restore Active Pane**: Switch back to agent's pane
3. **Clear Unexpected Input**: Reset pane content
4. **Re-display Warning**: Show "Agent Controlled" message

### Prevention Strategies

#### Visual Warnings
```
╔══════════════════════════════════════╗
║  ⚠️  AGENT CONTROLLED SESSION  ⚠️   ║
║                                      ║
║  This tmux session is managed by     ║
║  an automated agent.                 ║
║                                      ║
║  Manual changes may disrupt agent    ║
║  operations!                         ║
╚══════════════════════════════════════╝
```

---

## Read-Only Monitoring

### Yes, Read-Only Monitoring is Safe! 👁️

When humans attach with the `-r` flag (`tmux attach -r -t session-name`), they enter **read-only mode** which:

- ✅ **Cannot type** or send any input
- ✅ **Cannot split/kill panes** 
- ✅ **Cannot interfere** with agent operations
- ✅ **Can only watch** what's happening

### How the System Handles Read-Only Mode

#### Detection
The system differentiates between attachment types:
```rust
InterferenceEvent::HumanAttached { client_info, read_only: true }  // Safe!
InterferenceEvent::HumanAttached { client_info, read_only: false } // May interfere
```

#### Visual Feedback for Monitoring
```
╔══════════════════════════════════════╗
║  👁️  MONITORING MODE DETECTED  👁️   ║
║                                      ║
║  Welcome! You are in read-only mode. ║
║  You can safely watch the agent      ║
║  without disrupting operations.      ║
╚══════════════════════════════════════╝
```

### Usage Examples

```bash
# Safe monitoring (recommended)
tmux attach -r -t agent-session
tmux-agent monitor agent-session  # Convenience wrapper

# Interactive mode (can interfere)
tmux attach -t agent-session  # AVOID for monitoring
```

### Benefits of Read-Only Monitoring

1. **Zero Interference**: Agent continues uninterrupted
2. **Real-Time Visibility**: See exactly what agent is doing
3. **Debugging**: Watch agent behavior without affecting it
4. **Multiple Monitors**: Many people can watch simultaneously
5. **No Recovery Needed**: Agent doesn't need to handle interference

---

## Safety Features

### Core Safety Mechanisms

1. **Session Protection**
   - Clear visual warnings when entering agent sessions
   - Automatic detection of human interference
   - Recovery mechanisms to restore agent control

2. **Pane Management**
   - Automatic cleanup of extra panes
   - Protection against pane proliferation
   - Maintains single working pane for agent

3. **State Recovery**
   - Restores active pane when changed
   - Clears unexpected human input
   - Re-displays warning messages

### Configuration Options

```rust
let mut config = SafetyConfig::default();
config.allow_readonly_monitoring = true;  // Default: true
config.auto_cleanup_panes = true;        // Default: true
config.show_warnings = true;             // Default: true
```

---

## TMux Compatibility

### Supported Commands

tmux-agent supports standard tmux syntax for familiarity:

```bash
# Session management
tmux-agent new-session -s name
tmux-agent attach-session -t name
tmux-agent list-sessions
tmux-agent kill-session -t name

# Agent-specific extensions
tmux-agent monitor name    # Read-only attachment
tmux-agent health name     # Health check
tmux-agent cleanup name    # Clean extra panes
```

### TMux Flags

The `-r` flag is crucial for safe monitoring:
- `-r` is an alias for `-f read-only,ignore-size`
- Prevents ALL input from affecting the session
- Allows multiple simultaneous read-only viewers

### Best Practices

1. **Isolation**: Use dedicated tmux sessions for agents
2. **Naming**: Use clear names like `agent-worker-1` or `bot-session`
3. **Monitoring**: Always use `-r` flag or `monitor` command
4. **Documentation**: Label agent sessions clearly
5. **Permissions**: Consider using separate user accounts for agents

### Emergency Recovery

If an agent loses control completely:

```bash
# List agent sessions
tmux list-sessions | grep agent

# Kill specific session
tmux kill-session -t agent-session

# Kill all panes except first
tmux list-panes -t session | tail -n +2 | cut -d: -f1 | xargs -I{} tmux kill-pane -t session:{}
```

---

## Architecture Overview

### Components

1. **TmuxProxy** (lib.rs)
   - Core tmux interaction layer
   - Raw command execution
   - Basic session management

2. **SafeTmuxProxy** (safe_proxy.rs)
   - Safety wrapper around TmuxProxy
   - Implements all protection mechanisms
   - Handles interference detection and recovery

3. **InterferenceDetector** (interference.rs)
   - Monitors for human interference
   - Detects structural changes
   - Triggers recovery actions

4. **CLI** (bin/tmux-agent.rs)
   - User-facing command interface
   - tmux-compatible syntax
   - Agent-specific extensions

### Design Principles

1. **Safety First**: Protect agent operations from interference
2. **TMux Compatibility**: Familiar commands and behavior
3. **Minimal Dependencies**: Only tokio for async operations
4. **Clear Feedback**: Visual warnings and status messages
5. **Automatic Recovery**: Self-healing from common issues