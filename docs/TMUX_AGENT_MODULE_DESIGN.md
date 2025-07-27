# Tmux Agent Module System Design

## Overview

The tmux-agent acts as a proxy/wrapper around tmux sessions, providing a module system for extending functionality.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Tmux Session                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │              Agent Process/Commands               │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                            ▲
                            │ Wraps/Monitors
                            │
┌─────────────────────────────────────────────────────────┐
│                    Tmux Agent Proxy                      │
│  ┌─────────────────────────────────────────────────┐   │
│  │                  Core Functions                   │   │
│  │  - Session Management                            │   │
│  │  - Command Interception                          │   │
│  │  - Event System                                  │   │
│  │  - Module Loading                                │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  ┌──────────────────┐  ┌──────────────────┐          │
│  │   Module Slots   │  │   Event Hooks    │          │
│  │                  │  │                  │          │
│  │  - on_start     │  │  - pre_command   │          │
│  │  - on_command   │  │  - post_command  │          │
│  │  - on_output    │  │  - on_error      │          │
│  │  - on_stop      │  │  - on_session    │          │
│  └──────────────────┘  └──────────────────┘          │
└─────────────────────────────────────────────────────────┘
                            ▲
                            │ Modules attach here
                            │
┌─────────────────────────────────────────────────────────┐
│                       Modules                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │
│  │    Hub      │  │   Logger    │  │  Security   │   │
│  │ Connector   │  │   Module    │  │   Module    │   │
│  └─────────────┘  └─────────────┘  └─────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Module Interface

```python
class TmuxAgentModule:
    """Base class for all tmux-agent modules"""
    
    def __init__(self, agent_config):
        self.config = agent_config
        self.enabled = True
    
    # Lifecycle hooks
    def on_agent_start(self, session_info):
        """Called when agent starts"""
        pass
    
    def on_agent_stop(self, session_info):
        """Called when agent stops"""
        pass
    
    # Command hooks
    def pre_command(self, command, context):
        """Called before command execution"""
        return command  # Can modify command
    
    def post_command(self, command, output, exit_code, context):
        """Called after command execution"""
        pass
    
    # Output hooks
    def on_output(self, output, context):
        """Called when output is received"""
        pass
    
    def on_error(self, error, context):
        """Called on errors"""
        pass
    
    # Session hooks
    def on_session_create(self, session_name):
        """Called when tmux session is created"""
        pass
    
    def on_session_destroy(self, session_name):
        """Called when tmux session is destroyed"""
        pass
```

## Hub Connector Module

The hub-connector becomes the first module implementation:

```python
class HubConnectorModule(TmuxAgentModule):
    """Module for connecting to aany-hub"""
    
    def __init__(self, agent_config):
        super().__init__(agent_config)
        self.hub_url = agent_config.get('hub_url')
        self.agent_id = agent_config.get('agent_id')
        self.connection = None
    
    def on_agent_start(self, session_info):
        """Connect to hub on start"""
        if self.hub_url:
            self.connection = self._connect_to_hub()
            self._log_to_hub("INFO", "Agent started", {"session": session_info})
    
    def post_command(self, command, output, exit_code, context):
        """Log commands to hub"""
        self._log_to_hub("INFO", f"Command executed: {command}", {
            "exit_code": exit_code,
            "output_preview": output[:100] if output else "",
            "context": context
        })
    
    def on_error(self, error, context):
        """Log errors to hub"""
        self._log_to_hub("ERROR", str(error), context)
```

## Module Loading

```yaml
# agent-config.yaml
agent:
  id: my-agent-001
  
modules:
  hub_connector:
    enabled: true
    hub_url: localhost:50052
    
  logger:
    enabled: true
    log_level: INFO
    
  security:
    enabled: false
    allowed_commands: ["ls", "echo"]
```

## Benefits

1. **Modular**: Add/remove functionality without touching core
2. **Clean**: Each module has clear responsibilities
3. **Extensible**: Easy to add new modules
4. **Configurable**: Enable/disable modules per agent
5. **Testable**: Modules can be tested independently