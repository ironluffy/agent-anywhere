# Step-by-Step Instructions to Test Logging

## Test 1: Using aany pool UI

### Step 1: Clean up
```bash
tmux kill-server
rm -rf ~/.aany/agents/test-agent-ui
```

### Step 2: Start aany pool
```bash
./target/release/aany pool
```

### Step 3: Create agent in UI
- Press `n` (create new agent)
- Type: `test-agent-ui`
- Press Enter
- Press `q` (quit)

### Step 4: Send test commands
```bash
# Wait 5 seconds for agent to initialize
sleep 5

# Send commands
tmux send-keys -t agent-test-agent-ui "nvm use 22" C-m
sleep 2
tmux send-keys -t agent-test-agent-ui "claude" C-m
sleep 2
tmux send-keys -t agent-test-agent-ui "tell me about this directory" C-m
```

### Step 5: Check logs
```bash
# View interaction log
cat ~/.aany/agents/test-agent-ui/logs/tmux_interaction_*.log

# Should contain:
# - Agent ID: test-agent-ui
# - Session: agent-test-agent-ui
# - "nvm use 22"
# - "claude"
# - "tell me about this directory"
```

## Test 2: Using aany tmux new

### Step 1: Clean up
```bash
tmux kill-server
rm -rf ~/.aany/agents/test-agent-cli
```

### Step 2: Create agent via CLI
```bash
./target/release/aany tmux new test-agent-cli
```

### Step 3: Send test commands
```bash
# Wait 5 seconds
sleep 5

# Send commands
tmux send-keys -t agent-test-agent-cli "echo 'Hello from CLI agent'" C-m
sleep 2
tmux send-keys -t agent-test-agent-cli "pwd" C-m
```

### Step 4: Check logs
```bash
# View logs
cat ~/.aany/agents/test-agent-cli/logs/tmux_interaction_*.log

# Should contain:
# - Agent ID: test-agent-cli
# - "Hello from CLI agent"
# - "pwd"
```

## Verify Log Location

All logs are now stored in:
```
~/.aany/agents/{agent-name}/logs/
├── tmux_interaction_YYYYMMDD_HHMMSS.log  # User input & screen
├── session_YYYYMMDD_HHMMSS.log          # Session transcript
└── commands_YYYYMMDD_HHMMSS.log         # Command history
```

## Monitor Live

To watch logs in real-time:
```bash
# For UI-created agent
tail -f ~/.aany/agents/test-agent-ui/logs/tmux_interaction_*.log

# For CLI-created agent  
tail -f ~/.aany/agents/test-agent-cli/logs/tmux_interaction_*.log
```

## Clean Up

```bash
# Kill specific sessions
tmux kill-session -t agent-test-agent-ui
tmux kill-session -t agent-test-agent-cli

# Remove agent data
rm -rf ~/.aany/agents/test-agent-ui
rm -rf ~/.aany/agents/test-agent-cli
```