# Screenshot Logging Test Instructions

## Overview
The screenshot logging system captures tmux session content with process management and append capabilities.

## Step-by-Step Testing Instructions

### 1. Basic Setup
```bash
# Create a test tmux session
tmux new-session -d -s test-session

# Add some initial content
tmux send-keys -t test-session "echo 'Test session started'" C-m
```

### 2. Test Append Mode Logger (Single File, 3 minutes, 5-second intervals)
```bash
# Start the managed append logger
./tmux-screenshot-append-managed.sh test-session my-agent 180 5

# Check process status
./process-manager.sh list

# Add dynamic content while logger runs
tmux send-keys -t test-session "while true; do echo \"Time: \$(date)\"; sleep 2; done" C-m

# View captured content (after logger completes)
ls -la ~/.aany/agents/my-agent/screenshots/
cat ~/.aany/agents/my-agent/screenshots/session_log_*.txt
```

### 3. Test Process Management
```bash
# List all managed processes
./process-manager.sh list

# List only running processes
./process-manager.sh list running

# Stop a specific process
./process-manager.sh stop screenshot-append

# Stop all processes for an agent
./process-manager.sh stop-all agent my-agent

# Clean up dead processes
./process-manager.sh cleanup
```

### 4. Test Original Gap-Free Logger (Multiple Files)
```bash
# Start the enhanced logger with 30-second intervals
./tmux-screenshot-logger-v2.sh test-session my-agent 30 &

# Check captured files and metadata
ls -la ~/.aany/agents/my-agent/screenshots/pane_*.txt
cat ~/.aany/agents/my-agent/screenshots/pane_*.json

# Stop the logger
pkill -f tmux-screenshot-logger-v2.sh
```

### 5. Test Scrollback Handling
```bash
# Generate lots of content
tmux send-keys -t test-session "seq 1 1000" C-m

# Scroll up in tmux (attach and use Page Up)
tmux attach -t test-session
# Press Ctrl-B then Page Up to scroll

# Start logger - it should detect scroll and capture full buffer
./tmux-screenshot-append-managed.sh test-session my-agent 60 5
```

### 6. Verify No Gaps/Overlaps
```bash
# Start logger
./tmux-screenshot-append-managed.sh test-session my-agent 120 10

# Generate numbered lines
tmux send-keys -t test-session "for i in {1..50}; do echo \"Line \$i\"; sleep 0.5; done" C-m

# After completion, check the log file for:
# - All line numbers present (no gaps)
# - No duplicate line numbers (no overlaps)
grep "Line [0-9]" ~/.aany/agents/my-agent/screenshots/session_log_*.txt | sort -u
```

### 7. Test Edge Cases
```bash
# Empty buffer test
tmux new-session -d -s empty-test
./tmux-screenshot-append-managed.sh empty-test my-agent 30 5

# Very long output test
tmux send-keys -t test-session "cat /usr/share/dict/words | head -5000" C-m
./tmux-screenshot-append-managed.sh test-session my-agent 60 5
```

### 8. Cleanup
```bash
# Stop all processes
./process-manager.sh stop-all session test-session

# Kill tmux sessions
tmux kill-session -t test-session
tmux kill-session -t empty-test

# Clean up process entries
./process-manager.sh cleanup
```

## Verification Checklist
- [ ] Logger starts and creates output file
- [ ] Content is appended to single file with timestamps
- [ ] Process management tracks PIDs correctly
- [ ] No content gaps between captures
- [ ] No duplicate content (overlaps)
- [ ] Scrollback detection works
- [ ] Process cleanup removes dead entries
- [ ] Logs stop when tmux session ends

## Output Location
All screenshots are saved to: `~/.aany/agents/<agent_id>/screenshots/`

## Process Management
- Process metadata: `~/.aany/processes/*.json`
- PID files: `~/.aany/processes/*.pid`
- Archived processes: `~/.aany/processes/archive/`