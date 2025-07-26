"""
Setup script for tmux-agent Python wrapper.
This builds the Rust binary and packages it with Python.
"""
import os
import sys
import subprocess
import platform
from pathlib import Path
from setuptools import setup, find_packages
from setuptools.command.build_ext import build_ext
from setuptools.command.install import install


class BuildRustBinary(build_ext):
    """Custom build command to compile Rust binary."""
    
    def run(self):
        # Check if cargo is available
        try:
            subprocess.run(["cargo", "--version"], check=True, capture_output=True)
        except (subprocess.CalledProcessError, FileNotFoundError):
            print("Error: Rust/Cargo not found!")
            print("Installing Rust automatically...")
            self.install_rust()
        
        # Build the Rust binary
        print("Building tmux-agent binary...")
        rust_project = Path(__file__).parent.parent / "aany-tmux"
        
        if not rust_project.exists():
            raise RuntimeError(
                f"Rust project not found at {rust_project}. "
                "Make sure you're installing from the full repository."
            )
        
        # Build release binary
        subprocess.run(
            ["cargo", "build", "--release", "--bin", "tmux-agent"],
            cwd=rust_project,
            check=True
        )
        
        # Copy binary to package
        binary_name = "tmux-agent.exe" if platform.system() == "Windows" else "tmux-agent"
        source = rust_project / "target" / "release" / binary_name
        
        # Create bin directory in package
        bin_dir = Path(__file__).parent / "tmux_agent" / "bin"
        bin_dir.mkdir(parents=True, exist_ok=True)
        
        dest = bin_dir / binary_name
        print(f"Copying {source} to {dest}")
        dest.write_bytes(source.read_bytes())
        dest.chmod(0o755)
    
    def install_rust(self):
        """Install Rust using rustup."""
        if platform.system() == "Windows":
            print("Please install Rust manually from https://rustup.rs/")
            sys.exit(1)
        else:
            # Unix-like systems
            rustup_cmd = (
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | "
                "sh -s -- -y --profile minimal --default-toolchain stable"
            )
            subprocess.run(rustup_cmd, shell=True, check=True)
            
            # Add cargo to PATH for current session
            cargo_home = os.environ.get("CARGO_HOME", os.path.expanduser("~/.cargo"))
            os.environ["PATH"] = f"{cargo_home}/bin:{os.environ['PATH']}"


class CustomInstall(install):
    """Custom install command."""
    
    def run(self):
        # Check for tmux
        try:
            subprocess.run(["tmux", "-V"], check=True, capture_output=True)
        except (subprocess.CalledProcessError, FileNotFoundError):
            print("\n⚠️  WARNING: tmux not found!")
            print("Please install tmux:")
            print("  Ubuntu/Debian: sudo apt-get install tmux")
            print("  macOS: brew install tmux")
            print("  Fedora: sudo dnf install tmux")
            print()
        
        # Run standard install
        install.run(self)
        
        print("\n✅ tmux-agent installed successfully!")
        print("Try: tmux-agent help")


setup(
    cmdclass={
        'build_ext': BuildRustBinary,
        'install': CustomInstall,
    },
    ext_modules=[],  # Needed to trigger build_ext
)