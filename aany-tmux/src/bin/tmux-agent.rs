use aany_tmux::{SafeTmuxProxy, TmuxSession};
use std::env;
use std::process::{Command, exit};
use std::io::{self, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help();
        exit(0);
    }
    
    match args[1].as_str() {
        "help" | "--help" | "-h" => print_help(),
        "version" | "--version" | "-v" => print_version(),
        
        // tmux-compatible commands
        "new-session" | "new" => handle_new_session(&args),
        "attach-session" | "attach" | "a" => handle_attach(&args),
        "list-sessions" | "ls" => handle_list_sessions(&args),
        "kill-session" => handle_kill_session(&args),
        
        // agent-specific extensions
        "monitor" | "mon" => handle_monitor(&args),
        "health" => handle_health(&args),
        "cleanup" => handle_cleanup(&args),
        "protect" => handle_protect(&args),
        "status" => handle_status(&args),
        "update" => handle_update(&args),
        
        _ => {
            // Pass through to regular tmux
            pass_through_to_tmux(&args);
        }
    }
}

fn print_help() {
    println!("tmux-agent v{} - {}", VERSION, DESCRIPTION);
    println!("by {}", AUTHORS);
    println!();
    println!("USAGE:");
    println!("    tmux-agent <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS (tmux-compatible):");
    println!("    new-session [-s name]    Create new agent-controlled session");
    println!("    attach-session [-t name] Attach to session (interactive - use with caution!)");
    println!("    list-sessions            List all sessions");
    println!("    kill-session [-t name]   Kill a session");
    println!();
    println!("COMMANDS (agent-specific):");
    println!("    monitor <session>        Attach in read-only mode (safe)");
    println!("    health <session>         Check session health");
    println!("    cleanup <session>        Clean up extra panes in session");
    println!("    protect <session>        Display protection warning in session");
    println!("    status                   Show agent system status");
    println!("    version                  Show version information");
    println!("    update                   Check for updates");
    println!();
    println!("SHORTCUTS:");
    println!("    a                    Alias for attach");
    println!("    mon                  Alias for monitor");
    println!("    ls                   Alias for list");
    println!("    -v, --version        Show version");
    println!("    -h, --help           Show this help");
    println!();
    println!("EXAMPLES:");
    println!("    tmux-agent new my-agent      # Create agent session");
    println!("    tmux-agent monitor my-agent  # Safe monitoring");
    println!("    tmux-agent health my-agent   # Check health");
    println!("    tmux-agent cleanup my-agent  # Fix pane issues");
    println!();
    println!("SAFETY:");
    println!("    - 'monitor' command always attaches read-only");
    println!("    - 'attach' shows warnings for interactive mode");
    println!("    - Sessions are protected against pane proliferation");
    println!();
    println!("For regular tmux commands, just use them normally:");
    println!("    tmux-agent list-keys");
    println!("    tmux-agent show-options -g");
}

fn print_version() {
    println!("tmux-agent v{}", VERSION);
    println!("{}", DESCRIPTION);
    println!("by {}", AUTHORS);
    println!();
    println!("Repository: https://github.com/jayhansuh/agent-anywhere");
    println!("License: MIT");
}

fn handle_new_session(args: &[String]) {
    let mut session_name: Option<String> = None;
    let mut i = 2;
    
    // Parse tmux-style arguments
    while i < args.len() {
        match args[i].as_str() {
            "-s" => {
                if i + 1 < args.len() {
                    session_name = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: -s requires a session name");
                    exit(1);
                }
            }
            _ => {
                // If no flag, assume it's the session name (for backward compatibility)
                session_name = Some(args[i].clone());
                i += 1;
            }
        }
    }
    
    let session_name = session_name.unwrap_or_else(|| {
        // Generate default name if not provided
        format!("agent-{}", std::process::id())
    });
    
    // Check if session exists
    if TmuxSession::exists(&session_name) {
        eprintln!("Error: Session '{}' already exists", session_name);
        exit(1);
    }
    
    // Create new session
    match TmuxSession::create(&session_name, true) {
        Ok(_) => {
            println!("✅ Created agent-controlled session: {}", session_name);
            
            // Set up initial protection
            if let Ok(proxy) = SafeTmuxProxy::new(session_name.to_string(), 0, 0) {
                if let Ok(detector) = proxy.with_interference_detection() {
                    let _ = detector.display_lock_message();
                    let _ = detector.proxy.send_line(&format!(
                        "echo 'Session created with tmux-agent protection'"
                    ));
                }
            }
            
            println!("📺 To monitor: tmux-agent monitor {}", session_name);
            println!("⚠️  To attach: tmux-agent attach {}", session_name);
        }
        Err(e) => {
            eprintln!("Error creating session: {}", e);
            exit(1);
        }
    }
}

