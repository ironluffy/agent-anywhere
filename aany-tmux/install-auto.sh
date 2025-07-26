#!/bin/bash
# Automatic installer for tmux-agent - no prompts!

set -e

echo "🤖 TMux Agent Auto-Installer"
echo "==========================="
echo ""

# Check requirements
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

if ! command -v tmux &> /dev/null; then
    echo "❌ tmux not found!"
    echo ""
    echo "Please install tmux:"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "  brew install tmux"
    elif command -v apt-get &> /dev/null; then
        echo "  sudo apt-get install tmux"
    elif command -v yum &> /dev/null; then
        echo "  sudo yum install tmux"
    else
        echo "  Use your package manager to install tmux"
    fi
    exit 1
fi

# Build tmux-agent
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "🔨 Building tmux-agent..."
source "$HOME/.cargo/env" 2>/dev/null || true
cargo build --release --bin tmux-agent --quiet

# Install to user directory
echo "📁 Installing to ~/.local/bin..."
mkdir -p ~/.local/bin
cp target/release/tmux-agent ~/.local/bin/
chmod +x ~/.local/bin/tmux-agent

# Detect shell
USER_SHELL=$(basename "$SHELL")
SHELL_RC=""
case "$USER_SHELL" in
    bash) SHELL_RC="$HOME/.bashrc" ;;
    zsh)  SHELL_RC="$HOME/.zshrc" ;;
    fish) SHELL_RC="$HOME/.config/fish/config.fish" ;;
    *)    SHELL_RC="$HOME/.profile" ;;
esac

echo "🐚 Configuring $USER_SHELL..."

# Function to append to file if line doesn't exist
append_if_missing() {
    grep -qF "$1" "$2" 2>/dev/null || echo "$1" >> "$2"
}

# Add to PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    case "$USER_SHELL" in
        fish)
            mkdir -p ~/.config/fish/conf.d
            echo 'set -gx PATH $HOME/.local/bin $PATH' > ~/.config/fish/conf.d/tmux-agent-path.fish
            ;;
        *)
            append_if_missing 'export PATH="$HOME/.local/bin:$PATH"' "$SHELL_RC"
            ;;
    esac
fi

# Install completions
echo "🔧 Installing completions..."
mkdir -p "$SCRIPT_DIR/completions"

# Create completions if they don't exist
if [ ! -f "$SCRIPT_DIR/completions/tmux-agent.bash" ]; then
    cat > "$SCRIPT_DIR/completions/tmux-agent.bash" << 'EOF'
_tmux_agent() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "new monitor attach list health cleanup protect kill status help" -- ${cur}) )
    elif [[ "${prev}" =~ ^(monitor|attach|health|cleanup|protect|kill|mon|a)$ ]]; then
        local sessions=$(tmux list-sessions -F "#{session_name}" 2>/dev/null)
        COMPREPLY=( $(compgen -W "${sessions}" -- ${cur}) )
    fi
}
complete -F _tmux_agent tmux-agent
complete -F _tmux_agent tma
EOF
fi

case "$USER_SHELL" in
    bash)
        append_if_missing "source $SCRIPT_DIR/completions/tmux-agent.bash" "$SHELL_RC"
        ;;
    zsh)
        mkdir -p ~/.config/zsh/completions
        echo "fpath+=$SCRIPT_DIR/completions" >> "$SHELL_RC"
        ;;
esac

# Add aliases
echo "✨ Adding aliases..."
case "$USER_SHELL" in
    fish)
        mkdir -p ~/.config/fish/conf.d
        cat > ~/.config/fish/conf.d/tmux-agent.fish << 'EOF'
alias tma="tmux-agent"
alias tmon="tmux-agent monitor"
alias tml="tmux-agent list"
EOF
        ;;
    *)
        {
            echo ""
            echo "# tmux-agent aliases"
            echo 'alias tma="tmux-agent"'
            echo 'alias tmon="tmux-agent monitor"'
            echo 'alias tml="tmux-agent list"'
        } >> "$SHELL_RC"
        ;;
esac

# Success!
echo ""
echo "✅ Installation complete!"
echo ""
echo "🎯 Quick start:"
echo "   1. Reload shell: exec $SHELL"
echo "   2. Create session: tma new my-bot"
echo "   3. Monitor it: tmon my-bot"
echo "   4. List all: tml"
echo ""
echo "📖 Full docs: tmux-agent help"
echo ""
echo "Happy agenting! 🤖"