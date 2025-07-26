#!/bin/bash
# Simple wheel builder that includes pre-built binary
set -e

echo "🔨 Building wheel with pre-built binary"
echo "======================================"

# Build Rust binary first
echo "1️⃣ Building Rust binary..."
cd aany-tmux
cargo build --release --bin tmux-agent
cd ..

# Create bin directory in Python package
echo "2️⃣ Copying binary to Python package..."
mkdir -p python/tmux_agent/bin
cp aany-tmux/target/release/tmux-agent python/tmux_agent/bin/

# Build wheel
echo "3️⃣ Building Python wheel..."
cd python
python3 -m pip install --upgrade pip build wheel
python3 -m build --wheel --no-isolation || python3 -m build --wheel

# Show result
echo ""
echo "✅ Wheel built!"
ls -la dist/

# Create wheels directory in root for easy access
cd ..
mkdir -p wheels
cp python/dist/*.whl wheels/

echo ""
echo "📦 Wheel available at:"
ls -la wheels/
echo ""
echo "🚀 To install:"
echo "   pip install wheels/$(ls wheels/*.whl | head -1 | xargs basename)"