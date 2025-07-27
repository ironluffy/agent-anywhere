#!/bin/bash
# View tmux screenshot logs for an agent

set -e

# Check arguments
if [ $# -lt 1 ]; then
    echo "Usage: $0 <agent-name> [screenshot-number]"
    echo "Example: $0 my-agent"
    echo "         $0 my-agent 5"
    exit 1
fi

AGENT_NAME="$1"
SCREENSHOT_NUM="${2:-}"
SCREENSHOT_DIR="$HOME/.aany/agents/$AGENT_NAME/screenshots"

# Check if agent exists
if [ ! -d "$SCREENSHOT_DIR" ]; then
    echo "No screenshots found for agent: $AGENT_NAME"
    echo "Directory not found: $SCREENSHOT_DIR"
    exit 1
fi

# Function to display a screenshot
show_screenshot() {
    local file="$1"
    local metadata_file="${file%.txt}.json"
    
    echo "═══════════════════════════════════════════════════════════════"
    echo "Screenshot: $(basename "$file")"
    
    if [ -f "$metadata_file" ]; then
        echo "Metadata:"
        cat "$metadata_file" | grep -E '"timestamp"|"pane"|"command"' | sed 's/^/  /'
    fi
    
    echo "───────────────────────────────────────────────────────────────"
    # Display the ANSI-formatted file (terminal will render colors)
    cat "$file"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
}

# If no specific screenshot requested, show list
if [ -z "$SCREENSHOT_NUM" ]; then
    echo "Screenshots for agent: $AGENT_NAME"
    echo "Directory: $SCREENSHOT_DIR"
    echo ""
    
    # Count screenshots
    TOTAL=$(ls -1 "$SCREENSHOT_DIR"/pane_*_plain.txt 2>/dev/null | wc -l)
    if [ $TOTAL -eq 0 ]; then
        echo "No screenshots found yet."
        exit 0
    fi
    
    echo "Found $TOTAL screenshots:"
    echo ""
    
    # List recent screenshots
    ls -1t "$SCREENSHOT_DIR"/pane_*.txt 2>/dev/null | head -20 | nl -v 1 | while read num file; do
        timestamp=$(basename "$file" | sed 's/pane_\(.*\).txt/\1/')
        readable_time=$(echo "$timestamp" | sed 's/\([0-9]\{4\}\)\([0-9]\{2\}\)\([0-9]\{2\}\)_\([0-9]\{2\}\)\([0-9]\{2\}\)\([0-9]\{2\}\)/\1-\2-\3 \4:\5:\6/')
        echo "  $num. $readable_time"
    done
    
    echo ""
    echo "To view a specific screenshot: $0 $AGENT_NAME <number>"
    echo "To view latest: $0 $AGENT_NAME 1"
else
    # Show specific screenshot
    FILE=$(ls -1t "$SCREENSHOT_DIR"/pane_*.txt 2>/dev/null | sed -n "${SCREENSHOT_NUM}p")
    
    if [ -z "$FILE" ]; then
        echo "Screenshot #$SCREENSHOT_NUM not found"
        exit 1
    fi
    
    show_screenshot "$FILE"
    
    # Show navigation
    echo "Navigation:"
    echo "  Next: $0 $AGENT_NAME $((SCREENSHOT_NUM + 1))"
    echo "  Prev: $0 $AGENT_NAME $((SCREENSHOT_NUM - 1))"
    echo "  List: $0 $AGENT_NAME"
fi

