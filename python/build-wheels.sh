#!/bin/bash
# Build wheels for different platforms locally

set -e

echo "🔨 Building Python wheels for tmux-agent..."

# Install build dependencies
pip install build wheel setuptools-rust auditwheel

# Clean previous builds
rm -rf dist build *.egg-info

# Build the wheel
python -m build --wheel

# On Linux, create manylinux wheel
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "📦 Creating manylinux wheel..."
    auditwheel show dist/*.whl
    auditwheel repair dist/*.whl -w dist/
    # Remove the non-manylinux wheel
    rm dist/*-linux_*.whl
fi

echo "✅ Wheels built successfully!"
ls -la dist/

echo ""
echo "📤 To upload to PyPI:"
echo "   pip install twine"
echo "   twine upload dist/*"