"""Main entry point for aany-hub server"""

import asyncio
import logging
from concurrent import futures
from typing import Optional

import grpc
import uvicorn
from fastapi import FastAPI
from fastapi.responses import HTMLResponse

from .grpc_server import AgentHubServicer, serve_grpc
from .storage import StorageManager
from .config import Settings
from .websocket_terminal import TerminalWebSocketManager
from .terminal_ui import TERMINAL_HTML

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# FastAPI app
app = FastAPI(title="aany-hub", version="0.1.0")

# Global storage manager
storage_manager: Optional[StorageManager] = None

# Global servicer for agent tracking
hub_servicer: Optional[AgentHubServicer] = None

# Global terminal WebSocket manager
terminal_ws_manager: Optional[TerminalWebSocketManager] = None

@app.get("/terminal", response_class=HTMLResponse)
async def terminal_ui():
    """Terminal UI page"""
    return TERMINAL_HTML

@app.get("/", response_class=HTMLResponse)
async def dashboard():
    """Simple dashboard"""
    return """
    <html>
        <head>
            <title>aany-hub Dashboard</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 20px; }
                .status { color: green; }
                pre { background: #f0f0f0; padding: 10px; }
            </style>
        </head>
        <body>
            <h1>aany-hub Dashboard</h1>
            <p class="status">Status: Running</p>
            <p><a href="/terminal" style="color: #007acc;">Open Terminal UI</a></p>
            <h2>Connected Agents</h2>
            <div id="agents">Loading...</div>
            <h2>Recent Logs</h2>
            <pre id="logs">Loading...</pre>
            
            <script>
                // Simple auto-refresh
                setInterval(() => {
                    fetch('/api/agents')
                        .then(r => r.json())
                        .then(data => {
                            document.getElementById('agents').innerHTML = 
                                data.agents.map(a => `<p>${a.agent_id} - ${a.status}</p>`).join('');
                        });
                    
                    fetch('/api/logs/recent')
                        .then(r => r.json())
                        .then(data => {
                            document.getElementById('logs').innerHTML = 
                                data.logs.map(l => `${l.timestamp} [${l.level}] ${l.agent_id}: ${l.message}`).join('\\n');
                        });
                }, 2000);
            </script>
        </body>
    </html>
    """

@app.get("/api/agents")
async def get_agents():
    """Get connected agents"""
    if hub_servicer:
        agents = []
        for agent_id, info in hub_servicer.connected_agents.items():
            agents.append({
                "id": agent_id,
                "name": agent_id,
                "type": info.get('capabilities', {}).get('type', 'unknown'),
                "status": "active",
                "registered_at": info.get('registered_at'),
                "version": info.get('version'),
                "hostname": info.get('hostname')
            })
        return {"agents": agents}
    return {"agents": []}

@app.get("/api/logs/recent")
async def get_recent_logs():
    """Get recent logs"""
    if storage_manager:
        logs = await storage_manager.get_recent_logs(limit=50)
        return {"logs": logs}
    return {"logs": []}

@app.get("/api/terminals")
async def get_terminals():
    """Get all terminal sessions"""
    if hub_servicer:
        terminals = []
        for session_id, session_info in hub_servicer.terminal_sessions.items():
            terminals.append({
                "session_id": session_id,
                "agent_id": session_info['agent_id'],
                "tmux_session": session_info['tmux_session_name'],
                "status": session_info['status'],
                "created_at": session_info['created_at']
            })
        return {"terminals": terminals}
    return {"terminals": []}

@app.post("/api/terminals/create")
async def create_terminal(agent_id: str, tmux_session: str = None):
    """Create a new terminal session"""
    if hub_servicer and agent_id in hub_servicer.connected_agents:
        # TODO: Send create terminal request to agent via gRPC
        return {"success": True, "message": "Terminal creation requested"}
    return {"success": False, "message": "Agent not found"}

from fastapi import WebSocket

@app.websocket("/ws/terminal/{session_id}")
async def terminal_websocket(websocket: WebSocket, session_id: str):
    """WebSocket endpoint for terminal access"""
    if terminal_ws_manager:
        await terminal_ws_manager.connect(websocket, session_id)

async def run_servers(settings: Settings):
    """Run both HTTP and gRPC servers"""
    global storage_manager, hub_servicer, terminal_ws_manager
    
    # Initialize storage
    storage_manager = StorageManager(settings)
    
    # Create gRPC server
    grpc_server = grpc.aio.server(
        futures.ThreadPoolExecutor(max_workers=10),
        options=[
            ('grpc.max_receive_message_length', 100 * 1024 * 1024),  # 100MB
        ]
    )
    
    # Add servicer
    hub_servicer = AgentHubServicer(storage_manager)
    serve_grpc(grpc_server, hub_servicer)
    
    # Initialize terminal WebSocket manager
    terminal_ws_manager = TerminalWebSocketManager(hub_servicer)
    
    # Start gRPC server
    grpc_port = settings.grpc_port
    grpc_server.add_insecure_port(f'[::]:{grpc_port}')
    await grpc_server.start()
    logger.info(f"gRPC server started on port {grpc_port}")
    
    # Run FastAPI server in background
    config = uvicorn.Config(
        app=app,
        host="0.0.0.0",
        port=settings.http_port,
        log_level="info"
    )
    server = uvicorn.Server(config)
    
    # Run both servers
    await asyncio.gather(
        server.serve(),
        grpc_server.wait_for_termination()
    )

def main():
    """Main entry point"""
    settings = Settings()
    
    logger.info("Starting aany-hub server...")
    logger.info(f"HTTP port: {settings.http_port}")
    logger.info(f"gRPC port: {settings.grpc_port}")
    logger.info(f"Storage type: {settings.storage_type}")
    
    try:
        asyncio.run(run_servers(settings))
    except KeyboardInterrupt:
        logger.info("Server stopped by user")

if __name__ == "__main__":
    main()