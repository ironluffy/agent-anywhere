"""Simple setup.py to build wheel with pre-built binary"""
from setuptools import setup, find_packages
from pathlib import Path

setup(
    name="tmux-agent",
    version="0.3.0",
    description="Agent-friendly tmux wrapper with safety controls",
    author="Jay Suh",
    author_email="jayhansuh@gmail.com",
    packages=find_packages(),
    package_data={
        "tmux_agent": ["bin/*"],
    },
    include_package_data=True,
    entry_points={
        "console_scripts": [
            "tmux-agent=tmux_agent:main",
            "tma=tmux_agent:main",
            "tmon=tmux_agent:monitor",
        ],
    },
    python_requires=">=3.8",
    url="https://github.com/jayhansuh/agent-anywhere",
)