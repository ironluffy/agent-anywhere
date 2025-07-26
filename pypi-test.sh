#!/bin/bash
# Quick test script for PyPI installation
set -e

echo "🧪 Testing PyPI Installation"
echo "=========================="
echo ""

# Create temporary directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Create and activate virtual environment
echo "📦 Creating test environment..."
python3 -m venv pypi_test
source pypi_test/bin/activate

# Test installation from Test PyPI
echo ""
echo "🧪 Test 1: Install from Test PyPI"
echo "--------------------------------"
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ tmux-agent

if tmux-agent version; then
    echo "✅ Test PyPI installation successful!"
else
    echo "❌ Test PyPI installation failed!"
fi

# Clean and test production PyPI
deactivate
rm -rf pypi_test
python3 -m venv pypi_prod
source pypi_prod/bin/activate

echo ""
echo "🧪 Test 2: Install from PyPI"
echo "---------------------------"
pip install tmux-agent

if tmux-agent version; then
    echo "✅ PyPI installation successful!"
else
    echo "❌ PyPI installation failed!"
fi

# Test all commands
echo ""
echo "🧪 Test 3: Command functionality"
echo "-------------------------------"
commands=("tmux-agent help" "tma help" "tmon --help")

for cmd in "${commands[@]}"; do
    echo -n "  Testing '$cmd': "
    if $cmd >/dev/null 2>&1; then
        echo "✅"
    else
        echo "❌"
    fi
done

# Cleanup
deactivate
cd /
rm -rf "$TEMP_DIR"

echo ""
echo "✅ All tests completed!"