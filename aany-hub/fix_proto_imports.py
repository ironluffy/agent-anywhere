#!/usr/bin/env python3
"""Fix imports in generated protobuf files"""

import re
from pathlib import Path

def fix_imports():
    """Fix relative imports in generated proto files"""
    
    # Fix agent_pb2_grpc.py
    grpc_file = Path("src/aany_hub/agent_pb2_grpc.py")
    if grpc_file.exists():
        content = grpc_file.read_text()
        # Replace the import statement
        content = re.sub(
            r'^import agent_pb2 as agent__pb2$',
            'from . import agent_pb2 as agent__pb2',
            content,
            flags=re.MULTILINE
        )
        grpc_file.write_text(content)
        print(f"Fixed imports in {grpc_file}")
    
    return True

if __name__ == "__main__":
    fix_imports()