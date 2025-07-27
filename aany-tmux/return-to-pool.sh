#!/bin/bash
# Return to the pool UI session if AANY_RETURN_SESSION is set

RETURN_SESSION=$(tmux show-environment AANY_RETURN_SESSION 2>/dev/null | cut -d= -f2)

if [ -n "$RETURN_SESSION" ]; then
    echo "Returning to $RETURN_SESSION..."
    tmux switch-client -t "$RETURN_SESSION"
else
    echo "No return session set"
fi