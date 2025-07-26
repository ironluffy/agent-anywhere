# tmux-agent Python Package

Python wrapper for tmux-agent - an agent-friendly tmux wrapper with safety controls.

## Quick Install (No Compilation!)

```bash
# Pre-built wheel (instant installation)
pip install https://github.com/jayhansuh/agent-anywhere/raw/genesis/wheels/tmux_agent-0.3.0-py3-none-any.whl

# From PyPI (future)
pip install tmux-agent
```

## What's Included

- Pre-built binary (no Rust needed)
- Same CLI commands as Rust version
- Instant installation from wheels
- Full safety features

## Usage

```bash
# Create agent session
tmux-agent new my-bot
tma new my-bot  # shortcut

# Monitor safely (read-only)
tmux-agent monitor my-bot
tmon my-bot  # shortcut

# List sessions
tmux-agent list
tml  # shortcut

# Help
tmux-agent help
```

## Installation Options

### From Pre-built Wheel (Recommended)

```bash
# From GitHub release
pip install https://github.com/jayhansuh/agent-anywhere/raw/genesis/wheels/tmux_agent-0.3.0-py3-none-any.whl

# From future GitHub releases with platform-specific wheels
pip install https://github.com/jayhansuh/agent-anywhere/releases/download/v0.3.0/tmux_agent-0.3.0-py3-none-any.whl
```

### From Source (Requires Rust)

```bash
# Install from git (will compile Rust binary)
pip install git+ssh://git@github.com/jayhansuh/agent-anywhere.git#subdirectory=python

# For development
git clone git@github.com:jayhansuh/agent-anywhere.git
cd agent-anywhere/python
pip install -e .
```

### Add to Your Project

```bash
# Using uv
uv add git+ssh://git@github.com/jayhansuh/agent-anywhere.git#subdirectory=python

# In pyproject.toml
[project]
dependencies = [
    "tmux-agent @ git+ssh://git@github.com/jayhansuh/agent-anywhere.git#subdirectory=python",
]
```

## Why Wheels?

Pre-built wheels provide:
- ⚡ Instant installation (no Rust compilation)
- 🔧 No need for Rust toolchain
- 📦 Works on all platforms (Linux, macOS, Windows)
- 🚀 Same experience as installing ruff or uv

## Available Wheels

We provide wheels for:
- **Python**: 3.8, 3.9, 3.10, 3.11, 3.12
- **Linux**: manylinux wheels (glibc 2.17+)
- **macOS**: Universal wheels (Intel + Apple Silicon)
- **Windows**: 64-bit wheels

## Building Wheels Locally

```bash
cd python
./build-wheels.sh
```

## Prerequisites

- Python 3.8+
- tmux (must be installed separately)
- Git (for installation from repository)

## Note for Source Installation

When installing from source, the package will:
1. Check for Rust installation
2. Install Rust automatically if needed (requires curl)
3. Compile the tmux-agent binary (takes 1-2 minutes first time)
4. Future installs from cache will be much faster