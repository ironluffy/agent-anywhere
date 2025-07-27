# Ironluffy Homebrew Tap

This tap contains Homebrew formulas for Ironluffy tools.

## Installation

```bash
# Add the tap
brew tap ironluffy/tap

# Install tmux-agent
brew install tmux-agent
```

## One-liner Installation

```bash
brew install ironluffy/tap/tmux-agent
```

## Available Formulas

### tmux-agent

Agent-friendly tmux wrapper with safety controls.

```bash
# Install
brew install ironluffy/tap/tmux-agent

# Update
brew upgrade tmux-agent

# Uninstall
brew uninstall tmux-agent
```

## Development

To test locally:
```bash
brew install --build-from-source ./Formula/tmux-agent.rb
```