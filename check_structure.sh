#!/bin/bash
# Check project structure

echo "🔍 Checking Agent Anywhere project structure..."
echo ""

# Check directories
echo "📁 Directory Structure:"
for dir in aany aany-tmux aany-pool python; do
    if [ -d "$dir" ]; then
        echo "  ✅ $dir/"
    else
        echo "  ❌ $dir/ (missing)"
    fi
done

echo ""
echo "📄 Key Files:"

# Check key files
files=(
    "Cargo.toml"
    "aany/Cargo.toml"
    "aany-tmux/Cargo.toml"
    "aany-pool/Cargo.toml"
    "aany/src/main.rs"
    "aany/src/commands/tmux.rs"
    "aany/src/commands/pool.rs"
    "aany/src/commands/auth.rs"
    "aany-tmux/src/lib.rs"
    "aany-pool/src/lib.rs"
    "build.sh"
    "README.md"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "  ✅ $file"
    else
        echo "  ❌ $file (missing)"
    fi
done

echo ""
echo "📋 Cargo.toml workspace check:"
if grep -q '"aany"' Cargo.toml && grep -q '"aany-tmux"' Cargo.toml && grep -q '"aany-pool"' Cargo.toml; then
    echo "  ✅ Workspace configuration looks good"
    echo "  Members: $(grep -A 5 'members' Cargo.toml | grep '"' | tr '\n' ' ')"
else
    echo "  ❌ Workspace configuration issue"
fi

echo ""
echo "🎯 Command Structure:"
echo "  aany"
echo "  ├── tmux     (manage tmux sessions)"
echo "  ├── pool     (manage agent pools)"
echo "  ├── auth     (authentication)"
echo "  ├── init     (initialize workspace)"
echo "  └── config   (configuration)"

echo ""
echo "✅ Structure check complete!"