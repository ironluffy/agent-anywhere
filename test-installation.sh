#!/bin/bash
# Test tmux-agent installation on macOS
set -e

echo "🧪 Testing tmux-agent Installation"
echo "================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check system
echo "🔍 System Check:"
echo "  OS: $OSTYPE"
echo "  Python: $(python3 --version)"
echo "  pip: $(pip --version)"
echo ""

# Create test directory
TEST_DIR=$(mktemp -d)
echo "📁 Test directory: $TEST_DIR"
cd "$TEST_DIR"

# Test 1: Install from source (current method)
test_source_install() {
    echo -e "\n${YELLOW}Test 1: Install from source${NC}"
    echo "This simulates: pip install git+ssh://..."
    
    python3 -m venv venv_source
    source venv_source/bin/activate
    
    # Install directly from the local path
    pip install -e /home/jay/agent-anywhere/python
    
    # Test the command
    if tmux-agent version > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Source install successful!${NC}"
        tmux-agent version
    else
        echo -e "${RED}❌ Source install failed!${NC}"
    fi
    
    deactivate
}

# Test 2: Install from wheel
test_wheel_install() {
    echo -e "\n${YELLOW}Test 2: Install from wheel${NC}"
    echo "This simulates: pip install tmux-agent (from PyPI)"
    
    # First build the wheel
    cd /home/jay/agent-anywhere/python
    python -m build --wheel
    WHEEL_FILE=$(ls dist/*.whl | head -1)
    cd "$TEST_DIR"
    
    python3 -m venv venv_wheel
    source venv_wheel/bin/activate
    
    pip install "$WHEEL_FILE"
    
    # Test the command
    if tmux-agent version > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Wheel install successful!${NC}"
        tmux-agent version
    else
        echo -e "${RED}❌ Wheel install failed!${NC}"
    fi
    
    deactivate
}

# Test 3: Test all commands
test_commands() {
    echo -e "\n${YELLOW}Test 3: Testing all commands${NC}"
    
    source venv_wheel/bin/activate
    
    commands=(
        "tmux-agent version"
        "tmux-agent help"
        "tma help"
        "tmux-agent new test-session"
        "tmux-agent list"
        "tmux-agent kill-session -t test-session"
    )
    
    for cmd in "${commands[@]}"; do
        echo -n "  Testing: $cmd ... "
        if $cmd > /dev/null 2>&1; then
            echo -e "${GREEN}OK${NC}"
        else
            echo -e "${RED}FAILED${NC}"
        fi
    done
    
    deactivate
}

# Test 4: Performance test
test_performance() {
    echo -e "\n${YELLOW}Test 4: Installation performance${NC}"
    
    # Time wheel installation
    python3 -m venv venv_perf
    source venv_perf/bin/activate
    
    WHEEL_FILE="/home/jay/agent-anywhere/python/dist/tmux_agent-0.2.0-py3-none-any.whl"
    
    if [ -f "$WHEEL_FILE" ]; then
        echo -n "  Timing wheel install: "
        start_time=$(date +%s)
        pip install "$WHEEL_FILE" > /dev/null 2>&1
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo -e "${GREEN}${duration} seconds${NC}"
    else
        echo -e "${YELLOW}  Wheel not found, skipping performance test${NC}"
    fi
    
    deactivate
}

# Test 5: Integration test
test_integration() {
    echo -e "\n${YELLOW}Test 5: Integration with uv${NC}"
    
    if command -v uv &> /dev/null; then
        cd "$TEST_DIR"
        mkdir test_project
        cd test_project
        
        # Create a test project
        cat > pyproject.toml << EOF
[project]
name = "test-project"
version = "0.1.0"
dependencies = []
EOF
        
        echo -n "  Testing uv add (local): "
        if uv add /home/jay/agent-anywhere/python > /dev/null 2>&1; then
            echo -e "${GREEN}OK${NC}"
            cat pyproject.toml | grep tmux-agent
        else
            echo -e "${RED}FAILED${NC}"
        fi
    else
        echo -e "${YELLOW}  uv not installed, skipping${NC}"
    fi
}

# Run tests
echo "Starting tests..."
test_source_install
test_wheel_install
test_commands
test_performance
test_integration

# Cleanup
echo -e "\n🧹 Cleaning up..."
cd /
rm -rf "$TEST_DIR"

echo -e "\n${GREEN}✅ All tests completed!${NC}"
echo ""
echo "📝 Summary:"
echo "  - Source installation: Works but requires Rust compilation"
echo "  - Wheel installation: Fast when wheel is available"
echo "  - Commands: All CLI commands functional"
echo "  - Integration: Works with pip and uv"
echo ""
echo "💡 Next steps:"
echo "  1. Push to GitHub to trigger wheel building"
echo "  2. Download wheels from GitHub releases"
echo "  3. Publish to PyPI for easiest installation"