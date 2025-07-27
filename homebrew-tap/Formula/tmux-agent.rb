class TmuxAgent < Formula
  desc "Agent-friendly tmux wrapper with safety controls"
  homepage "https://github.com/ironluffy/agent-anywhere"
  license "MIT"
  
  # Use different installation methods
  if OS.mac?
    # For macOS, use pre-built universal binary
    url "https://github.com/ironluffy/agent-anywhere/releases/download/v0.2.0/tmux-agent-v0.2.0-macos.tar.gz"
    sha256 "PLACEHOLDER_SHA256_MACOS"  # Update when release is created
  else
    # For Linux, build from source
    url "https://github.com/ironluffy/agent-anywhere/archive/refs/tags/v0.2.0.tar.gz"
    sha256 "PLACEHOLDER_SHA256_SOURCE"  # Update when release is created
    depends_on "rust" => :build
  end
  
  depends_on "tmux"
  
  def install
    if OS.mac?
      # Install pre-built binary
      bin.install "tmux-agent"
      
      # Install completions if they exist
      if File.exist?("completions/tmux-agent.bash")
        bash_completion.install "completions/tmux-agent.bash"
      end
      if File.exist?("completions/tmux-agent.zsh")
        zsh_completion.install "completions/tmux-agent.zsh" => "_tmux-agent"
      end
      
      # Install documentation
      doc.install Dir["*.md"]
    else
      # Build from source on Linux
      cd "tmux-agent-proxy" do
        system "cargo", "build", "--release", "--bin", "tmux-agent"
        bin.install "target/release/tmux-agent"
        
        # Install completions
        bash_completion.install "completions/tmux-agent.bash" if File.exist?("completions/tmux-agent.bash")
        zsh_completion.install "completions/tmux-agent.zsh" => "_tmux-agent" if File.exist?("completions/tmux-agent.zsh")
        
        # Install documentation
        doc.install Dir["*.md"]
      end
    end
  end
  
  def caveats
    <<~EOS
      tmux-agent has been installed! 🎉
      
      Quick start:
        tmux-agent new my-bot      # Create agent session
        tmux-agent monitor my-bot  # Safe monitoring
        tmux-agent help           # Show all commands
      
      Aliases available:
        tma  = tmux-agent
        tmon = tmux-agent monitor
        
      To enable aliases, add to your shell config:
        alias tma='tmux-agent'
        alias tmon='tmux-agent monitor'
    EOS
  end
  
  test do
    system "#{bin}/tmux-agent", "version"
  end
end