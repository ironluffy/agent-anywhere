#!/bin/bash
# Build script for macOS release

set -e

echo "🍎 Building tmux-agent for macOS..."
echo ""

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "⚠️  This script should be run on macOS for native builds"
    echo "   For cross-compilation, use: cargo build --target x86_64-apple-darwin"
    exit 1
fi

# Check dependencies
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

if ! command -v tmux &> /dev/null; then
    echo "❌ tmux not found. Please install:"
    echo "   brew install tmux"
    exit 1
fi

VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
echo "📦 Building version $VERSION"

# Build for Intel Macs
echo "🔨 Building for x86_64 (Intel)..."
rustup target add x86_64-apple-darwin 2>/dev/null || true
cargo build --release --target x86_64-apple-darwin --bin tmux-agent

# Build for Apple Silicon
echo "🔨 Building for aarch64 (Apple Silicon)..."
rustup target add aarch64-apple-darwin 2>/dev/null || true
cargo build --release --target aarch64-apple-darwin --bin tmux-agent

# Create universal binary
echo "🔗 Creating universal binary..."
mkdir -p target/release-universal

lipo -create \
    target/x86_64-apple-darwin/release/tmux-agent \
    target/aarch64-apple-darwin/release/tmux-agent \
    -output target/release-universal/tmux-agent

chmod +x target/release-universal/tmux-agent

# Create tarball for release
echo "📦 Creating release tarball..."
mkdir -p dist
cd dist

# Create directory structure
mkdir -p tmux-agent-v$VERSION-macos
cp ../target/release-universal/tmux-agent tmux-agent-v$VERSION-macos/
cp ../README.md tmux-agent-v$VERSION-macos/
cp ../TMUX_AGENT_README.md tmux-agent-v$VERSION-macos/
mkdir -p tmux-agent-v$VERSION-macos/completions
cp ../completions/* tmux-agent-v$VERSION-macos/completions/ 2>/dev/null || true

# Create tarball
tar -czf tmux-agent-v$VERSION-macos.tar.gz tmux-agent-v$VERSION-macos
rm -rf tmux-agent-v$VERSION-macos

# Calculate SHA256
echo ""
echo "📝 SHA256 checksum:"
shasum -a 256 tmux-agent-v$VERSION-macos.tar.gz

cd ..

echo ""
echo "✅ Build complete!"
echo "   Binary: target/release-universal/tmux-agent"
echo "   Release: dist/tmux-agent-v$VERSION-macos.tar.gz"
echo ""
echo "📤 To create a GitHub release:"
echo "   1. Create a tag: git tag v$VERSION"
echo "   2. Push tag: git push origin v$VERSION"
echo "   3. Upload dist/tmux-agent-v$VERSION-macos.tar.gz to the release"
echo "   4. Update the SHA256 in homebrew-tap/Formula/tmux-agent.rb"