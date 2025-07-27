"""Simplified gRPC server that actually works"""

import asyncio
import grpc
import logging
from concurrent import futures
from datetime import datetime
import os
import json
import uuid

# Import protobuf with absolute imports
from . import agent_pb2
from . import agent_pb2_grpc

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Simple file-based storage - use absolute path from environment
import pathlib
DEFAULT_LOG_DIR = pathlib.Path(__file__).parent.parent.parent / "logs"
LOG_DIR = os.environ.get('AANY_HUB_STORAGE_PATH', str(DEFAULT_LOG_DIR))

class SimpleAgentHub(agent_pb2_grpc.AgentHubServicer):
    def __init__(self):
        os.makedirs(LOG_DIR, exist_ok=True)
        self.agents = {}
        
    async def RegisterAgent(self, request, context):
        """Register agent"""
        agent_id = request.agent_id
        self.agents[agent_id] = {
            'version': request.version,
            'hostname': request.hostname,
            'registered_at': datetime.utcnow().isoformat()
        }
        logger.info(f"Agent registered: {agent_id}")
        
        return agent_pb2.RegistrationResponse(
            success=True,
            message=f"Agent {agent_id} registered",
            session_id=str(uuid.uuid4())
        )
    
    async def SendLogs(self, request_iterator, context):
        """Receive and store logs"""
        count = 0
        async for log_entry in request_iterator:
            # Store log
            agent_dir = os.path.join(LOG_DIR, log_entry.agent_id)
            os.makedirs(agent_dir, exist_ok=True)
            
            date_str = datetime.utcnow().strftime('%Y-%m-%d')
            log_file = os.path.join(agent_dir, f"{date_str}.jsonl")
            
            log_data = {
                'agent_id': log_entry.agent_id,
                'timestamp': log_entry.timestamp,
                'level': log_entry.level,
                'message': log_entry.message,
                'component': log_entry.component,
                'metadata': dict(log_entry.metadata),
                'server_timestamp': datetime.utcnow().isoformat()
            }
            
            with open(log_file, 'a') as f:
                f.write(json.dumps(log_data) + '\n')
            
            count += 1
            logger.info(f"Stored log from {log_entry.agent_id}: [{log_entry.level}] {log_entry.message}")
        
        return agent_pb2.LogResponse(
            success=True,
            entries_received=count
        )
    
    async def Heartbeat(self, request, context):
        """Handle heartbeat"""
        return agent_pb2.HeartbeatResponse(
            alive=True,
            server_time=datetime.utcnow().isoformat()
        )

async def serve():
    """Start gRPC server"""
    server = grpc.aio.server()
    servicer = SimpleAgentHub()
    agent_pb2_grpc.add_AgentHubServicer_to_server(servicer, server)
    
    server.add_insecure_port('[::]:50052')
    await server.start()
    logger.info("gRPC server started on port 50052")
    
    try:
        await server.wait_for_termination()
    except KeyboardInterrupt:
        await server.stop(0)

if __name__ == '__main__':
    print("Starting simplified Agent Hub server...")
    asyncio.run(serve())