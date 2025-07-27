#!/usr/bin/env python3
"""
Enhanced monitor for tmux pane that sends full terminal updates
"""
import grpc
import sys
import os
import time
import subprocess
import asyncio
import signal

# Add the aany-hub path to import proto files
script_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(script_dir, 'src'))

from aany_hub import agent_pb2, agent_pb2_grpc

class TmuxMonitor:
    def __init__(self, agent_id, hub_url='localhost:50052'):
        self.agent_id = agent_id
        self.hub_url = hub_url
        self.session_name = f"agent-{agent_id}"
        self.terminal_session_id = None
        self.running = True
        self.last_content_hash = None
        
    def capture_pane(self):
        """Capture current tmux pane content"""
        try:
            result = subprocess.run(
                ['tmux', 'capture-pane', '-t', self.session_name, '-p', '-e'],  # -e preserves colors
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                return result.stdout
            return None
        except Exception as e:
            print(f"Error capturing pane: {e}")
            return None
            
    async def stream_output(self, stub):
        """Stream terminal output to hub"""
        print(f"Starting output stream for session {self.terminal_session_id}")
        
        # Generator for outbound stream
        async def request_generator():
            # Send initial message
            yield agent_pb2.TerminalData(
                session_id=self.terminal_session_id,
                timestamp=int(time.time() * 1000)
            )
            
            while self.running:
                # Capture pane content
                content = self.capture_pane()
                if content:
                    # Calculate hash to detect changes
                    content_hash = hash(content)
                    
                    if content_hash != self.last_content_hash:
                        # Content changed - send full update
                        # Clear screen and send new content
                        output = f"\033[2J\033[H{content}"  # Clear screen + home cursor + content
                        
                        yield agent_pb2.TerminalData(
                            session_id=self.terminal_session_id,
                            output=output,
                            timestamp=int(time.time() * 1000)
                        )
                        
                        self.last_content_hash = content_hash
                        print(f"Sent update: {len(content)} bytes")
                
                await asyncio.sleep(0.2)  # Check more frequently
        
        try:
            # Create bidirectional stream
            stream = stub.TerminalIO(request_generator())
            
            # Process incoming messages
            async for response in stream:
                if response.HasField('input'):
                    # Send input to tmux
                    subprocess.run([
                        'tmux', 'send-keys', '-t', self.session_name, 
                        response.input
                    ])
                    print(f"Received input: {response.input}")
                    
        except Exception as e:
            print(f"Stream error: {e}")
            
    async def run(self):
        """Main monitoring loop"""
        # Connect to hub
        channel = grpc.aio.insecure_channel(self.hub_url)
        stub = agent_pb2_grpc.AgentHubStub(channel)
        
        try:
            # Create terminal session
            request = agent_pb2.CreateTerminalRequest(
                agent_id=self.agent_id,
                tmux_session_name=self.session_name,
                window=0,
                pane=0,
                metadata={}
            )
            
            response = await stub.CreateTerminalSession(request)
            if not response.success:
                print(f"Failed to create terminal session: {response.message}")
                return
                
            self.terminal_session_id = response.session_id
            print(f"Terminal session created: {self.terminal_session_id}")
            
            # Start streaming
            await self.stream_output(stub)
            
        except Exception as e:
            print(f"Error: {e}")
        finally:
            await channel.close()
            
    def stop(self):
        """Stop monitoring"""
        self.running = False

async def main():
    if len(sys.argv) < 2:
        print("Usage: monitor_tmux_pane_v2.py <agent_id> [hub_url]")
        sys.exit(1)
        
    agent_id = sys.argv[1]
    hub_url = sys.argv[2] if len(sys.argv) > 2 else 'localhost:50052'
    
    monitor = TmuxMonitor(agent_id, hub_url)
    
    # Handle shutdown
    def signal_handler(sig, frame):
        print("\nShutting down...")
        monitor.stop()
        
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    await monitor.run()

if __name__ == "__main__":
    asyncio.run(main())