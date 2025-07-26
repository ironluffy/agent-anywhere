#!/bin/bash
# Bash completion for tmux-agent

_tmux_agent() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    # Main commands
    main_opts="help new attach monitor list kill health cleanup protect status"
    
    # Aliases
    case "${COMP_WORDS[1]}" in
        a) COMP_WORDS[1]="attach" ;;
        mon) COMP_WORDS[1]="monitor" ;;
        ls) COMP_WORDS[1]="list" ;;
    esac
    
    # If completing a session name
    case "${prev}" in
        attach|monitor|kill|health|cleanup|protect|a|mon)
            # Get tmux sessions
            if command -v tmux &> /dev/null; then
                local sessions=$(tmux list-sessions -F "#{session_name}" 2>/dev/null)
                COMPREPLY=( $(compgen -W "${sessions}" -- ${cur}) )
            fi
            return 0
            ;;
        new)
            # Suggest session name patterns
            COMPREPLY=( $(compgen -W "agent- bot- worker- test-" -- ${cur}) )
            return 0
            ;;
    esac
    
    # Complete main commands
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        # Include both commands and common tmux commands
        all_opts="${main_opts} a mon ls"
        
        # Add common tmux commands for pass-through
        tmux_cmds="list-sessions list-windows list-panes show-options list-keys"
        
        COMPREPLY=( $(compgen -W "${all_opts} ${tmux_cmds}" -- ${cur}) )
        return 0
    fi
    
    # Help with specific commands
    case "${COMP_WORDS[1]}" in
        help|--help|-h|list|ls|status)
            # No more arguments needed
            return 0
            ;;
    esac
}

complete -F _tmux_agent tmux-agent