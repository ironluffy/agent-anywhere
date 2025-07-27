# Agent Creation Test Cookbook

This cookbook provides step-by-step instructions for creating a new agent in the Agent Anywhere system, accessing it, and debugging common issues.

## Prerequisites

1. Ensure `aany` is built: `cargo build --release`
2. Have tmux installed and running
3. Know your agent pool location (default: `~/.aany/`)

## Test Sequence: Create New Agent

### Step 1: Launch Agent Pool Manager

```bash
# In a new tmux session (e.g., session 15)
./target/release/aany
```

**Expected Output:**
- Agent Pool Manager UI should appear
- List of existing agents (if any)
- Navigation instructions at the bottom

**User Pattern:**
- Look for "◆ Create New Agent" at the top
- Check existing agents are listed without errors

### Step 2: Initiate Agent Creation

```bash
# Send 'n' key to create new agent
tmux send-keys -t 15 n
```

**Expected Output:**
- Creation form appears with fields:
  - Name: (auto-generated like "calm-sage-748")
  - Type: (default: "claude" or "general")
  - Purpose: "Development and testing"
  - Env Vars: Default environment variables
  - Config: Advanced Options (collapsed)

**User Pattern:**
- Verify all fields are populated
- Check that cursor is on the Name field

### Step 3: Customize Agent Name (Optional)

```bash
# Clear current name
tmux send-keys -t 15 C-u

# Type new name
tmux send-keys -t 15 "my-test-agent"

# Capture and verify
tmux capture-pane -t 15 -p | grep "Name:"
```

**Note:** If name doesn't update, proceed with auto-generated name.

### Step 4: Navigate to Create Button

```bash
# Method 1: Using Tab navigation
tmux send-keys -t 15 Tab Tab Tab Tab Tab Tab Tab Tab

# Method 2: Using Down arrow
tmux send-keys -t 15 Down Down Down Down Down

# Create the agent
tmux send-keys -t 15 Enter
```

**User Pattern:**
- Focus should move through fields
- Watch for CREATE AGENT button highlight
- If env vars expand, continue tabbing/arrowing

### Step 5: Verify Agent Creation

```bash
# Wait for creation
sleep 3

# Capture result
tmux capture-pane -t 15 -p
```

**Expected Output:**
- Return to main pool manager view
- New agent appears in list with STOPPED status
- "↻ Refreshed" message may appear

**Success Criteria:**
- Agent name appears in the list
- Status shows as STOPPED (newly created)
- No error messages

## Test Sequence: Access Agent

### Step 1: Navigate to New Agent

```bash
# Use arrow keys to select agent
tmux send-keys -t 15 Down  # Repeat until agent is selected
```

**User Pattern:**
- Selected agent should be highlighted
- Agent details visible in selection

### Step 2: Start the Agent

```bash
# Press Space to start/stop
tmux send-keys -t 15 Space

# Wait for startup
sleep 2

# Verify status
tmux capture-pane -t 15 -p | grep "RUNNING"
```

**Expected Output:**
- Agent status changes from STOPPED to RUNNING
- No error messages

### Step 3: Attach to Agent Session

```bash
# Press Enter or 'a' to attach
tmux send-keys -t 15 Enter

# Or directly attach
tmux attach -t agent-<agent-name>
```

**Expected Output:**
- Connected to agent's tmux session
- Claude interface should be visible
- Working directory: `~/.aany/agents/<agent-name>/workspace/`

## Debugging Common Issues

### Issue 1: "Failed to load agent" Errors

**Symptoms:**
```
Failed to load agent test-agent: IO error: No such file or directory (os error 2)
```

**Root Cause:** Agent directories exist without metadata files

**Debug Steps:**
```bash
# Check agent directory structure
ls -la ~/.aany/agents/<agent-name>/

# Look for metadata
ls -la ~/.aany/agents/<agent-name>/metadata/agent.yaml
ls -la ~/.aany/agents/<agent-name>/.agent.yaml

# If missing, agent is corrupted
```

**Fix:** Already implemented - agents without metadata are skipped

### Issue 2: Agent Creation Fails

**Debug Steps:**
```bash
# Check pool permissions
ls -la ~/.aany/

# Verify disk space
df -h ~/.aany/

# Check pool config
cat ~/.aany/.pool.yaml

# Look for agent limits
grep max_agents ~/.aany/.pool.yaml
```

### Issue 3: Agent Won't Start

**Debug Steps:**
```bash
# Check if tmux session exists
tmux ls | grep agent-

# Check agent metadata
cat ~/.aany/agents/<agent-name>/metadata/agent.yaml

# Verify tmux can create sessions
tmux new-session -d -s test-session
tmux kill-session -t test-session

# Check logs
ls -la ~/.aany/agents/<agent-name>/logs/
```

### Issue 4: Can't Attach to Agent

**Debug Steps:**
```bash
# Verify session is running
tmux has-session -t agent-<agent-name>
echo $?  # Should be 0 if exists

# List all tmux sessions
tmux ls

# Check agent status in pool
./target/release/aany
# Look for RUNNING status
```

## Complete Test Script

```bash
#!/bin/bash
# test_agent_creation.sh

echo "=== Agent Creation Test ==="

# Start pool manager in tmux
tmux new-session -d -s test-pool './target/release/aany'
sleep 2

# Create new agent
echo "Creating new agent..."
tmux send-keys -t test-pool n
sleep 1

# Navigate and create
tmux send-keys -t test-pool Tab Tab Tab Tab Tab Tab Tab Tab
tmux send-keys -t test-pool Enter
sleep 3

# Capture result
echo "Checking creation result..."
tmux capture-pane -t test-pool -p | tail -20

# Start the agent (assuming it's the last one)
echo "Starting agent..."
tmux send-keys -t test-pool End  # Go to end of list
tmux send-keys -t test-pool Space
sleep 2

# Verify running
echo "Verifying agent status..."
tmux capture-pane -t test-pool -p | grep "RUNNING"

# Cleanup
echo "Test complete. Pool manager running in session 'test-pool'"
echo "To attach: tmux attach -t test-pool"
```

## Validation Checklist

- [ ] Agent pool manager launches without errors
- [ ] Can navigate UI with keyboard
- [ ] Agent creation form appears when pressing 'n'
- [ ] Agent is created and appears in list
- [ ] Agent can be started (status changes to RUNNING)
- [ ] Can attach to agent session
- [ ] Agent workspace directory is created
- [ ] Metadata files are properly saved
- [ ] No error messages during the process

## Notes

1. The UI may have slight variations in auto-generated names
2. First agent creation might take longer due to directory setup
3. Use `tmux capture-pane -t <session> -p` frequently to debug UI state
4. Agent names must be unique within the pool