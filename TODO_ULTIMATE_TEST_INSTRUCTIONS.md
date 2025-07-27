# TODO_ULTIMATE_TEST Instructions

## Overview
This test verifies that tmux-agent logs all interactions including:
- User typing (keyboard input)
- Tmux screen history
- Agent identifier and metadata
- All interactions are streamed to hub

## Step-by-Step Instructions

### 1. Start the test
```bash
./test_ultimate_logging.sh
```

### 2. Create agent from aany pool UI
When the pool UI opens:
1. Press `n` to create new agent
2. Enter name: `ultimate-test`
3. Press `q` to quit

### 3. The script will automatically:
- Send `nvm use 22` command
- Send `claude` command  
- Send `tell me about this directory` command
- Wait for logging
- Verify all interactions were logged

### 4. Check the results
The script will show:
- Location of log file
- Contents of the log
- Verification of all required elements:
  - ✓ Agent ID (ultimate-test)
  - ✓ Session name
  - ✓ User typing commands
  - ✓ Screen history updates
  - ✓ Metadata

## Manual Verification

To manually verify logs:
```bash
# View the log file
tail -f /tmp/tmux-agent-logs/tmux_interaction_ultimate-test_*.log

# Monitor the agent session
tmux attach -r -t agent-ultimate-test

# Check hub logs (if hub is running)
curl http://localhost:8090/logs
```

## Expected Log Format
```
[TIMESTAMP] [interaction_type] agent_id: content
  Agent ID: ultimate-test
  Session: agent-ultimate-test
  Hub URL: localhost:50052

[TIMESTAMP] [screen_update] ultimate-test: nvm use 22
  Agent ID: ultimate-test
  Session: agent-ultimate-test
  Hub URL: localhost:50052
```

## Success Criteria
- All user typing is captured
- All screen updates are logged
- Agent ID and session metadata present
- Logs are saved locally to /tmp/tmux-agent-logs/
- Logs are streamed to hub (if enabled)