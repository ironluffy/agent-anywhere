# Protobuf and gRPC Troubleshooting Guide

This document covers common issues encountered with Protocol Buffers (protobuf) and gRPC in the aany-hub project.

## Common Errors and Solutions

### 1. AttributeError: module 'agent_pb2' has no attribute 'RegisterResponse'

**Error:**
```
AttributeError: module 'aany_hub.agent_pb2' has no attribute 'RegisterResponse'
```

**Cause:** The actual class name in the proto file is `RegistrationResponse` (not `RegisterResponse`). This is a naming mismatch.

**Solution:** Use the correct class name as defined in your `.proto` file.

### 2. ImportError: attempted relative import with no known parent package

**Error:**
```
ImportError: attempted relative import with no known parent package
```

**Cause:** Running Python files directly instead of as modules, or incorrect import paths in generated files.

**Solution:**
1. Run as module: `python -m aany_hub.module_name`
2. Fix imports in generated files:
   ```python
   # Change from:
   import agent_pb2 as agent__pb2
   # To:
   from . import agent_pb2 as agent__pb2
   ```

### 3. Protobuf Version Mismatch

**Error:**
```
google.protobuf.runtime_version.VersionError: Detected mismatched Protobuf Gencode/Runtime major versions when loading agent.proto: gencode 6.31.1 runtime 5.28.3
```

**Cause:** The protobuf files were generated with a different version than what's installed in the runtime environment.

**Solution:**
1. Check installed version:
   ```bash
   pip show protobuf
   ```

2. Regenerate proto files with matching version:
   ```bash
   # Install specific version
   pip install protobuf==5.28.3
   
   # Regenerate files
   python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. agent.proto
   ```

3. Or upgrade to match generated version:
   ```bash
   pip install protobuf==6.31.1
   ```

### 4. Module 'google.protobuf' has no attribute 'runtime_version'

**Error:**
```
ImportError: cannot import name 'runtime_version' from 'google.protobuf'
```

**Cause:** Older protobuf versions (< 5.26.0) don't have the `runtime_version` module.

**Solution:**
1. Downgrade protobuf to a compatible version:
   ```bash
   pip install protobuf==5.28.3 grpcio-tools==1.62.0
   ```

2. Or regenerate with older protoc that doesn't use runtime_version checks.

### 5. Multiple Python Processes on Same Port

**Symptom:** Server appears to start but doesn't actually work. Multiple Python processes listening on the same port.

**Diagnosis:**
```bash
lsof -i :50052
# Shows multiple Python processes
```

**Cause:** Ctrl+C doesn't always kill async Python processes cleanly.

**Solution:**
```bash
# Force kill all processes on port
lsof -ti :50052 | xargs kill -9

# Or kill specific PIDs
ps aux | grep python.*aany_hub | grep -v grep
kill -9 <PID1> <PID2>
```

## Best Practices

### 1. Version Pinning

Always pin your protobuf and grpcio versions in requirements.txt:
```
protobuf==5.28.3
grpcio==1.68.1
grpcio-tools==1.68.1
```

### 2. Proto Compilation Script

Create a consistent compilation script:
```python
#!/usr/bin/env python3
import subprocess
import sys
from pathlib import Path

def compile_protos():
    proto_dir = Path("src/aany_hub/proto")
    output_dir = Path("src/aany_hub")
    
    for proto_file in proto_dir.glob("*.proto"):
        cmd = [
            sys.executable, "-m", "grpc_tools.protoc",
            f"-I{proto_dir}",
            f"--python_out={output_dir}",
            f"--grpc_python_out={output_dir}",
            str(proto_file)
        ]
        subprocess.run(cmd, check=True)
        print(f"Compiled {proto_file.name}")

if __name__ == "__main__":
    compile_protos()
```

### 3. Import Path Management

Always run from the correct directory:
```bash
# Good - run as module from src directory
cd src
python -m aany_hub.server

# Bad - run file directly
python src/aany_hub/server.py
```

### 4. Virtual Environment Isolation

Use separate virtual environments for different protobuf versions:
```bash
# Create venv with specific Python version
python3.9 -m venv .venv
source .venv/bin/activate

# Install exact versions
pip install -r requirements.txt
```

## Debugging Tips

1. **Check protobuf version compatibility:**
   ```python
   import google.protobuf
   print(google.protobuf.__version__)
   ```

2. **Verify generated files:**
   ```python
   # Check what's actually in the module
   from aany_hub import agent_pb2
   print(dir(agent_pb2))
   ```

3. **Test imports in isolation:**
   ```python
   # Test if basic imports work
   python -c "from aany_hub import agent_pb2; print('Import successful')"
   ```

4. **Use absolute imports in generated files:**
   Sometimes you need to manually fix imports in generated `_pb2_grpc.py` files.

## Common Workarounds

### HTTP-Only Server (No gRPC)

If gRPC issues persist, you can use an HTTP-only server as a workaround:
```python
# working_hub.py - Simple HTTP API without gRPC
from fastapi import FastAPI
from pydantic import BaseModel

app = FastAPI()

@app.post("/api/logs")
async def receive_logs(logs: list[dict]):
    # Store logs without protobuf
    pass
```

### Simple gRPC Server

For testing, use a minimal gRPC server without complex dependencies:
```python
# simple_server.py - Minimal gRPC implementation
import grpc
from concurrent import futures

# Direct imports, no relative paths
import agent_pb2
import agent_pb2_grpc

class SimpleHub(agent_pb2_grpc.AgentHubServicer):
    # Minimal implementation
    pass
```

## References

- [Protobuf Python Generated Code Guide](https://protobuf.dev/reference/python/python-generated/)
- [gRPC Python Quickstart](https://grpc.io/docs/languages/python/quickstart/)
- [Protobuf Version Compatibility](https://protobuf.dev/support/cross-version-runtime-guarantee/)
- [Common gRPC Python Issues](https://github.com/grpc/grpc/issues?q=is%3Aissue+label%3A%22lang%2Fpython%22)