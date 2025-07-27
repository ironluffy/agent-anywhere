#!/bin/bash
# Switch to a tmux session and set up return behavior

TARGET_SESSION="$1"
RETURN_SESSION="$2"

if [ -z "$TARGET_SESSION" ] || [ -z "$RETURN_SESSION" ]; then
    echo "Usage: $0 <target-session> <return-session>"
    exit 1
fi

# Set an environment variable in the target session to remember where to return
tmux set-environment -t "$TARGET_SESSION" AANY_RETURN_SESSION "$RETURN_SESSION"

# Switch to the target session
tmux switch-client -t "$TARGET_SESSION"