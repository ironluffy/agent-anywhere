#!/bin/bash
# Full deployment workflow for tmux-agent
set -e

echo "🚀 TMux Agent Deployment Workflow"
echo "================================="
echo ""

# Get version from Cargo.toml
VERSION=$(grep "^version" aany-tmux/Cargo.toml | head -1 | cut -d'"' -f2)
echo "📦 Version: $VERSION"

# Check if we're on the right branch
CURRENT_BRANCH=$(git branch --show-current)
echo "🌿 Current branch: $CURRENT_BRANCH"

# Function to build Rust binaries
build_rust_binaries() {
    echo ""
    echo "🔨 Building Rust binaries..."
    cd aany-tmux
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "🍎 Building macOS universal binary..."
        ./build-macos-release.sh
    else
        echo "🐧 Building Linux binary..."
        cargo build --release --bin tmux-agent
        mkdir -p dist
        tar -czf dist/tmux-agent-v$VERSION-linux-x86_64.tar.gz \
            -C target/release tmux-agent \
            -C ../.. README.md TMUX_AGENT_README.md
    fi
    
    cd ..
}

# Function to build Python wheels
build_python_wheels() {
    echo ""
    echo "🐍 Building Python wheels..."
    cd python
    
    # Install build dependencies
    pip install build wheel setuptools-rust
    
    # Clean old builds
    rm -rf dist build *.egg-info
    
    # Build wheel
    echo "📦 Building wheel for current platform..."
    python -m build --wheel
    
    # On macOS, create universal wheel if possible
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "🍎 Creating macOS universal wheel..."
        pip install delocate
        delocate-wheel -w dist -v dist/*.whl
    fi
    
    # On Linux, create manylinux wheel
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "🐧 Creating manylinux wheel..."
        pip install auditwheel
        auditwheel repair dist/*.whl -w dist/
        rm dist/*-linux_*.whl  # Remove non-manylinux
    fi
    
    cd ..
}

# Function to test installation
test_installation() {
    echo ""
    echo "🧪 Testing installation..."
    
    # Create test virtualenv
    echo "📦 Creating test environment..."
    python -m venv test_env
    source test_env/bin/activate || source test_env/Scripts/activate
    
    # Test wheel installation
    echo "🔧 Installing from wheel..."
    pip install python/dist/*.whl
    
    # Test commands
    echo "✅ Testing tmux-agent command..."
    tmux-agent version
    
    # Cleanup
    deactivate
    rm -rf test_env
}

# Function to prepare release
prepare_release() {
    echo ""
    echo "📋 Preparing release..."
    
    # Check for uncommitted changes
    if ! git diff-index --quiet HEAD --; then
        echo "❌ Error: You have uncommitted changes!"
        echo "   Please commit or stash them first."
        exit 1
    fi
    
    # Create release notes
    cat > RELEASE_NOTES.md << EOF
# Release v$VERSION

## Installation

### Python Package (Recommended)
\`\`\`bash
# From GitHub Release wheel (no compilation needed!)
pip install https://github.com/jayhansuh/agent-anywhere/releases/download/v$VERSION/tmux_agent-$VERSION-py3-none-any.whl

# From source (requires Rust)
pip install git+https://github.com/jayhansuh/agent-anywhere.git@v$VERSION#subdirectory=python
\`\`\`

### Rust Binary
\`\`\`bash
# macOS
curl -L https://github.com/jayhansuh/agent-anywhere/releases/download/v$VERSION/tmux-agent-v$VERSION-macos.tar.gz | tar xz
sudo mv tmux-agent /usr/local/bin/

# Linux
curl -L https://github.com/jayhansuh/agent-anywhere/releases/download/v$VERSION/tmux-agent-v$VERSION-linux-x86_64.tar.gz | tar xz
sudo mv tmux-agent /usr/local/bin/
\`\`\`

## What's New
- Agent-friendly tmux wrapper with safety controls
- Read-only monitoring mode
- Human interference detection
- Pre-built wheels for instant installation

## Checksums
\`\`\`
$(cd aany-tmux/dist 2>/dev/null && shasum -a 256 *.tar.gz 2>/dev/null || echo "Rust binaries not built yet")
$(cd python/dist 2>/dev/null && shasum -a 256 *.whl 2>/dev/null || echo "Python wheels not built yet")
\`\`\`
EOF
    
    echo "📝 Release notes created: RELEASE_NOTES.md"
}

# Function to create git tag
create_tag() {
    echo ""
    echo "🏷️  Creating git tag v$VERSION..."
    
    if git tag | grep -q "^v$VERSION$"; then
        echo "⚠️  Tag v$VERSION already exists!"
        read -p "Delete and recreate? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            git tag -d v$VERSION
            git push origin :refs/tags/v$VERSION 2>/dev/null || true
        else
            echo "Skipping tag creation."
            return
        fi
    fi
    
    git tag -a v$VERSION -m "Release v$VERSION"
    echo "✅ Tag created: v$VERSION"
}

# Main menu
echo ""
echo "Choose deployment steps:"
echo "1) Full deployment (all steps)"
echo "2) Build Rust binaries only"
echo "3) Build Python wheels only"
echo "4) Test installation"
echo "5) Prepare release + tag"
echo "6) Push to GitHub"
echo "7) Publish to PyPI"
echo ""
read -p "Enter choice [1-7]: " choice

case $choice in
    1)
        build_rust_binaries
        build_python_wheels
        test_installation
        prepare_release
        create_tag
        echo ""
        echo "✅ Ready for deployment!"
        echo "   Next: ./deploy.sh and choose option 6 to push"
        ;;
    2)
        build_rust_binaries
        ;;
    3)
        build_python_wheels
        ;;
    4)
        test_installation
        ;;
    5)
        prepare_release
        create_tag
        ;;
    6)
        echo "📤 Pushing to GitHub..."
        git push origin $CURRENT_BRANCH
        git push origin v$VERSION
        echo "✅ Pushed! Check GitHub Actions for wheel building."
        echo "   https://github.com/jayhansuh/agent-anywhere/actions"
        ;;
    7)
        echo "📦 Publishing to PyPI..."
        cd python
        pip install twine
        twine upload dist/*
        cd ..
        echo "✅ Published to PyPI!"
        echo "   pip install tmux-agent"
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac