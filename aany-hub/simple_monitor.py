#!/usr/bin/env python3
"""
Simple tmux monitor that sends output to hub
"""
import grpc
import sys
import os
import time
import subprocess
import asyncio

# Add the aany-hub path to import proto files
script_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(script_dir, 'src'))

from aany_hub import agent_pb2, agent_pb2_grpc

async def monitor_and_stream(agent_id, hub_url='localhost:50052'):
    """Monitor tmux and stream to hub"""
    session_name = f"agent-{agent_id}"
    
    # Connect to hub
    async with grpc.aio.insecure_channel(hub_url) as channel:
        stub = agent_pb2_grpc.AgentHubStub(channel)
        
        # Create terminal session
        request = agent_pb2.CreateTerminalRequest(
            agent_id=agent_id,
            tmux_session_name=session_name,
            window=0,
            pane=0,
            metadata={}
        )
        
        response = await stub.CreateTerminalSession(request)
        if not response.success:
            print(f"Failed to create terminal session: {response.message}")
            return
            
        session_id = response.session_id
        print(f"Terminal session created: {session_id}")
        
        # Start streaming
        async def stream_generator():
            # Send initial message
            yield agent_pb2.TerminalData(
                session_id=session_id,
                timestamp=int(time.time() * 1000)
            )
            
            last_content = ""
            while True:
                try:
                    # Capture pane
                    result = subprocess.run(
                        ['tmux', 'capture-pane', '-t', session_name, '-p'],
                        capture_output=True,
                        text=True
                    )
                    
                    if result.returncode == 0:
                        content = result.stdout
                        if content != last_content:
                            # Send full content for simplicity
                            yield agent_pb2.TerminalData(
                                session_id=session_id,
                                output=content,
                                timestamp=int(time.time() * 1000)
                            )
                            last_content = content
                            print(f"Sent {len(content)} bytes")
                    
                    await asyncio.sleep(0.5)
                    
                except Exception as e:
                    print(f"Error in loop: {e}")
                    break
        
        # Create the stream
        try:
            call = stub.TerminalIO(stream_generator())
            
            # Read responses (if any)
            async for response in call:
                print(f"Got response: {response}")
                
        except Exception as e:
            print(f"Stream error: {e}")

async def main():
    if len(sys.argv) < 2:
        print("Usage: simple_monitor.py <agent_id>")
        sys.exit(1)
        
    agent_id = sys.argv[1]
    await monitor_and_stream(agent_id)

if __name__ == "__main__":
    asyncio.run(main())