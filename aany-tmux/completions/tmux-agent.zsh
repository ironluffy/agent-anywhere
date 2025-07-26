#compdef tmux-agent
# ZSH completion for tmux-agent

_tmux_agent() {
    local -a commands sessions
    
    commands=(
        'help:Show help information'
        'new:Create new agent-controlled session'
        'attach:Attach to session (interactive mode)'
        'monitor:Attach in read-only mode (safe)'
        'list:List all sessions with agent info'
        'kill:Kill a session'
        'health:Check session health'
        'cleanup:Clean up extra panes in session'
        'protect:Display protection warning in session'
        'status:Show agent system status'
        'a:Alias for attach'
        'mon:Alias for monitor'
        'ls:Alias for list'
    )
    
    # Get tmux sessions if available
    if command -v tmux &> /dev/null; then
        sessions=(${(f)"$(tmux list-sessions -F '#{session_name}' 2>/dev/null)"})
    fi
    
    case $state in
        args)
            case $line[1] in
                attach|a|monitor|mon|kill|health|cleanup|protect)
                    _describe -t sessions 'tmux sessions' sessions
                    ;;
                new)
                    _message 'session name'
                    ;;
                *)
                    _describe -t commands 'tmux-agent commands' commands
                    ;;
            esac
            ;;
    esac
    
    _arguments \
        '1: :->command' \
        '2: :->args'
    
    case $state in
        command)
            _describe -t commands 'tmux-agent commands' commands
            ;;
    esac
}

_tmux_agent "$@"