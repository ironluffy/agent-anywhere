#!/bin/bash
# Quick installer for tmux-agent
# Usage: curl -sSL https://example.com/quick-install.sh | bash

set -e

echo "🚀 Quick Installing tmux-agent..."
echo ""

# Create temp directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Clone or download the project
if command -v git &> /dev/null; then
    git clone https://github.com/jayhansuh/agent-anywhere.git
    cd agent-anywhere/aany-tmux
else
    echo "❌ Error: git not found. Please install git first."
    exit 1
fi

# Already in aany-tmux directory

# Auto-select option 1 (user install) for quick install
echo "1" | ./install.sh

# Clean up
cd ~
rm -rf "$TEMP_DIR"

echo ""
echo "✅ Installation complete!"
echo ""
echo "🔄 Please run this to reload your shell:"
echo "   exec $SHELL"
echo ""
echo "Then try:"
echo "   tmux-agent help"