fn handle_attach(args: &[String]) {
    let mut session_name: Option<String> = None;
    let mut i = 2;
    
    // Parse tmux-style arguments
    while i < args.len() {
        match args[i].as_str() {
            "-t" => {
                if i + 1 < args.len() {
                    session_name = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: -t requires a session name");
                    exit(1);
                }
            }
            _ => {
                // If no flag, assume it's the session name (for backward compatibility)
                session_name = Some(args[i].clone());
                i += 1;
            }
        }
    }
    
    let session_name = session_name.unwrap_or_else(|| {
        eprintln!("Error: Session name required");
        eprintln!("Usage: tmux-agent attach-session [-t session-name]");
        exit(1);
    });
    
    if !TmuxSession::exists(&session_name) {
        eprintln!("Error: Session '{}' not found", session_name);
        exit(1);
    }
    
    // Show warning
    println!("⚠️  WARNING: Attaching in INTERACTIVE mode!");
    println!("⚠️  You can type and potentially interfere with the agent.");
    println!();
    println!("🛡️  For safe monitoring, use: tmux-agent monitor {}", session_name);
    println!();
    print!("Continue with interactive attach? [y/N] ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Cancelled.");
        exit(0);
    }
    
    // Attach interactively
    let status = Command::new("tmux")
        .args(["attach", "-t", &session_name])
        .status()
        .expect("Failed to execute tmux");
        
    exit(status.code().unwrap_or(1));
}

fn handle_monitor(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Error: Session name required");
        eprintln!("Usage: tmux-agent monitor <session-name>");
        exit(1);
    }
    
    let session_name = &args[2];
    
    if !TmuxSession::exists(&session_name) {
        eprintln!("Error: Session '{}' not found", session_name);
        exit(1);
    }
    
    println!("👁️  Attaching in READ-ONLY mode (safe monitoring)");
    println!("📝 You cannot type or interfere with the agent");
    println!("🔚 Detach with: Ctrl+B, d");
    println!();
    
    // Attach read-only
    let status = Command::new("tmux")
        .args(["attach", "-r", "-t", &session_name])
        .status()
        .expect("Failed to execute tmux");
        
    exit(status.code().unwrap_or(1));
}

fn handle_list_sessions(_args: &[String]) {
    println!("🤖 Agent-Controlled Sessions:");
    println!("============================");
    
    match TmuxSession::list() {
        Ok(sessions) => {
            if sessions.is_empty() {
                println!("No sessions found.");
                return;
            }
            
            for session in sessions {
                print!("📺 {}", session);
                
                // Check if it's likely an agent session
                if session.contains("agent") || session.contains("bot") || session.contains("auto") {
                    print!(" [🤖 likely agent-controlled]");
                }
                
                // Try to get health info
                if let Ok(proxy) = SafeTmuxProxy::new(session.clone(), 0, 0) {
                    if let Ok(health) = proxy.get_pane_health() {
                        match health.status {
                            aany_tmux::HealthStatus::Healthy => print!(" ✅"),
                            aany_tmux::HealthStatus::Warning => print!(" ⚠️"),
                            aany_tmux::HealthStatus::Critical => print!(" ❌"),
                        }
                        print!(" {} panes", health.pane_count);
                    }
                }
                
                println!();
            }
            
            println!();
            println!("Monitor with: tmux-agent monitor <session>");
        }
        Err(e) => {
            eprintln!("Error listing sessions: {}", e);
            exit(1);
        }
    }
}

fn handle_kill_session(args: &[String]) {
    let mut session_name: Option<String> = None;
    let mut i = 2;
    
    // Parse tmux-style arguments
    while i < args.len() {
        match args[i].as_str() {
            "-t" => {
                if i + 1 < args.len() {
                    session_name = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: -t requires a session name");
                    exit(1);
                }
            }
            _ => {
                // If no flag, assume it's the session name (for backward compatibility)
                session_name = Some(args[i].clone());
                i += 1;
            }
        }
    }
    
    let session_name = session_name.unwrap_or_else(|| {
        eprintln!("Error: Session name required");
        eprintln!("Usage: tmux-agent kill-session [-t session-name]");
        exit(1);
    });
    
    print!("Kill session '{}'? [y/N] ", session_name);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Cancelled.");
        exit(0);
    }
    
    match TmuxSession::kill(&session_name) {
        Ok(_) => println!("✅ Killed session: {}", session_name),
        Err(e) => {
            eprintln!("Error killing session: {}", e);
            exit(1);
        }
    }
}

