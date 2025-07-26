#!/bin/bash
# One-liner installer script for tmux-agent
# This script is meant to be piped from curl

set -e

echo "🤖 TMux Agent Quick Installer"
echo "============================"
echo ""

# Detect OS
OS="unknown"
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
else
    echo "❌ Unsupported OS: $OSTYPE"
    exit 1
fi

echo "🔍 Detected OS: $OS"

# macOS: Use Homebrew
if [[ "$OS" == "macos" ]]; then
    echo "🍺 Installing via Homebrew..."
    
    if ! command -v brew &> /dev/null; then
        echo "❌ Homebrew not found!"
        echo "Install it first: /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        exit 1
    fi
    
    brew install jayhansuh/tap/tmux-agent || {
        brew tap jayhansuh/tap
        brew install tmux-agent
    }
    
    echo ""
    echo "✅ Installation complete!"
    echo "Try: tmux-agent help"

# Linux: Download and run installer
elif [[ "$OS" == "linux" ]]; then
    echo "🐧 Installing on Linux..."
    
    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # Clone repository
    echo "📦 Downloading tmux-agent..."
    git clone --branch genesis --depth 1 https://github.com/jayhansuh/agent-anywhere.git
    
    # Run the auto installer
    cd agent-anywhere/aany-tmux
    ./install-auto.sh
    
    # Cleanup
    cd ~
    rm -rf "$TEMP_DIR"
    
    echo ""
    echo "🔄 Reload your shell: exec \$SHELL"
fi