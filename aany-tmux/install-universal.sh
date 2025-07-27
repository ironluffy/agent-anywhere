#!/bin/bash
# Universal installer for tmux-agent
# Works on macOS (via Homebrew) and Linux

set -e

echo "🤖 TMux Agent Universal Installer"
echo "================================"
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

# macOS installation via Homebrew
if [[ "$OS" == "macos" ]]; then
    echo ""
    echo "🍺 Installing via Homebrew..."
    
    # Check if Homebrew is installed
    if ! command -v brew &> /dev/null; then
        echo "❌ Homebrew not found!"
        echo ""
        echo "Install Homebrew first:"
        echo '  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"'
        exit 1
    fi
    
    # Install tmux if needed
    if ! command -v tmux &> /dev/null; then
        echo "📦 Installing tmux..."
        brew install tmux
    fi
    
    # Install tmux-agent
    echo "📦 Installing tmux-agent..."
    brew install ironluffy/tap/tmux-agent || {
        echo "⚠️  Direct formula install failed, trying tap first..."
        brew tap ironluffy/tap
        brew install tmux-agent
    }
    
    echo ""
    echo "✅ Installation complete!"
    echo ""
    echo "🚀 Quick start:"
    echo "   tmux-agent new my-bot      # Create session"
    echo "   tmux-agent monitor my-bot  # Monitor safely"
    echo "   tmux-agent help           # Show help"
    echo ""
    echo "💡 Tip: Add aliases to your ~/.zshrc or ~/.bash_profile:"
    echo "   alias tma='tmux-agent'"
    echo "   alias tmon='tmux-agent monitor'"
    
# Linux installation
elif [[ "$OS" == "linux" ]]; then
    echo ""
    echo "🐧 Installing on Linux..."
    
    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        echo "📦 Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    # Check for tmux
    if ! command -v tmux &> /dev/null; then
        echo "❌ tmux not found!"
        echo ""
        echo "Please install tmux:"
        if command -v apt-get &> /dev/null; then
            echo "  sudo apt-get install tmux"
        elif command -v yum &> /dev/null; then
            echo "  sudo yum install tmux"
        elif command -v pacman &> /dev/null; then
            echo "  sudo pacman -S tmux"
        else
            echo "  Use your package manager to install tmux"
        fi
        exit 1
    fi
    
    # Clone and build
    echo "📦 Downloading and building tmux-agent..."
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    git clone --branch genesis https://github.com/ironluffy/agent-anywhere.git
    cd agent-anywhere/aany-tmux
    
    # Run the auto installer
    ./install-auto.sh
    
    # Cleanup
    cd ~
    rm -rf "$TEMP_DIR"
    
    echo ""
    echo "✅ Installation complete!"
    echo ""
    echo "🔄 Reload your shell:"
    echo "   exec $SHELL"
    echo ""
    echo "Then try:"
    echo "   tmux-agent help"
fi