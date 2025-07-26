# Bug Report: Infinite Loop When Opening Log Files Inside TMux Agent Session

## Summary
Opening a session log file from within a tmux agent session causes an infinite logging loop, as the log viewer's output is captured by the logging system, creating a recursive feedback loop.

## Environment
- Project: agent-anywhere
- Component: aany-pool logging system
- Date Discovered: 2025-07-26
- Reference Log: `session_20250726_091424.log`

## Description
The logging system uses `tmux pipe-pane` to capture all terminal I/O from agent sessions. When a user opens a log file (e.g., using `cat`, `less`, `tail`) from within the agent's tmux session, the output of the log viewer is itself captured and appended to the same log file being viewed. This creates an infinite feedback loop where:

1. User opens log file → 
2. Log content is displayed in terminal → 
3. TMux pipe-pane captures the displayed content → 
4. Content is appended to the log file → 
5. If using `tail -f` or similar, the viewer shows the new content → 
6. Loop continues indefinitely

## Steps to Reproduce
1. Start an agent: `aany pool start research-bot`
2. Attach to the agent: `aany pool attach research-bot`
3. Navigate to logs directory: `cd ~/.aany/pool/agents/research-bot/logs/`
4. Open the current session log: `tail -f session_latest.log`
5. Observe the infinite loop as the log file grows exponentially

## Expected Behavior
Users should be able to view log files from within agent sessions without causing recursive logging.

## Actual Behavior
Opening log files creates an infinite feedback loop, rapidly growing the log file size and potentially filling disk space.

## Impact
- **Severity**: Medium
- **Frequency**: Occurs whenever logs are viewed from within agent session
- **User Impact**: Can fill disk space, make logs unreadable, requires force-stopping the viewer

## Potential Solutions
1. **Exclude log viewing commands**: Filter out specific commands (cat, less, tail, etc.) when they target log files
2. **Temporary pause logging**: Add a command to temporarily disable logging while viewing logs
3. **Separate log viewer session**: Create a dedicated tmux pane/window for log viewing that isn't logged
4. **Smart filtering**: Detect and prevent logging of content that matches existing log entries
5. **Read-only log access**: Provide a special log viewing command that bypasses the logging system

## Workaround
View logs from outside the agent session:
```bash
# From main terminal (not inside agent)
aany pool logs research-bot
```

## Related Code
- Logging setup: `/aany-pool/src/logging.rs:setup_tmux_logging()`
- TMux pipe-pane command: `/aany-pool/src/logging.rs:195-202`
- Agent start logging: `/aany-pool/src/agent.rs:139-147`

## Notes
This is a fundamental issue with using `tmux pipe-pane` for comprehensive logging. Any solution needs to balance between complete logging coverage and preventing recursive loops.