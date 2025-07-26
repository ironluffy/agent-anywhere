# Agent Anywhere

A comprehensive system for managing AI agents with tmux integration and safety controls.

## 🚀 Quick Start

### Prerequisites

- **tmux**: `apt install tmux` (Ubuntu) or `brew install tmux` (macOS)
- **Python**: 3.8+

### Install

```bash
# Download pre-built binary (macOS ARM64)
curl -L https://github.com/ironluffy/agent-anywhere/releases/download/v0.1.0/aany-v0.1.0-Darwin-arm64.tar.gz | tar xz
sudo mv aany /usr/local/bin/

# Or build from source (requires Rust)
cargo install --path aany
```

### First Steps

```bash
# Using unified CLI (aany)
aany pool create my-bot        # Create an agent
aany pool start my-bot         # Start the agent
aany pool ui                   # Interactive UI

# Using tmux commands directly
aany tmux new my-session       # Create tmux session
aany tmux monitor my-session   # Monitor (read-only)
aany tmux list                 # List all sessions

# Get help
aany --help
aany pool --help
aany tmux --help
```

## 📦 Components

### 1. **aany** - Unified CLI
- Single entry point: `aany <command>`
- Manages both tmux sessions and agent pools
- Consistent interface across all features

### 2. **aany-tmux** - TMux Integration
- Safe tmux session management
- Human interference detection
- Read-only monitoring mode
- Formerly `tmux-agent-proxy`

### 3. **aany-pool** - Agent Pool Manager
- Manage multiple AI agents
- Agent lifecycle management
- Metadata and task tracking
- Template system
- Interactive TUI with `aany pool ui`

### 4. **python/** - Python Package
- Works with existing `tmux-agent` commands
- Pre-built wheels for easy installation
- No Rust compilation required

## 💡 Key Features

- **Agent Pool Management**: Create, start, stop, and manage multiple AI agents
- **TMux Safety**: Prevents accidental interference with AI sessions
- **Logging**: Comprehensive session and event logging
- **Templates**: Pre-configured agent types (claude, general, research, custom)
- **Interactive UI**: Terminal UI for easy agent management

## 🛠️ Other Install Options

<details>
<summary>Build from source</summary>

```bash
# Clone and build with Rust
git clone https://github.com/ironluffy/agent-anywhere.git
cd agent-anywhere
cargo build --release

# Binary will be at target/release/aany
```
</details>

<details>
<summary>Add to Python project</summary>

```bash
# In pyproject.toml
[project]
dependencies = [
    "aany @ git+https://github.com/ironluffy/agent-anywhere.git#subdirectory=python",
]
```
</details>

## 📚 Documentation

Full documentation coming soon. For now, use `aany --help` for command reference.

## Requirements

- Python 3.8+
- tmux
- Rust (only for building from source)

## License

MIT