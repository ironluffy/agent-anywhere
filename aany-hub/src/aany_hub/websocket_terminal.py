"""WebSocket terminal handler for aany-hub"""

import json
import logging
import asyncio
from typing import Dict, Any
from datetime import datetime

from fastapi import WebSocket, WebSocketDisconnect
from fastapi.websockets import WebSocketState

logger = logging.getLogger(__name__)

class TerminalWebSocketManager:
    """Manages WebSocket connections for terminal sessions"""
    
    def __init__(self, hub_servicer):
        self.hub_servicer = hub_servicer
        self.active_connections: Dict[str, list[WebSocket]] = {}
        
    async def connect(self, websocket: WebSocket, session_id: str):
        """Accept WebSocket connection for terminal session"""
        await websocket.accept()
        
        # Add to active connections
        if session_id not in self.active_connections:
            self.active_connections[session_id] = []
        self.active_connections[session_id].append(websocket)
        
        # Create queue for this WebSocket
        queue = asyncio.Queue()
        self.hub_servicer.register_websocket_queue(session_id, queue)
        
        try:
            # Send initial terminal info
            if session_id in self.hub_servicer.terminal_sessions:
                session_info = self.hub_servicer.terminal_sessions[session_id]
                await websocket.send_json({
                    'type': 'connected',
                    'session_id': session_id,
                    'tmux_session': session_info['tmux_session_name'],
                    'agent_id': session_info['agent_id']
                })
                
                # Send buffered output
                buffered_output = self.hub_servicer.get_terminal_buffer(session_id)
                for output in buffered_output:
                    await websocket.send_json(output)
            
            # Start tasks for bidirectional communication
            receive_task = asyncio.create_task(self._receive_from_websocket(websocket, session_id))
            send_task = asyncio.create_task(self._send_to_websocket(websocket, queue))
            
            # Wait for either task to complete
            done, pending = await asyncio.wait(
                [receive_task, send_task],
                return_when=asyncio.FIRST_COMPLETED
            )
            
            # Cancel pending tasks
            for task in pending:
                task.cancel()
                
        except WebSocketDisconnect:
            logger.info(f"WebSocket disconnected for session {session_id}")
        except Exception as e:
            logger.error(f"Error in WebSocket connection: {e}")
        finally:
            # Clean up
            self.hub_servicer.unregister_websocket_queue(session_id, queue)
            if session_id in self.active_connections:
                try:
                    self.active_connections[session_id].remove(websocket)
                except ValueError:
                    pass
                    
    async def _receive_from_websocket(self, websocket: WebSocket, session_id: str):
        """Receive messages from WebSocket and forward to terminal"""
        try:
            while True:
                # Receive message from WebSocket
                data = await websocket.receive_json()
                
                message_type = data.get('type')
                
                if message_type == 'input':
                    # Forward input to terminal via gRPC
                    input_data = data.get('data', '')
                    await self.hub_servicer.send_terminal_input(session_id, input_data)
                    
                elif message_type == 'resize':
                    # Handle terminal resize
                    rows = data.get('rows', 24)
                    cols = data.get('cols', 80)
                    # TODO: Forward resize to agent
                    logger.info(f"Terminal resize: {rows}x{cols}")
                    
                elif message_type == 'ping':
                    # Respond to ping
                    await websocket.send_json({'type': 'pong'})
                    
        except WebSocketDisconnect:
            raise
        except Exception as e:
            logger.error(f"Error receiving from WebSocket: {e}")
            raise
            
    async def _send_to_websocket(self, websocket: WebSocket, queue: asyncio.Queue):
        """Send messages from queue to WebSocket"""
        try:
            while True:
                # Get message from queue
                message = await queue.get()
                
                # Send to WebSocket
                if websocket.client_state == WebSocketState.CONNECTED:
                    await websocket.send_json(message)
                else:
                    break
                    
        except Exception as e:
            logger.error(f"Error sending to WebSocket: {e}")
            raise
            
    async def broadcast_to_session(self, session_id: str, message: Dict[str, Any]):
        """Broadcast message to all WebSockets connected to a session"""
        if session_id in self.active_connections:
            disconnected = []
            
            for websocket in self.active_connections[session_id]:
                try:
                    if websocket.client_state == WebSocketState.CONNECTED:
                        await websocket.send_json(message)
                    else:
                        disconnected.append(websocket)
                except Exception as e:
                    logger.error(f"Error broadcasting to WebSocket: {e}")
                    disconnected.append(websocket)
                    
            # Remove disconnected WebSockets
            for ws in disconnected:
                try:
                    self.active_connections[session_id].remove(ws)
                except ValueError:
                    pass