fn handle_health(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Error: Session name required");
        eprintln!("Usage: tmux-agent health <session-name>");
        exit(1);
    }
    
    let session_name = &args[2];
    
    match SafeTmuxProxy::new(session_name.to_string(), 0, 0) {
        Ok(proxy) => {
            match proxy.get_pane_health() {
                Ok(health) => {
                    println!("Health Report for '{}':", session_name);
                    println!("========================");
                    health.report();
                }
                Err(e) => {
                    eprintln!("Error checking health: {}", e);
                    exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error accessing session: {}", e);
            exit(1);
        }
    }
}

fn handle_cleanup(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Error: Session name required");
        eprintln!("Usage: tmux-agent cleanup <session-name>");
        exit(1);
    }
    
    let session_name = &args[2];
    
    match SafeTmuxProxy::new(session_name.to_string(), 0, 0) {
        Ok(proxy) => {
            println!("🧹 Cleaning up session '{}'...", session_name);
            
            match proxy.cleanup_extra_panes() {
                Ok(cleaned) => {
                    if cleaned > 0 {
                        println!("✅ Cleaned {} extra pane(s)", cleaned);
                    } else {
                        println!("✅ No cleanup needed - session is healthy");
                    }
                }
                Err(e) => {
                    eprintln!("Error during cleanup: {}", e);
                    exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error accessing session: {}", e);
            exit(1);
        }
    }
}

fn handle_protect(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Error: Session name required");
        eprintln!("Usage: tmux-agent protect <session-name>");
        exit(1);
    }
    
    let session_name = &args[2];
    
    match SafeTmuxProxy::new(session_name.to_string(), 0, 0) {
        Ok(proxy) => {
            match proxy.with_interference_detection() {
                Ok(detector) => {
                    println!("🛡️  Adding protection warning to '{}'...", session_name);
                    
                    match detector.display_lock_message() {
                        Ok(_) => println!("✅ Protection warning displayed"),
                        Err(e) => {
                            eprintln!("Error displaying warning: {}", e);
                            exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error setting up protection: {}", e);
                    exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error accessing session: {}", e);
            exit(1);
        }
    }
}

fn handle_status(_args: &[String]) {
    println!("🤖 TMux Agent System Status");
    println!("==========================");
    
    let sessions = TmuxSession::list().unwrap_or_default();
    let mut healthy = 0;
    let mut warning = 0;
    let mut critical = 0;
    
    for session in &sessions {
        if let Ok(proxy) = SafeTmuxProxy::new(session.clone(), 0, 0) {
            if let Ok(health) = proxy.get_pane_health() {
                match health.status {
                    aany_tmux::HealthStatus::Healthy => healthy += 1,
                    aany_tmux::HealthStatus::Warning => warning += 1,
                    aany_tmux::HealthStatus::Critical => critical += 1,
                }
            }
        }
    }
    
    println!("Total sessions: {}", sessions.len());
    println!("  ✅ Healthy: {}", healthy);
    println!("  ⚠️  Warning: {}", warning);
    println!("  ❌ Critical: {}", critical);
    println!();
    
    if critical > 0 {
        println!("⚠️  Some sessions need attention!");
        println!("Run 'tmux-agent health <session>' for details");
        println!("Run 'tmux-agent cleanup <session>' to fix");
    } else if warning > 0 {
        println!("📝 Some sessions have warnings");
    } else {
        println!("✅ All sessions are healthy!");
    }
}

fn pass_through_to_tmux(args: &[String]) {
    // Pass all arguments except the program name to tmux
    let tmux_args: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
    
    let status = Command::new("tmux")
        .args(&tmux_args)
        .status()
        .expect("Failed to execute tmux");
        
    exit(status.code().unwrap_or(1));
}

fn handle_update(_args: &[String]) {
    println!("🔄 Checking for tmux-agent updates...");
    println!("   Current version: v{}", VERSION);
    println!();
    
    // Check if we're in a git repo
    let git_status = Command::new("git")
        .args(["status", "--porcelain"])
        .output();
        
    if git_status.is_ok() {
        // We're in the development directory
        println!("📦 Development mode detected");
        println!();
        println!("To update, run:");
        println!("   git pull");
        println!("   cargo build --release --bin tmux-agent");
        println!("   ./install.sh");
    } else {
        // Installed version
        println!("To update tmux-agent:");
        println!();
        println!("Option 1 - Quick update:");
        println!("   curl -sSL https://raw.githubusercontent.com/jayhansuh/agent-anywhere/genesis/tmux-agent-proxy/install-oneliner.sh | bash");
        println!();
        println!("Option 2 - Manual update:");
        println!("   cd ~/agent-anywhere/tmux-agent-proxy");
        println!("   git pull");
        println!("   ./install.sh");
        println!();
        println!("📝 Check releases at:");
        println!("   https://github.com/jayhansuh/agent-anywhere/releases");
    }
}