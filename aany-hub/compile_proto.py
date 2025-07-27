#!/usr/bin/env python3
"""Compile protobuf files"""

import subprocess
import sys
from pathlib import Path

def compile_proto():
    """Compile proto files to Python"""
    proto_dir = Path("src/aany_hub/proto")
    out_dir = Path("src/aany_hub")
    
    proto_files = list(proto_dir.glob("*.proto"))
    
    if not proto_files:
        print("No proto files found")
        return False
    
    for proto_file in proto_files:
        print(f"Compiling {proto_file}...")
        
        cmd = [
            sys.executable, "-m", "grpc_tools.protoc",
            f"-I{proto_dir}",
            f"--python_out={out_dir}",
            f"--grpc_python_out={out_dir}",
            str(proto_file)
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"Error compiling {proto_file}:")
            print(result.stderr)
            return False
    
    print("Proto files compiled successfully")
    
    # Fix imports
    from fix_proto_imports import fix_imports
    fix_imports()
    
    return True

if __name__ == "__main__":
    compile_proto()