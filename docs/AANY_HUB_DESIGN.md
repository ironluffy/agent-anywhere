# aany-hub Design

## Overview

`aany-hub` is an **independent** centralized platform for Agent Anywhere. It provides browser-based discovery and management of distributed agents. The hub is optional - agents can run completely standalone without it.

## Design Principles

1. **Independent Platform**
   - Standalone service (agents work without hub)
   - Self-contained in `aany-hub/` directory
   - No dependencies injected into core agent code
   - Optional integration for agents

2. **Repository Structure**
   - Component name: `aany-hub` (lowercase, hyphenated)
   - Python-based for rapid development
   - All hub code stays within `aany-hub/` folder
   - Agents remain fully functional without hub

2. **Scalable & Fast**
   - FastAPI for high-performance async API
   - Redis for caching and real-time data
   - S3 for scalable storage
   - gRPC for efficient agent communication

3. **Browser-Based Interface**
   - Accessible from any device on network
   - Real-time agent status via WebSocket
   - Mobile-responsive design
   - No installation required on client devices

## Integration with Existing Components

```
aany (CLI - Rust)
 ├── aany-tmux (tmux integration - Rust)
 ├── aany-pool (agent management - Rust)
 └── aany-hub (web dashboard - Python) <-- NEW
```

### Agent Integration (Optional)

Agents can optionally connect to hub for enhanced features:

```bash
# Agent runs standalone (default)
aany pool start my-agent

# Agent connects to hub (optional, future)
aany pool start my-agent --hub-url https://hub.agent-anywhere.com
aany pool start my-agent --hub-login  # OAuth login flow
```

### Hub Deployment

```bash
# Hub runs as separate service
cd aany-hub
python -m aany_hub.main

# Or via Docker
docker run -p 8080:8080 agent-anywhere/hub
```

## Architecture

### Backend Stack (Python)

```
┌─────────────────────────────────────────────────────────┐
│                    FastAPI Server                        │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │   REST API  │  │  WebSocket   │  │   gRPC Server │ │
│  └─────────────┘  └──────────────┘  └───────────────┘ │
└─────────────────────────┬───────────────────────────────┘
                          │
┌─────────────────────────┼───────────────────────────────┐
│                         │                                │
│  ┌─────────────┐  ┌────┴─────┐  ┌──────────────────┐  │
│  │    Redis    │  │   S3/    │  │  Agent           │  │
│  │  (Cache)    │  │  MinIO   │  │  Communication   │  │
│  └─────────────┘  └──────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Tech Stack
- **Web Framework**: FastAPI (async, high performance)
- **gRPC**: grpcio for agent communication
- **WebSocket**: FastAPI's built-in WebSocket support
- **Cache**: Redis (agent status, session data)
- **Storage**: S3/MinIO (logs, artifacts, templates)
- **Task Queue**: Celery + Redis (background jobs)
- **Authentication**: OAuth 2.0 (GitHub, Google, custom providers)

### Frontend
- React or Vue.js for dynamic UI
- WebSocket client for real-time updates
- Tailwind CSS for styling
- Chart.js for metrics visualization

### Communication Protocols
- **REST API**: Client ↔ Hub (CRUD operations)
- **WebSocket**: Client ↔ Hub (real-time updates)
- **gRPC**: Hub ↔ Agents (efficient binary protocol)
- **Redis Pub/Sub**: Internal event streaming
- **MCP**: Agents ↔ External Tools/Services (standardized tool access)

## File Structure

```
aany-hub/
├── pyproject.toml
├── requirements.txt
├── src/
│   └── aany_hub/
│       ├── __init__.py
│       ├── main.py           # FastAPI app entry
│       ├── api/
│       │   ├── __init__.py
│       │   ├── agents.py     # Agent endpoints
│       │   ├── templates.py  # Template marketplace
│       │   └── websocket.py  # WebSocket handlers
│       ├── grpc/
│       │   ├── __init__.py
│       │   ├── server.py     # gRPC server
│       │   └── agent.proto   # Protocol definitions
│       ├── mcp/
│       │   ├── __init__.py
│       │   ├── server.py     # MCP server implementation
│       │   ├── services/
│       │   │   ├── autopilot.py      # Autopilot service
│       │   │   ├── troubleshoot.py   # KB service
│       │   │   ├── rag.py            # Personal RAG
│       │   │   ├── vector_search.py  # Document search
│       │   │   ├── marketplace.py    # Template service
│       │   │   └── llm_bridge.py     # LLM connections
│       │   └── registry.py   # Service registry
│       ├── storage/
│       │   ├── __init__.py
│       │   ├── s3.py         # S3 operations
│       │   └── redis.py      # Redis operations
│       ├── models/
│       │   ├── __init__.py
│       │   └── agent.py      # Pydantic models
│       └── utils/
│           ├── __init__.py
│           ├── auth.py       # OAuth implementation
│           └── jwt.py        # JWT token handling
├── static/
│   └── (frontend build output)
├── frontend/
│   ├── package.json
│   ├── src/
│   └── public/
└── tests/
    ├── test_api.py
    └── test_grpc.py
