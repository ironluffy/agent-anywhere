#!/usr/bin/env python3
"""Test gRPC client that sends real logs to hub"""

import grpc
import asyncio
import time
from datetime import datetime

# Import generated protobuf classes
import sys
sys.path.append('src')
from aany_hub import agent_pb2, agent_pb2_grpc

async def send_logs_to_hub():
    """Send test logs via gRPC"""
    # Connect to hub
    channel = grpc.aio.insecure_channel('localhost:50052')
    stub = agent_pb2_grpc.AgentHubStub(channel)
    
    # Register agent
    agent_info = agent_pb2.AgentInfo(
        agent_id="keyboard-agent-001",
        version="0.1.0",
        hostname="test-host",
        capabilities={"typing": "enabled", "autocorrect": "enabled"}
    )
    
    try:
        print("📡 Registering agent with hub...")
        response = await stub.RegisterAgent(agent_info)
        if response.success:
            print(f"✅ Registered: {response.message}")
            print(f"   Session ID: {response.session_id}")
        else:
            print(f"❌ Registration failed: {response.message}")
            return
    except grpc.RpcError as e:
        print(f"❌ gRPC error: {e}")
        return
    
    # Send logs
    print("\n📝 Sending logs...")
    
    async def log_generator():
        """Generate log entries"""
        logs = [
            ("INFO", "Agent started", "lifecycle", {}),
            ("INFO", "Keyboard simulation beginning", "keyboard", {"session": "test-1"}),
            ("DEBUG", "Typing: Hello from real gRPC client!", "keyboard", {"speed": "50"}),
            ("INFO", "Command executed: echo 'test'", "command", {"exit_code": "0"}),
            ("WARN", "Typo detected and corrected: ehco -> echo", "autocorrect", {"corrections": "1"}),
            ("INFO", "Command executed: date", "command", {"exit_code": "0"}),
            ("ERROR", "Failed to execute: invalid-command", "command", {"exit_code": "127"}),
            ("INFO", "Agent stopping", "lifecycle", {}),
        ]
        
        for level, message, component, metadata in logs:
            log_entry = agent_pb2.LogEntry(
                agent_id="keyboard-agent-001",
                timestamp=datetime.utcnow().isoformat(),
                level=level,
                message=message,
                component=component,
                metadata=metadata
            )
            
            print(f"   [{level}] {component}: {message}")
            yield log_entry
            await asyncio.sleep(0.5)
    
    try:
        response = await stub.SendLogs(log_generator())
        print(f"\n✅ Logs sent successfully! Entries received: {response.entries_received}")
    except grpc.RpcError as e:
        print(f"❌ Failed to send logs: {e}")
    
    # Send heartbeat
    print("\n💓 Sending heartbeat...")
    heartbeat_request = agent_pb2.HeartbeatRequest(
        agent_id="keyboard-agent-001",
        timestamp=datetime.utcnow().isoformat()
    )
    
    try:
        response = await stub.Heartbeat(heartbeat_request)
        if response.alive:
            print(f"✅ Heartbeat acknowledged. Server time: {response.server_time}")
        else:
            print("❌ Heartbeat failed")
    except grpc.RpcError as e:
        print(f"❌ Heartbeat error: {e}")
    
    await channel.close()

def cleanup_message():
    """Show cleanup message if running in tmux"""
    import subprocess
    import os
    
    if os.environ.get('TMUX'):
        try:
            result = subprocess.run(['tmux', 'display-message', '-p', '#{session_name}:#{window_index}'], 
                                  capture_output=True, text=True)
            if result.returncode == 0:
                print(f"\n🧹 Test completed. To clean up this window, run: tmux kill-window")
        except:
            pass

if __name__ == "__main__":
    print("🚀 Agent Anywhere gRPC Client Test")
    print("==================================")
    try:
        asyncio.run(send_logs_to_hub())
    finally:
        cleanup_message()