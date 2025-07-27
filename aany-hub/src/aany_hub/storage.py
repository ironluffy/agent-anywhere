"""Storage management for agent logs"""

import json
import os
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Any

import aiofiles
import boto3
from botocore.exceptions import ClientError

from .config import Settings

class StorageManager:
    """Manages log storage (local or S3)"""
    
    def __init__(self, settings: Settings):
        self.settings = settings
        self.storage_type = settings.storage_type
        
        if self.storage_type == "local":
            # Create local directory
            self.local_path = Path(settings.storage_path)
            self.local_path.mkdir(parents=True, exist_ok=True)
        elif self.storage_type == "s3":
            # Initialize S3 client
            self.s3_client = boto3.client('s3')
            self.bucket = settings.s3_bucket
            self.prefix = settings.s3_prefix
    
    async def store_log(self, log_entry: Dict[str, Any]) -> bool:
        """Store a single log entry"""
        try:
            # Add server timestamp
            log_entry['server_timestamp'] = datetime.utcnow().isoformat()
            
            if self.storage_type == "local":
                return await self._store_local(log_entry)
            elif self.storage_type == "s3":
                return await self._store_s3(log_entry)
            
            return False
        except Exception as e:
            print(f"Error storing log: {e}")
            return False
    
    async def _store_local(self, log_entry: Dict[str, Any]) -> bool:
        """Store log locally"""
        # Create agent-specific directory
        agent_id = log_entry.get('agent_id', 'unknown')
        agent_dir = self.local_path / agent_id
        agent_dir.mkdir(exist_ok=True)
        
        # Create daily log file
        date_str = datetime.utcnow().strftime('%Y-%m-%d')
        log_file = agent_dir / f"{date_str}.jsonl"
        
        # Append log entry
        async with aiofiles.open(log_file, 'a') as f:
            await f.write(json.dumps(log_entry) + '\n')
        
        return True
    
    async def _store_s3(self, log_entry: Dict[str, Any]) -> bool:
        """Store log in S3"""
        agent_id = log_entry.get('agent_id', 'unknown')
        timestamp = datetime.utcnow()
        
        # Create S3 key
        key = f"{self.prefix}{agent_id}/{timestamp.strftime('%Y/%m/%d')}/{timestamp.strftime('%H%M%S')}.json"
        
        # Upload to S3
        try:
            self.s3_client.put_object(
                Bucket=self.bucket,
                Key=key,
                Body=json.dumps(log_entry),
                ContentType='application/json'
            )
            return True
        except ClientError as e:
            print(f"S3 upload error: {e}")
            return False
    
    async def get_recent_logs(self, agent_id: str = None, limit: int = 100) -> List[Dict[str, Any]]:
        """Get recent logs"""
        logs = []
        
        if self.storage_type == "local":
            # Read from local files
            if agent_id:
                agent_dirs = [self.local_path / agent_id]
            else:
                agent_dirs = [d for d in self.local_path.iterdir() if d.is_dir()]
            
            for agent_dir in agent_dirs:
                # Get today's log file
                date_str = datetime.utcnow().strftime('%Y-%m-%d')
                log_file = agent_dir / f"{date_str}.jsonl"
                
                if log_file.exists():
                    async with aiofiles.open(log_file, 'r') as f:
                        lines = await f.readlines()
                        for line in lines[-limit:]:  # Get last N lines
                            try:
                                logs.append(json.loads(line.strip()))
                            except json.JSONDecodeError:
                                pass
        
        # Sort by timestamp and limit
        logs.sort(key=lambda x: x.get('timestamp', ''), reverse=True)
        return logs[:limit]