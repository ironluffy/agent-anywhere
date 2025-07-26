#!/bin/bash

set -e

echo "🤖 TMux Agent Installer"
echo "====================="
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust/Cargo not found!"
    echo "Please install Rust first: https://rustup.rs"
    exit 1
fi

# Check if tmux is installed
if ! command -v tmux &> /dev/null; then
    echo "❌ Error: tmux not found!"
    echo "Please install tmux first:"
    echo "  Ubuntu/Debian: sudo apt install tmux"
    echo "  macOS: brew install tmux"
    exit 1
fi

# Get the directory of this script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "📦 Building tmux-agent..."
source "$HOME/.cargo/env" 2>/dev/null || true
cargo build --release --bin tmux-agent

# Check if build succeeded
if [ ! -f "target/release/tmux-agent" ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo ""
echo "🛠️  Installation Options:"
echo "1) Install to ~/.local/bin (user-only)"
echo "2) Install to /usr/local/bin (system-wide, requires sudo)"
echo "3) Just build, don't install"
echo ""
read -p "Choose option (1-3): " choice

case $choice in
    1)
        # User installation
        mkdir -p ~/.local/bin
        cp target/release/tmux-agent ~/.local/bin/
        chmod +x ~/.local/bin/tmux-agent
        
        echo "✅ Installed to ~/.local/bin/tmux-agent"
        
        # Check if ~/.local/bin is in PATH
        if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
            echo ""
            echo "⚠️  ~/.local/bin is not in your PATH!"
            echo "Add this to your ~/.bashrc or ~/.zshrc:"
            echo '    export PATH="$HOME/.local/bin:$PATH"'
        fi
        ;;
    2)
        # System installation
        sudo cp target/release/tmux-agent /usr/local/bin/
        sudo chmod +x /usr/local/bin/tmux-agent
        echo "✅ Installed to /usr/local/bin/tmux-agent"
        ;;
    3)
        echo "✅ Build complete. Binary at: target/release/tmux-agent"
        echo "You can run it directly: ./target/release/tmux-agent"
        ;;
    *)
        echo "Invalid option"
        exit 1
        ;;
esac

echo ""
echo "🎉 Installation complete!"
echo ""
echo "Try these commands:"
echo "  tmux-agent help              # Show help"
echo "  tmux-agent new my-agent      # Create agent session"
echo "  tmux-agent monitor my-agent  # Safe monitoring"
echo "  tmux-agent list              # List sessions"
echo ""

# Auto-install shell completions and update PATH
echo ""
echo "🔧 Setting up your environment..."

# Detect user's shell
USER_SHELL=$(basename "$SHELL")
echo "   Detected shell: $USER_SHELL"

# Function to add line to file if not already present
add_to_file_if_missing() {
    local file="$1"
    local line="$2"
    local comment="$3"
    
    if ! grep -Fxq "$line" "$file" 2>/dev/null; then
        echo "" >> "$file"
        echo "# $comment" >> "$file"
        echo "$line" >> "$file"
        return 0
    fi
    return 1
}

# Handle PATH setup
PATH_UPDATED=false
if [[ "$choice" == "1" ]] && [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo "   Adding ~/.local/bin to PATH..."
    
    case "$USER_SHELL" in
        bash)
            if add_to_file_if_missing ~/.bashrc 'export PATH="$HOME/.local/bin:$PATH"' "Added by tmux-agent installer"; then
                PATH_UPDATED=true
            fi
            ;;
        zsh)
            if add_to_file_if_missing ~/.zshrc 'export PATH="$HOME/.local/bin:$PATH"' "Added by tmux-agent installer"; then
                PATH_UPDATED=true
            fi
            ;;
        fish)
            mkdir -p ~/.config/fish/conf.d
            echo 'set -gx PATH $HOME/.local/bin $PATH' > ~/.config/fish/conf.d/tmux-agent.fish
            PATH_UPDATED=true
            ;;
    esac
fi

# Install completions automatically
echo "   Installing shell completions..."
COMPLETIONS_DIR="$SCRIPT_DIR/completions"

