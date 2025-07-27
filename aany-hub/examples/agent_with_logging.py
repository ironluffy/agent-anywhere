#!/usr/bin/env python3
"""Example of how a tmux agent would send logs to aany-hub"""

import time
import random
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from aany_hub.agent_connector import init_hub_connection, log_to_hub, close_hub_connection

def simulate_agent_activity(agent_id: str):
    """Simulate an agent doing work and logging"""
    
    # Connect to hub
    print(f"Connecting {agent_id} to hub...")
    init_hub_connection("localhost:50052", agent_id)
    
    try:
        # Log agent startup
        log_to_hub("INFO", f"Agent {agent_id} started", "lifecycle")
        
        # Simulate tmux session creation
        log_to_hub("INFO", "Created tmux session: agent-main", "tmux", 
                   session="agent-main", action="create")
        
        # Simulate various activities
        activities = [
            ("Running health check", "health"),
            ("Executing user command: ls -la", "executor"),
            ("Monitoring system resources", "monitor"),
            ("Processing task queue", "scheduler"),
            ("Updating agent state", "state"),
        ]
        
        for i in range(20):
            activity, component = random.choice(activities)
            level = random.choice(["INFO", "INFO", "INFO", "DEBUG", "WARN"])
            
            if random.random() < 0.1:  # 10% chance of error
                level = "ERROR"
                activity = f"Error in {component}: {random.choice(['timeout', 'connection failed', 'permission denied'])}"
            
            log_to_hub(level, activity, component, 
                       iteration=i, 
                       cpu_usage=random.randint(10, 90),
                       memory_mb=random.randint(100, 500))
            
            print(f"[{level}] {activity}")
            time.sleep(1)
        
        # Log agent shutdown
        log_to_hub("INFO", f"Agent {agent_id} shutting down", "lifecycle")
        
    finally:
        close_hub_connection()
        print(f"Agent {agent_id} disconnected from hub")

if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Simulate agent with hub logging")
    parser.add_argument("--agent-id", default="test-agent-001", help="Agent identifier")
    
    args = parser.parse_args()
    
    simulate_agent_activity(args.agent_id)