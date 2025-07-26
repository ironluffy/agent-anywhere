#!/bin/bash
# Build script for Agent Anywhere project
set -e

echo "🚀 Agent Anywhere Build Script"
echo "=============================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${YELLOW}→${NC} $1"
}

# Check prerequisites
echo "🔍 Checking prerequisites..."

# Check for Rust
if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo not found!"
    echo "Please install Rust from: https://rustup.rs"
    exit 1
else
    print_status "Rust found: $(rustc --version)"
fi

# Check for tmux
if ! command -v tmux &> /dev/null; then
    print_error "tmux not found!"
    echo "Please install tmux:"
    echo "  Ubuntu/Debian: sudo apt-get install tmux"
    echo "  macOS: brew install tmux"
    echo "  Fedora: sudo dnf install tmux"
    exit 1
else
    print_status "tmux found: $(tmux -V)"
fi

# Parse arguments
BUILD_TYPE="debug"
INSTALL=false
CLEAN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_TYPE="release"
            shift
            ;;
        --install)
            INSTALL=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --release    Build in release mode (optimized)"
            echo "  --install    Install binaries to ~/.cargo/bin"
            echo "  --clean      Clean build artifacts before building"
            echo "  --help       Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Clean if requested
if [ "$CLEAN" = true ]; then
    echo ""
    print_info "Cleaning build artifacts..."
    cargo clean
    print_status "Clean complete"
fi

# Build the project
echo ""
echo "🔨 Building Agent Anywhere..."
echo "Build type: $BUILD_TYPE"

if [ "$BUILD_TYPE" = "release" ]; then
    CARGO_FLAGS="--release"
else
    CARGO_FLAGS=""
fi

# Build all workspace members
print_info "Building workspace..."
cargo build $CARGO_FLAGS

# Build specific binaries
print_info "Building aany CLI..."
cargo build $CARGO_FLAGS --bin aany

print_info "Building tmux-agent binary..."
cargo build $CARGO_FLAGS --bin tmux-agent

print_status "Build complete!"

# Show built binaries
echo ""
echo "📦 Built binaries:"
if [ "$BUILD_TYPE" = "release" ]; then
    BIN_DIR="target/release"
else
    BIN_DIR="target/debug"
fi

ls -lh $BIN_DIR/aany 2>/dev/null && print_status "aany: $BIN_DIR/aany"
ls -lh $BIN_DIR/tmux-agent 2>/dev/null && print_status "tmux-agent: $BIN_DIR/tmux-agent"

# Install if requested
if [ "$INSTALL" = true ]; then
    echo ""
    print_info "Installing binaries..."
    
    cargo install --path aany
    cargo install --path aany-tmux --bin tmux-agent
    
    print_status "Installation complete!"
    echo ""
    echo "Installed to: ~/.cargo/bin/"
    echo "Make sure ~/.cargo/bin is in your PATH"
fi

# Test the binaries
echo ""
echo "🧪 Testing binaries..."

# Test aany
if $BIN_DIR/aany --version &>/dev/null; then
    print_status "aany: $($BIN_DIR/aany --version)"
else
    print_error "aany binary test failed"
fi

# Test tmux-agent
if $BIN_DIR/tmux-agent --version &>/dev/null; then
    print_status "tmux-agent: $($BIN_DIR/tmux-agent --version)"
else
    print_error "tmux-agent binary test failed"
fi

# Show usage examples
echo ""
echo "✅ Build successful!"
echo ""
echo "📚 Usage examples:"
echo ""
echo "  # Using the unified CLI:"
echo "  $BIN_DIR/aany tmux new my-bot"
echo "  $BIN_DIR/aany pool create research-bot"
echo "  $BIN_DIR/aany auth login"
echo ""
echo "  # Or if installed:"
echo "  aany tmux new my-bot"
echo "  aany pool list"
echo ""
echo "  # Traditional tmux-agent:"
echo "  $BIN_DIR/tmux-agent new my-bot"
echo ""

# Development tips
if [ "$BUILD_TYPE" = "debug" ]; then
    echo "💡 Development tips:"
    echo "  - Use --release for optimized builds"
    echo "  - Use --install to install to ~/.cargo/bin"
    echo "  - Set RUST_LOG=debug for detailed logging"
    echo ""
fi