case "$USER_SHELL" in
    bash)
        # Create bash completion
        mkdir -p ~/.local/share/bash-completion/completions
        cp "$COMPLETIONS_DIR/tmux-agent.bash" ~/.local/share/bash-completion/completions/tmux-agent 2>/dev/null || {
            # Fallback: add to bashrc
            add_to_file_if_missing ~/.bashrc "source $COMPLETIONS_DIR/tmux-agent.bash" "tmux-agent bash completion"
        }
        echo "   ✅ Bash completions installed"
        ;;
    zsh)
        # Create zsh completion
        mkdir -p ~/.local/share/zsh/site-functions
        cp "$COMPLETIONS_DIR/tmux-agent.zsh" ~/.local/share/zsh/site-functions/_tmux-agent 2>/dev/null || {
            # Fallback: add to zshrc
            echo "fpath+=$COMPLETIONS_DIR" >> ~/.zshrc
        }
        echo "   ✅ Zsh completions installed"
        ;;
    fish)
        # Create fish completion
        mkdir -p ~/.config/fish/completions
        # Convert bash completion to fish format (simplified)
        cat > ~/.config/fish/completions/tmux-agent.fish << 'EOF'
complete -c tmux-agent -f
complete -c tmux-agent -n "__fish_use_subcommand" -a "new" -d "Create new agent session"
complete -c tmux-agent -n "__fish_use_subcommand" -a "monitor mon" -d "Safe read-only monitoring"
complete -c tmux-agent -n "__fish_use_subcommand" -a "attach a" -d "Interactive attach (careful!)"
complete -c tmux-agent -n "__fish_use_subcommand" -a "list ls" -d "List sessions"
complete -c tmux-agent -n "__fish_use_subcommand" -a "health" -d "Check session health"
complete -c tmux-agent -n "__fish_use_subcommand" -a "cleanup" -d "Clean up extra panes"
complete -c tmux-agent -n "__fish_use_subcommand" -a "protect" -d "Show protection warning"
complete -c tmux-agent -n "__fish_use_subcommand" -a "kill" -d "Kill session"
complete -c tmux-agent -n "__fish_use_subcommand" -a "status" -d "System overview"
complete -c tmux-agent -n "__fish_use_subcommand" -a "help" -d "Show help"

# Complete session names for relevant commands
complete -c tmux-agent -n "__fish_seen_subcommand_from monitor mon attach a kill health cleanup protect" \
    -a "(tmux list-sessions -F '#{session_name}' 2>/dev/null)"
EOF
        echo "   ✅ Fish completions installed"
        ;;
esac

# Create convenient aliases
echo "   Adding convenient aliases..."
case "$USER_SHELL" in
    bash)
        add_to_file_if_missing ~/.bashrc "alias tma='tmux-agent'" "tmux-agent alias"
        add_to_file_if_missing ~/.bashrc "alias tmon='tmux-agent monitor'" "tmux-agent monitor alias"
        ;;
    zsh)
        add_to_file_if_missing ~/.zshrc "alias tma='tmux-agent'" "tmux-agent alias"
        add_to_file_if_missing ~/.zshrc "alias tmon='tmux-agent monitor'" "tmux-agent monitor alias"
        ;;
    fish)
        mkdir -p ~/.config/fish/conf.d
        echo "alias tma='tmux-agent'" > ~/.config/fish/conf.d/tmux-agent-aliases.fish
        echo "alias tmon='tmux-agent monitor'" >> ~/.config/fish/conf.d/tmux-agent-aliases.fish
        ;;
esac

echo ""
echo "✨ Setup complete!"
echo ""

# Show what was configured
if [[ "$PATH_UPDATED" == "true" ]]; then
    echo "📝 Updated your shell config:"
    echo "   - Added ~/.local/bin to PATH"
fi
echo "   - Installed shell completions"
echo "   - Added aliases: tma, tmon"
echo ""

# Final instructions
echo "🚀 To start using tmux-agent:"
echo ""
if [[ "$PATH_UPDATED" == "true" ]] || [[ "$USER_SHELL" != "bash" ]]; then
    echo "   1. Reload your shell:"
    case "$USER_SHELL" in
        bash) echo "      source ~/.bashrc" ;;
        zsh)  echo "      source ~/.zshrc" ;;
        fish) echo "      source ~/.config/fish/config.fish" ;;
        *)    echo "      Start a new terminal" ;;
    esac
    echo ""
    echo "   2. Try it out:"
else
    echo "   Try it out:"
fi
echo "      tmux-agent new my-bot     # Create session"
echo "      tmon my-bot               # Monitor safely"
echo "      tma list                  # List sessions"
echo ""
echo "📖 Documentation:"
echo "   cat TMUX_AGENT_README.md"