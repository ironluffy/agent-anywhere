# .aany Configuration & Features Design

## Overview
The `~/.aany` directory serves as the central configuration and data hub for the Agent Anywhere system.

## Directory Structure
```
~/.aany/
├── config.yaml           # Global configuration
├── pool/                 # Agent pool (already implemented)
│   ├── .pool.yaml       
│   └── agents/          
├── templates/            # Agent templates
│   ├── research.yaml    
│   ├── code.yaml        
│   └── custom/          
├── hooks/                # Lifecycle scripts
│   ├── pre-start/       
│   ├── post-stop/       
│   └── health-check/    
├── logs/                 # Centralized logs
│   ├── aany.log         
│   ├── agents/          
│   └── archive/         
├── backups/              # Agent backups
│   └── snapshots/       
└── cache/                # Temporary data
```

## 1. Global Configuration (`config.yaml`)

```yaml
# ~/.aany/config.yaml
api:
  claude:
    api_key: "${CLAUDE_API_KEY}"  # Can reference env vars
    default_model: "claude-3-opus-20240229"
    max_tokens: 4096
    temperature: 0.7

defaults:
  agent:
    type: "general"
    memory_enabled: true
    auto_save_interval: 300  # seconds
    max_workspace_size: "1GB"
  
  tmux:
    default_shell: "/bin/bash"
    scrollback_lines: 10000
    mouse_support: true

logging:
  level: "info"  # debug, info, warn, error
  max_file_size: "10MB"
  retention_days: 30
  
resources:
  max_agents: 10
  cpu_limit_per_agent: "2"
  memory_limit_per_agent: "4GB"

features:
  auto_backup: true
  backup_interval: 3600  # seconds
  health_check_interval: 60
  auto_cleanup_crashed: true
```

## 2. Agent Templates (`templates/`)

### Research Agent Template
```yaml
# ~/.aany/templates/research.yaml
agent:
  type: "research"
  description: "Autonomous research assistant"
  
claude:
  model: "claude-3-opus-20240229"
  system_prompt_file: "research_prompt.md"
  tools:
    - "web_search"
    - "file_read"
    - "note_taking"
  
init_commands:
  - "mkdir -p research/{sources,notes,reports}"
  - "echo '# Research Log' > research/README.md"
  
environment:
  RESEARCH_MODE: "academic"
  CITATION_STYLE: "APA"
```

### Code Agent Template
```yaml
# ~/.aany/templates/code.yaml
agent:
  type: "code"
  description: "Software development assistant"
  
claude:
  model: "claude-3-opus-20240229"
  system_prompt_file: "code_prompt.md"
  tools:
    - "code_execution"
    - "git"
    - "terminal"
    
init_commands:
  - "git init"
  - "echo '# Project' > README.md"
  
environment:
  EDITOR: "vim"
  NODE_ENV: "development"
```

## 3. Lifecycle Hooks (`hooks/`)

### Pre-start Hook Example
```bash
#!/bin/bash
# ~/.aany/hooks/pre-start/setup_environment.sh

AGENT_NAME=$1
AGENT_DIR=$2

echo "Setting up environment for $AGENT_NAME..."

# Install dependencies if package.json exists
if [ -f "$AGENT_DIR/workspace/package.json" ]; then
    cd "$AGENT_DIR/workspace"
    npm install
fi

# Set up Python virtual environment
if [ -f "$AGENT_DIR/workspace/requirements.txt" ]; then
    python -m venv "$AGENT_DIR/workspace/.venv"
    source "$AGENT_DIR/workspace/.venv/bin/activate"
    pip install -r "$AGENT_DIR/workspace/requirements.txt"
fi
```

### Health Check Hook
```bash
#!/bin/bash
# ~/.aany/hooks/health-check/memory_check.sh

AGENT_NAME=$1
MAX_MEMORY_MB=4096

# Get tmux session memory usage
MEMORY_USAGE=$(ps aux | grep "tmux.*agent-$AGENT_NAME" | awk '{sum += $6} END {print sum/1024}')

if (( $(echo "$MEMORY_USAGE > $MAX_MEMORY_MB" | bc -l) )); then
    echo "WARNING: Agent $AGENT_NAME using ${MEMORY_USAGE}MB (limit: ${MAX_MEMORY_MB}MB)"
    exit 1
fi

exit 0
```

## 4. CLI Extensions

### Config Management
```bash
# View/edit global config
aany config
aany config set api.claude.model "claude-3-sonnet-20240229"
aany config get logging.level

# Template management
aany template list
aany template create my-template
aany template edit research
aany template export research > my-research-template.yaml
```

### Backup & Restore
```bash
# Backup commands
aany backup create my-agent
aany backup list my-agent
aany backup restore my-agent --snapshot 2024-01-15-1200

# Auto-backup management
aany backup auto enable
aany backup auto disable
aany backup auto status
```

### Log Management
```bash
# View logs
aany logs                    # System logs
aany logs my-agent          # Agent logs
aany logs --follow my-agent # Tail logs
aany logs --since 1h        # Last hour
aany logs --grep "ERROR"    # Filter logs
```

### Hook Management
```bash
# Manage hooks
aany hook list
aany hook enable pre-start/setup_environment.sh
aany hook disable health-check/memory_check.sh
aany hook test pre-start/setup_environment.sh my-agent
```

## 5. Advanced Features

### Agent Collaboration
```yaml
# ~/.aany/pool/agents/researcher/.agent.yaml
collaboration:
  can_message:
    - "code-bot"
    - "writer-bot"
  shared_workspace: "/shared/research-project"
```

### Resource Monitoring
```bash
# Built-in resource monitoring
aany monitor              # System-wide dashboard
aany monitor my-agent    # Agent-specific metrics
aany monitor --export    # Export metrics to file
```

### Agent Scheduling
```yaml
# Schedule agents to run at specific times
schedule:
  daily_summary:
    agent: "reporter"
    cron: "0 9 * * *"  # 9 AM daily
    task: "Generate daily summary report"
    
  code_review:
    agent: "reviewer"
    cron: "0 */4 * * *"  # Every 4 hours
    task: "Review recent commits"
```

### Integration Features
```bash
# GitHub integration
aany integrate github --repo owner/repo
aany integrate github sync my-agent

# Slack notifications
aany integrate slack --webhook https://...
aany notify slack "Agent task completed"

# Export/Import
aany export my-agent > my-agent-export.tar.gz
aany import my-agent-export.tar.gz --name imported-agent
```

## 6. Security Features

### Secrets Management
```bash
# Store secrets securely
aany secret set OPENAI_KEY "sk-..."
aany secret list
aany secret delete OPENAI_KEY

# Use in agent config
claude:
  api_key: "${secret:CLAUDE_KEY}"
```

### Access Control
```yaml
# ~/.aany/config.yaml
security:
  require_confirmation:
    - "delete"
    - "export"
  
  allowed_commands:
    research-bot: ["read", "search", "write"]
    code-bot: ["all"]
```

## Implementation Priority

1. **Phase 1 - Core Config**
   - Global config.yaml
   - Template system
   - Basic hooks (pre-start, post-stop)

2. **Phase 2 - Management**
   - Log aggregation
   - Backup/restore
   - Resource monitoring

3. **Phase 3 - Advanced**
   - Agent collaboration
   - Scheduling
   - Integrations

This design makes `.aany` a powerful configuration and management hub for AI agents!