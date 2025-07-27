#!/bin/bash
# Helper script to activate the correct virtual environment

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Use venv from environment or default to .venv
VENV_PATH="${AANY_HUB_VENV:-.venv}"

if [ ! -d "$VENV_PATH" ]; then
    echo "Virtual environment not found at $VENV_PATH"
    echo "Run: python3 -m venv $VENV_PATH"
    exit 1
fi

echo "Activating $VENV_PATH..."
source "$VENV_PATH/bin/activate"

# Show which Python we're using
echo "Using Python: $(which python)"
echo "Python version: $(python --version)"