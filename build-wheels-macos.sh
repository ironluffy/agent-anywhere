#!/bin/bash
# Build wheels specifically for macOS
set -e

echo "🍎 Building wheels for macOS"
echo "==========================="
echo ""

cd python

# Check Python version
echo "🐍 Python version:"
python3 --version

# Install build dependencies
echo "📦 Installing build dependencies..."
pip install --upgrade pip
pip install build wheel setuptools-rust delocate

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf dist build *.egg-info

# Build the wheel
echo "🔨 Building wheel..."
python -m build --wheel

# Create macOS-specific wheel
echo "🍎 Processing macOS wheel..."
delocate-listdeps dist/*.whl
delocate-wheel -w dist -v dist/*.whl

# Show results
echo ""
echo "✅ Build complete!"
echo "📦 Wheels created:"
ls -la dist/

# Test the wheel
echo ""
echo "🧪 Testing wheel installation..."
python3 -m venv test_venv
source test_venv/bin/activate
pip install dist/*.whl
tmux-agent version
deactivate
rm -rf test_venv

echo ""
echo "✅ Wheel tested successfully!"
echo ""
echo "📤 To test manually:"
echo "   pip install dist/$(ls dist/*.whl | head -1 | xargs basename)"