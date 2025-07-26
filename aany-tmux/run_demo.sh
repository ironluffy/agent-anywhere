#!/bin/bash

echo "🤖 TMUX AGENT PROXY DEMOS"
echo "========================"
echo ""
echo "Choose a demo to run:"
echo ""
echo "1) 📺 Monitor - Watch the agent handle different scenarios automatically"
echo "2) 🎓 Guided - Step-by-step walkthrough with explanations"
echo "3) 🎮 Interactive - Attach to tmux and interfere with the agent"
echo "4) 👁️  Read-Only - Demonstrates safe monitoring with -r flag"
echo ""
read -p "Enter choice (1-4): " choice

cd "$(dirname "$0")"
source "$HOME/.cargo/env"

case $choice in
    1)
        echo ""
        echo "Starting Monitor Demo..."
        echo "This will show you various scenarios automatically."
        echo ""
        cargo run --bin monitor
        ;;
    2)
        echo ""
        echo "Starting Guided Demo..."
        echo "This will walk you through each feature step by step."
        echo ""
        cargo run --bin guided_demo
        ;;
    3)
        echo ""
        echo "Starting Interactive Demo..."
        echo "You'll need to open another terminal to interact with tmux."
        echo ""
        cargo run --bin interactive_demo
        ;;
    4)
        echo ""
        echo "Starting Read-Only Demo..."
        echo "This shows the difference between read-only (-r) and interactive modes."
        echo ""
        cargo run --bin readonly_demo
        ;;
    *)
        echo "Invalid choice. Please run again and select 1, 2, 3, or 4."
        exit 1
        ;;
esac