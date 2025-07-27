#!/bin/bash
# Test script to verify environment variable handling

echo "=== Agent Anywhere Environment Variable Test ==="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test function
test_env_var() {
    local var_name=$1
    local expected_behavior=$2
    local test_value=$3
    
    echo -n "Testing $var_name: "
    
    if [ -n "${!var_name}" ]; then
        echo -e "${GREEN}✓${NC} Set to: ${!var_name}"
    else
        echo -e "${YELLOW}○${NC} Not set (using default behavior)"
    fi
}

echo "1. System Environment Variables:"
echo "================================"

test_env_var "AANY_REPO_PATH" "Path to agent-anywhere repository"
test_env_var "AANY_HUB_URL" "gRPC hub URL (default: localhost:50052)"
test_env_var "AANY_LOGGING_DISABLED" "Disable logging (default: false)"

echo
echo "2. Script Path Overrides:"
echo "========================="

test_env_var "AANY_CLAUDE_WRAPPER" "Path to claude-with-return.sh"
test_env_var "AANY_TMUX_LOGGER" "Path to tmux-logger.sh"
test_env_var "AANY_INTERACTION_LOGGER" "Path to tmux-interaction-logger.sh"
test_env_var "AANY_SCREENSHOT_LOGGER" "Path to tmux-screenshot-rotating-logger.sh"

echo
echo "3. Internal Variables (set by system):"
echo "======================================"

test_env_var "GRPC_ENABLE_FORK_SUPPORT" "gRPC fork support"
test_env_var "GRPC_POLL_STRATEGY" "gRPC polling strategy"

echo
echo "4. Testing Script Discovery:"
echo "============================"

# Function to test script discovery
find_script() {
    local script_name=$1
    local env_var=$2
    
    echo -n "Looking for $script_name: "
    
    # Check environment variable
    if [ -n "${!env_var}" ] && [ -f "${!env_var}" ]; then
        echo -e "${GREEN}✓${NC} Found via $env_var: ${!env_var}"
        return 0
    fi
    
    # Check AANY_REPO_PATH
    if [ -n "$AANY_REPO_PATH" ]; then
        local repo_script="$AANY_REPO_PATH/aany-tmux/$script_name"
        if [ -f "$repo_script" ]; then
            echo -e "${GREEN}✓${NC} Found in AANY_REPO_PATH: $repo_script"
            return 0
        fi
    fi
    
    # Check PATH
    if command -v "$script_name" &> /dev/null; then
        echo -e "${GREEN}✓${NC} Found in PATH: $(which $script_name)"
        return 0
    fi
    
    # Check common locations
    local common_paths=(
        "/usr/local/bin/$script_name"
        "/opt/homebrew/bin/$script_name"
        "$HOME/.local/bin/$script_name"
        "$HOME/bin/$script_name"
    )
    
    for path in "${common_paths[@]}"; do
        if [ -f "$path" ]; then
            echo -e "${GREEN}✓${NC} Found at: $path"
            return 0
        fi
    done
    
    echo -e "${RED}✗${NC} Not found"
    return 1
}

find_script "claude-with-return.sh" "AANY_CLAUDE_WRAPPER"
find_script "tmux-logger.sh" "AANY_TMUX_LOGGER"
find_script "tmux-interaction-logger.sh" "AANY_INTERACTION_LOGGER"
find_script "tmux-screenshot-rotating-logger.sh" "AANY_SCREENSHOT_LOGGER"

echo
echo "5. Checking for .env file:"
echo "=========================="

if [ -f ".env" ]; then
    echo -e "${GREEN}✓${NC} .env file found"
    echo "   Contents:"
    while IFS= read -r line; do
        # Skip empty lines and comments
        if [[ -n "$line" && ! "$line" =~ ^[[:space:]]*# ]]; then
            echo "   - $line"
        fi
    done < .env
else
    echo -e "${YELLOW}○${NC} No .env file found (using defaults)"
fi

echo
echo "6. Test Summary:"
echo "================"

# Check if any critical scripts are missing
critical_missing=0
for script in "tmux-logger.sh" "tmux-interaction-logger.sh" "tmux-screenshot-rotating-logger.sh"; do
    if ! command -v "$script" &> /dev/null && [ -z "$AANY_REPO_PATH" ]; then
        ((critical_missing++))
    fi
done

if [ $critical_missing -gt 0 ]; then
    echo -e "${YELLOW}⚠${NC}  Some scripts not found. Consider setting AANY_REPO_PATH or installing scripts to PATH"
else
    echo -e "${GREEN}✓${NC} All scripts are discoverable"
fi

if [ "$AANY_LOGGING_DISABLED" = "true" ]; then
    echo -e "${YELLOW}ℹ${NC}  Logging is disabled (AANY_LOGGING_DISABLED=true)"
fi

echo
echo "For more information, see docs/ENVIRONMENT_VARIABLES.md"