#!/bin/bash
# Quick installation verification script

echo "=== Agent Anywhere Installation Verification ==="
echo

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

errors=0
warnings=0

# Check prerequisites
echo "Checking prerequisites..."

# Check tmux
if command -v tmux &> /dev/null; then
    echo -e "${GREEN}✓${NC} tmux is installed ($(tmux -V))"
else
    echo -e "${RED}✗${NC} tmux is not installed (required)"
    ((errors++))
fi

# Check git
if command -v git &> /dev/null; then
    echo -e "${GREEN}✓${NC} git is installed ($(git --version | head -1))"
else
    echo -e "${YELLOW}⚠${NC} git is not installed (recommended for cloning repositories)"
    ((warnings++))
fi

# Check Python (optional)
if command -v python3 &> /dev/null; then
    echo -e "${GREEN}✓${NC} Python is installed ($(python3 --version))"
else
    echo -e "${YELLOW}⚠${NC} Python is not installed (optional)"
    ((warnings++))
fi

echo

# Check aany commands
echo "Checking Agent Anywhere installation..."

# Check main binary
if command -v aany &> /dev/null; then
    echo -e "${GREEN}✓${NC} aany command is available"
    # Try to get version
    if aany --version &> /dev/null; then
        echo "   Version: $(aany --version)"
    fi
else
    echo -e "${RED}✗${NC} aany command not found"
    ((errors++))
    
    # Check if built locally
    if [ -f "./target/release/aany" ]; then
        echo -e "${YELLOW}ℹ${NC}  Found locally built binary at ./target/release/aany"
        echo "   Add to PATH or use: ./target/release/aany"
    fi
fi

# Check aany-pool
if command -v aany-pool &> /dev/null; then
    echo -e "${GREEN}✓${NC} aany-pool command is available"
else
    echo -e "${YELLOW}⚠${NC} aany-pool command not found (built into aany)"
    
    # Check if built locally
    if [ -f "./target/release/aany-pool" ]; then
        echo -e "${YELLOW}ℹ${NC}  Found locally built binary at ./target/release/aany-pool"
    fi
fi

echo

# Check environment
echo "Checking environment configuration..."

# Load .env if exists
if [ -f ".env" ]; then
    source .env
    echo -e "${GREEN}✓${NC} .env file found and loaded"
else
    echo -e "${YELLOW}⚠${NC} No .env file found (optional)"
    ((warnings++))
fi

# Check AANY_REPO_PATH
if [ -n "$AANY_REPO_PATH" ]; then
    if [ -d "$AANY_REPO_PATH" ]; then
        echo -e "${GREEN}✓${NC} AANY_REPO_PATH is set and valid: $AANY_REPO_PATH"
    else
        echo -e "${RED}✗${NC} AANY_REPO_PATH is set but directory doesn't exist: $AANY_REPO_PATH"
        ((errors++))
    fi
else
    echo -e "${YELLOW}⚠${NC} AANY_REPO_PATH not set (scripts must be in PATH)"
    ((warnings++))
fi

echo

# Summary
echo "=== Summary ==="
if [ $errors -eq 0 ]; then
    if [ $warnings -eq 0 ]; then
        echo -e "${GREEN}✓ All checks passed!${NC} Agent Anywhere is ready to use."
        echo
        echo "Get started with:"
        echo "  aany pool        # Launch the agent pool manager"
        echo "  aany --help      # Show available commands"
    else
        echo -e "${GREEN}✓ Core installation successful${NC} with $warnings warnings."
        echo
        echo "Agent Anywhere will work, but consider:"
        echo "- Installing git for repository cloning"
        echo "- Creating a .env file from .env.example"
        echo "- Setting AANY_REPO_PATH if scripts aren't in PATH"
    fi
else
    echo -e "${RED}✗ Installation has $errors error(s)${NC}"
    echo
    echo "Please fix the errors above before using Agent Anywhere."
    echo "See docs/GETTING_STARTED.md for installation instructions."
    exit 1
fi

echo
echo "For configuration options, see:"
echo "- .env.example"
echo "- docs/ENVIRONMENT_VARIABLES.md"