```

## Data Flow

1. **Agent Registration**
   ```
   Agent → gRPC → Hub → Redis (cache) → S3 (persist)
   ```

2. **Real-time Updates**
   ```
   Agent → gRPC → Hub → Redis Pub/Sub → WebSocket → Client
   ```

3. **Log Streaming**
   ```
   Agent → gRPC Stream → Hub → S3 (store) → WebSocket → Client
   ```

4. **MCP Tool Access**
   ```
   Agent → MCP Client → MCP Server (in Hub) → External Tools
   ```

## MCP Integration

### Hub as Intelligent MCP Server
The aany-hub acts as an MCP server, providing intelligent services to agents:

```
┌─────────────────────────────────────────────────────────┐
│              aany-hub (Intelligent MCP Server)           │
│  ┌─────────────────────────────────────────────────┐   │
│  │            MCP Intelligence Services              │   │
│  │  ┌──────────────┐  ┌─────────────────────────┐  │   │
│  │  │  Autopilot   │  │  Troubleshooting KB     │  │   │
│  │  │  Service     │  │  (Learning from logs)   │  │   │
│  │  └──────────────┘  └─────────────────────────┘  │   │
│  │  ┌──────────────┐  ┌─────────────────────────┐  │   │
│  │  │ Personal RAG │  │  Work Document Search   │  │   │
│  │  │ (Your data)  │  │  (Vector embeddings)    │  │   │
│  │  └──────────────┘  └─────────────────────────┘  │   │
│  │  ┌──────────────┐  ┌─────────────────────────┐  │   │
│  │  │  Marketplace │  │  Third-party LLM        │  │   │
│  │  │  Templates   │  │  Bridge (via MCP)       │  │   │
│  │  └──────────────┘  └─────────────────────────┘  │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────┬───────────────────────────────┘
                          │ MCP Protocol
┌─────────────────────────┴───────────────────────────────┐
│                    Agents (MCP Clients)                  │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐       │
│  │   Agent    │  │   Agent    │  │   Agent    │       │
│  │ (Claude)   │  │ (GPT)      │  │ (Gemini)   │       │
│  └────────────┘  └────────────┘  └────────────┘       │
└─────────────────────────────────────────────────────────┘
```

### MCP Intelligence Services

1. **Autopilot Service**
   - Learns from your workflow patterns
   - Suggests next actions based on context
   - Executes repetitive tasks automatically
   - Adapts to your coding style over time

2. **Troubleshooting Knowledge Base**
   - Automatically indexes errors and solutions from agent logs
   - Provides instant fixes for known issues
   - Learns from community troubleshooting patterns
   - Suggests preventive measures

3. **Personalized RAG (Retrieval-Augmented Generation)**
   - Indexes your private documents and code
   - Provides context-aware answers from your data
   - Connects to third-party LLM services via MCP
   - Maintains privacy while enhancing AI capabilities

4. **Work Document Vector Search**
   - Semantic search across all your documents
   - Code similarity search
   - Finds relevant examples from your past work
   - Links related concepts across projects

5. **Marketplace Integration**
   - Browse and fetch agent role templates
   - Preview templates before installation
   - Rate and review templates
   - Share your custom templates

6. **Third-party LLM Bridge**
   - Connect any LLM service through MCP protocol
   - Unified interface for different AI providers
   - Load balancing across multiple LLMs
   - Cost optimization by routing to appropriate models

## Implementation Plan

### Phase 1: Core API
- FastAPI server setup
- Basic REST endpoints
- Redis integration
- S3 configuration

### Phase 2: Agent Communication
- gRPC server implementation
- Agent registration/discovery
- Status monitoring
- WebSocket real-time updates

### Phase 3: Advanced Features
- Log streaming and storage
- Template marketplace
- Metrics and monitoring
- OAuth implementation (GitHub, Google)
- Agent authentication tokens
- Platform login for agents (future)

## Development Setup

```bash
# Create virtual environment
python -m venv venv
source venv/bin/activate

# Install dependencies
pip install fastapi uvicorn redis boto3 grpcio

# Start Redis
redis-server

# Start MinIO (local S3)
docker run -p 9000:9000 minio/minio server /data

# Run development server
uvicorn aany_hub.main:app --reload
```

## Key Design Decisions

1. **Python instead of Rust** - Faster development, rich ecosystem
2. **FastAPI** - Modern, fast, async Python framework
3. **Redis** - Fast caching and pub/sub for real-time
4. **S3** - Scalable storage for logs and artifacts
5. **gRPC** - Efficient binary protocol for agent communication
6. **Separate frontend** - Modern SPA for better UX
7. **OAuth 2.0** - Secure user authentication and agent authorization
8. **Independent deployment** - Hub runs separately from agents