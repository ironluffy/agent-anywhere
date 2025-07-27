"""gRPC server implementation for agent communication"""

import logging
from typing import AsyncIterator, Dict, Any, Optional
import uuid
from datetime import datetime
import asyncio
from collections import defaultdict

import grpc

# Import generated protobuf classes
from . import agent_pb2
from . import agent_pb2_grpc

logger = logging.getLogger(__name__)

class AgentHubServicer(agent_pb2_grpc.AgentHubServicer):
    """gRPC service implementation"""
    
    def __init__(self, storage_manager):
        self.storage_manager = storage_manager
        self.connected_agents: Dict[str, Dict[str, Any]] = {}
        self.terminal_sessions: Dict[str, Dict[str, Any]] = {}
        self.terminal_streams: Dict[str, asyncio.Queue] = {}
        self.websocket_queues: Dict[str, list[asyncio.Queue]] = defaultdict(list)
        self.terminal_buffers: Dict[str, list[Dict[str, Any]]] = defaultdict(list)
    
    async def SendLogs(self, request_iterator, context):
        """Handle streaming logs from agents"""
        entries_received = 0
        
        try:
            async for log_entry in request_iterator:
                # Convert to dict
                log_dict = {
                    'agent_id': log_entry.agent_id,
                    'timestamp': log_entry.timestamp,
                    'level': log_entry.level,
                    'message': log_entry.message,
                    'component': log_entry.component,
                    'metadata': dict(log_entry.metadata) if log_entry.metadata else {}
                }
                
                # Store log
                success = await self.storage_manager.store_log(log_dict)
                if success:
                    entries_received += 1
                
                # Log locally for debugging
                logger.debug(f"Received log from {log_entry.agent_id}: {log_entry.message}")
            
            return agent_pb2.LogResponse(success=True, entries_received=entries_received)
            
        except Exception as e:
            logger.error(f"Error in SendLogs: {e}")
            return agent_pb2.LogResponse(success=False, entries_received=entries_received)
    
    async def RegisterAgent(self, request, context):
        """Register a new agent"""
        try:
            session_id = str(uuid.uuid4())
            
            # Store agent info (request IS the AgentInfo)
            self.connected_agents[request.agent_id] = {
                'session_id': session_id,
                'version': request.version,
                'hostname': request.hostname,
                'capabilities': dict(request.capabilities) if request.capabilities else {},
                'registered_at': datetime.utcnow().isoformat(),
                'last_heartbeat': datetime.utcnow().isoformat()
            }
            
            logger.info(f"Agent registered: {request.agent_id} from {request.hostname}")
            
            return agent_pb2.RegistrationResponse(
                success=True,
                message=f"Agent {request.agent_id} registered successfully",
                session_id=session_id
            )
            
        except Exception as e:
            logger.error(f"Error in RegisterAgent: {e}")
            return agent_pb2.RegistrationResponse(
                success=False,
                message=str(e),
                session_id=""
            )
    
    async def Heartbeat(self, request, context):
        """Handle agent heartbeat"""
        try:
            if request.agent_id in self.connected_agents:
                self.connected_agents[request.agent_id]['last_heartbeat'] = datetime.utcnow().isoformat()
            
            return agent_pb2.HeartbeatResponse(
                alive=True,
                server_time=datetime.utcnow().isoformat()
            )
            
        except Exception as e:
            logger.error(f"Error in Heartbeat: {e}")
            return agent_pb2.HeartbeatResponse(
                alive=False,
                server_time=datetime.utcnow().isoformat()
            )
    
    async def CreateTerminalSession(self, request, context):
        """Create a new terminal session"""
        try:
            session_id = str(uuid.uuid4())
            
            # Store terminal session info
            self.terminal_sessions[session_id] = {
                'agent_id': request.agent_id,
                'tmux_session_name': request.tmux_session_name,
                'window': request.window,
                'pane': request.pane,
                'metadata': dict(request.metadata) if request.metadata else {},
                'created_at': datetime.utcnow().isoformat(),
                'status': 'active'
            }
            
            # Create queue for this session
            self.terminal_streams[session_id] = asyncio.Queue()
            
            logger.info(f"Terminal session created: {session_id} for agent {request.agent_id}")
            
            terminal_info = agent_pb2.TerminalInfo(
                session_id=session_id,
                tmux_session_name=request.tmux_session_name,
                rows=24,  # Default terminal size
                cols=80,
                status='active'
            )
            
            return agent_pb2.TerminalSessionResponse(
                success=True,
                session_id=session_id,
                message="Terminal session created",
                terminal_info=terminal_info
            )
            
        except Exception as e:
            logger.error(f"Error in CreateTerminalSession: {e}")
            return agent_pb2.TerminalSessionResponse(
                success=False,
                session_id="",
                message=str(e)
            )
    
    async def AttachTerminalSession(self, request, context):
        """Attach to existing terminal session"""
        try:
            # For now, just verify the session exists
            if request.session_id in self.terminal_sessions:
                session = self.terminal_sessions[request.session_id]
                
                terminal_info = agent_pb2.TerminalInfo(
                    session_id=request.session_id,
                    tmux_session_name=session['tmux_session_name'],
                    rows=24,
                    cols=80,
                    status=session['status']
                )
                
                return agent_pb2.TerminalSessionResponse(
                    success=True,
                    session_id=request.session_id,
                    message="Attached to terminal session",
                    terminal_info=terminal_info
                )
            else:
                return agent_pb2.TerminalSessionResponse(
                    success=False,
                    session_id="",
                    message="Session not found"
                )
                
        except Exception as e:
            logger.error(f"Error in AttachTerminalSession: {e}")
            return agent_pb2.TerminalSessionResponse(
                success=False,
                session_id="",
                message=str(e)
            )
    
    async def TerminalIO(self, request_iterator, context):
        """Handle bidirectional terminal I/O streaming"""
        session_id = None
        output_queue = None
        
        try:
            # Process incoming stream and setup output stream
            async def process_input():
                nonlocal session_id, output_queue
                
                async for terminal_data in request_iterator:
                    if not session_id:
                        session_id = terminal_data.session_id
                        if session_id not in self.terminal_sessions:
                            logger.error(f"Invalid session_id: {session_id}")
                            break
                        
                        # Get or create output queue for this session
                        if session_id not in self.terminal_streams:
                            self.terminal_streams[session_id] = asyncio.Queue()
                        output_queue = self.terminal_streams[session_id]
                        
                        logger.info(f"Terminal stream established for session {session_id}")
                    
                    # Handle different types of terminal data
                    which_data = terminal_data.WhichOneof('data')
                    
                    if which_data == 'output':
                        # Terminal output from agent - broadcast to WebSocket clients
                        logger.info(f"Received output for session {session_id}: {terminal_data.output[:50]}...")
                        logger.info(f"WebSocket queues for session: {len(self.websocket_queues.get(session_id, []))}")
                        
                        output_data = {
                            'type': 'output',
                            'data': terminal_data.output,
                            'timestamp': terminal_data.timestamp
                        }
                        
                        # Buffer the output for later WebSocket connections
                        self.terminal_buffers[session_id].append(output_data)
                        # Keep only last 1000 entries
                        if len(self.terminal_buffers[session_id]) > 1000:
                            self.terminal_buffers[session_id] = self.terminal_buffers[session_id][-1000:]
                        
                        # Broadcast to current connections
                        await self._broadcast_to_websockets(session_id, output_data)
                        
                    elif which_data == 'input':
                        # Input from hub - should not happen in this direction
                        pass
                        
                    elif which_data == 'resize':
                        # Terminal resize event
                        await self._broadcast_to_websockets(session_id, {
                            'type': 'resize',
                            'rows': terminal_data.resize.rows,
                            'cols': terminal_data.resize.cols,
                            'timestamp': terminal_data.timestamp
                        })
            
            # Start processing input
            input_task = asyncio.create_task(process_input())
            
            # Wait for session_id to be set
            while not session_id:
                await asyncio.sleep(0.1)
                if input_task.done():
                    return
            
            # Send output back to agent
            try:
                while not input_task.done():
                    try:
                        # Get data from queue with timeout
                        data = await asyncio.wait_for(output_queue.get(), timeout=1.0)
                        yield data
                    except asyncio.TimeoutError:
                        continue
                    except Exception as e:
                        logger.error(f"Error sending output: {e}")
                        break
                        
            except Exception as e:
                logger.error(f"Error in output stream: {e}")
            finally:
                input_task.cancel()
                    
        except Exception as e:
            logger.error(f"Error in TerminalIO: {e}")
            
    async def ListTerminalSessions(self, request, context):
        """List terminal sessions for an agent"""
        try:
            sessions = []
            
            for session_id, session_info in self.terminal_sessions.items():
                if session_info['agent_id'] == request.agent_id:
                    terminal_info = agent_pb2.TerminalInfo(
                        session_id=session_id,
                        tmux_session_name=session_info['tmux_session_name'],
                        rows=24,
                        cols=80,
                        status=session_info['status']
                    )
                    sessions.append(terminal_info)
            
            return agent_pb2.ListTerminalSessionsResponse(sessions=sessions)
            
        except Exception as e:
            logger.error(f"Error in ListTerminalSessions: {e}")
            return agent_pb2.ListTerminalSessionsResponse(sessions=[])
    
    async def CloseTerminalSession(self, request, context):
        """Close a terminal session"""
        try:
            if request.session_id in self.terminal_sessions:
                # Mark session as closed
                self.terminal_sessions[request.session_id]['status'] = 'closed'
                
                # Clean up queues and buffers
                if request.session_id in self.terminal_streams:
                    del self.terminal_streams[request.session_id]
                if request.session_id in self.websocket_queues:
                    del self.websocket_queues[request.session_id]
                if request.session_id in self.terminal_buffers:
                    del self.terminal_buffers[request.session_id]
                
                logger.info(f"Terminal session closed: {request.session_id}")
                
                return agent_pb2.CloseTerminalResponse(
                    success=True,
                    message="Terminal session closed"
                )
            else:
                return agent_pb2.CloseTerminalResponse(
                    success=False,
                    message="Session not found"
                )
                
        except Exception as e:
            logger.error(f"Error in CloseTerminalSession: {e}")
            return agent_pb2.CloseTerminalResponse(
                success=False,
                message=str(e)
            )
    
    async def _broadcast_to_websockets(self, session_id: str, data: Dict[str, Any]):
        """Broadcast terminal data to all connected WebSocket clients"""
        if session_id in self.websocket_queues:
            for queue in self.websocket_queues[session_id]:
                try:
                    await queue.put(data)
                except Exception as e:
                    logger.error(f"Error broadcasting to WebSocket: {e}")
    
    def register_websocket_queue(self, session_id: str, queue: asyncio.Queue):
        """Register a WebSocket queue for a terminal session"""
        self.websocket_queues[session_id].append(queue)
        
    def get_terminal_buffer(self, session_id: str) -> list[Dict[str, Any]]:
        """Get buffered output for a terminal session"""
        return self.terminal_buffers.get(session_id, [])
        
    def unregister_websocket_queue(self, session_id: str, queue: asyncio.Queue):
        """Unregister a WebSocket queue"""
        if session_id in self.websocket_queues:
            try:
                self.websocket_queues[session_id].remove(queue)
            except ValueError:
                pass
    
    async def send_terminal_input(self, session_id: str, input_data: str):
        """Send input from WebSocket to terminal session"""
        if session_id in self.terminal_streams:
            terminal_data = agent_pb2.TerminalData(
                session_id=session_id,
                input=input_data,
                timestamp=int(datetime.utcnow().timestamp() * 1000)
            )
            await self.terminal_streams[session_id].put(terminal_data)

def serve_grpc(server, servicer):
    """Add servicer to gRPC server"""
    agent_pb2_grpc.add_AgentHubServicer_to_server(servicer, server)
    logger.info("gRPC servicer registered")