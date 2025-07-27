# Tmux Screenshot Logging

## Overview

The screenshot logger captures visual snapshots of tmux sessions at regular intervals, providing a visual history of agent activities.

## Features

### 1. Text-Based Screenshots
- Captures full tmux pane content with ANSI colors
- Also saves plain text version for readability
- Includes cursor position and pane dimensions
- Stored in: `~/.aany/agents/{agent}/logs/screenshots/`

### 2. Visual Screenshots (macOS)
- Captures actual screen images (if available)
- Taken every 5 minutes (less frequent than text)
- Stored as PNG files

### 3. Automatic Management
- Keeps last 100 screenshots
- Automatic cleanup of old files
- Minimal disk usage

## File Structure

```
~/.aany/agents/{agent}/logs/screenshots/
├── pane_YYYYMMDD_HHMMSS.txt         # ANSI colored capture
├── pane_YYYYMMDD_HHMMSS_plain.txt   # Plain text version
├── pane_YYYYMMDD_HHMMSS.json        # Metadata (cursor, size, etc)
├── visual_YYYYMMDD_HHMMSS.png       # Screen capture (if available)
├── visual_YYYYMMDD_HHMMSS.json      # Visual capture metadata
└── index.html                        # Web viewer (future)
```

## Configuration

### Default Settings
- Capture interval: 30 seconds
- Retention: Last 100 screenshots
- Enabled by default (unless `AANY_LOGGING_DISABLED=true`)

### Custom Interval
The screenshot interval can be customized when starting the logger:
```bash
# 60 second interval
tmux-screenshot-logger.sh session-name agent-name 60
```

## Viewing Screenshots

### Using the viewer script:
```bash
# List all screenshots
./aany-tmux/view-screenshots.sh my-agent

# View latest screenshot
./aany-tmux/view-screenshots.sh my-agent 1

# View specific screenshot
./aany-tmux/view-screenshots.sh my-agent 5
```

### Direct access:
```bash
# View latest plain text screenshot
ls -t ~/.aany/agents/my-agent/logs/screenshots/pane_*_plain.txt | head -1 | xargs cat

# View with ANSI colors
ls -t ~/.aany/agents/my-agent/logs/screenshots/pane_*.txt | head -1 | xargs cat
```

## Use Cases

1. **Debugging**: See exactly what was on screen when an error occurred
2. **Monitoring**: Visual history of agent activities
3. **Documentation**: Capture important moments
4. **Audit Trail**: Visual proof of actions taken

## Example Metadata

```json
{
  "timestamp": "2025-01-27T10:30:45Z",
  "agent_id": "my-agent",
  "session": "agent-my-agent",
  "pane": {
    "width": 80,
    "height": 24,
    "cursor_x": 15,
    "cursor_y": 10,
    "command": "bash"
  },
  "files": {
    "ansi": "pane_20250127_103045.txt",
    "plain": "pane_20250127_103045_plain.txt"
  }
}
```

## Performance

- Minimal CPU usage (captures are instant)
- ~2KB per text screenshot
- Automatic cleanup prevents disk bloat
- Separate process doesn't impact agent performance