#!/bin/bash

echo "🤖 TMux-Agent Demo"
echo "=================="
echo ""
echo "This demo shows tmux-agent in action!"
echo ""

# Ensure tmux-agent is built
cd "$(dirname "$0")"
source "$HOME/.cargo/env" 2>/dev/null || true

if [ ! -f "./target/release/tmux-agent" ]; then
    echo "Building tmux-agent first..."
    cargo build --release --bin tmux-agent
fi

TMUX_AGENT="./target/release/tmux-agent"

echo "1️⃣ Creating a new agent session..."
$TMUX_AGENT new demo-agent
echo ""

echo "2️⃣ Checking system status..."
$TMUX_AGENT status
echo ""

echo "3️⃣ Checking session health..."
$TMUX_AGENT health demo-agent
echo ""

echo "4️⃣ Protecting the session..."
$TMUX_AGENT protect demo-agent
echo ""

echo "5️⃣ Available commands:"
echo "   Monitor safely:     $TMUX_AGENT monitor demo-agent"
echo "   Attach (careful!):  $TMUX_AGENT attach demo-agent"
echo "   List sessions:      $TMUX_AGENT list"
echo "   Clean up:           $TMUX_AGENT cleanup demo-agent"
echo "   Kill session:       $TMUX_AGENT kill demo-agent"
echo ""

echo "📝 Try monitoring the session in another terminal:"
echo "   $TMUX_AGENT monitor demo-agent"
echo ""
echo "✨ tmux-agent makes tmux safer for automated agents!"