# aany-hub Development Notes

## Virtual Environment

**IMPORTANT**: This project uses `.venv` (not `venv`)

```bash
# Always use
source .venv/bin/activate

# Or use the helper
source activate.sh
```

## Environment Variables

All configuration is in `.env` file:
- `AANY_HUB_VENV=.venv` - Virtual environment path
- `AANY_HUB_GRPC_PORT=50052` - gRPC server port
- `AANY_HUB_HTTP_PORT=8090` - HTTP dashboard port

## Running the Server

```bash
# Preferred method
./run_server.sh

# Or manually
source .venv/bin/activate
python -m src.aany_hub.simple_server  # For gRPC only
python -m src.aany_hub.main          # For full server with dashboard
```

## Common Mistakes to Avoid

1. **Wrong venv**: Always use `.venv`, not `venv`
2. **Relative paths**: Log directory should use absolute paths
3. **Process cleanup**: Use `kill -9` to ensure Python processes are terminated
4. **Import paths**: Run as module from src directory: `python -m aany_hub.module`