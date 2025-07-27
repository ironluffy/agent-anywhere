#!/bin/bash
# Wrapper for claude that returns to pool UI on exit

# Run claude
claude

# After claude exits (e.g., with Ctrl+D), check if we should return
RETURN_SESSION=$(tmux show-environment AANY_RETURN_SESSION 2>/dev/null | cut -d= -f2)

if [ -n "$RETURN_SESSION" ]; then
    echo "Returning to pool UI..."
    sleep 0.5  # Brief pause for visual feedback
    tmux switch-client -t "$RETURN_SESSION"
fi