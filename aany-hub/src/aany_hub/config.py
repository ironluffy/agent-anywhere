"""Configuration management for aany-hub"""

from typing import Optional
from pydantic_settings import BaseSettings
from pydantic import Field

class Settings(BaseSettings):
    """Application settings"""
    
    # Server settings
    http_port: int = Field(default=8080, description="HTTP server port")
    grpc_port: int = Field(default=50052, description="gRPC server port")
    
    # Storage settings
    storage_type: str = Field(default="local", description="Storage type: local or s3")
    storage_path: str = Field(default="./logs", description="Path where agent logs are stored")
    s3_bucket: Optional[str] = Field(default=None, description="S3 bucket name")
    s3_prefix: str = Field(default="agent-logs/", description="S3 key prefix")
    
    # Logging settings
    log_level: str = Field(default="INFO", description="Logging level")
    log_format: str = Field(default="json", description="Log format: json or text")
    
    class Config:
        env_prefix = "AANY_HUB_"
        env_file = ".env"