"""Terminal UI HTML template"""

TERMINAL_HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>aany-hub Terminal</title>
    <meta charset="utf-8">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/xterm@5.3.0/css/xterm.css" />
    <style>
        body {
            margin: 0;
            padding: 20px;
            background: #1e1e1e;
            color: #d4d4d4;
            font-family: 'Consolas', 'Monaco', monospace;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 20px;
            padding: 10px;
            background: #2d2d2d;
            border-radius: 5px;
        }
        
        .terminal-list {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        
        .terminal-card {
            background: #2d2d2d;
            border: 1px solid #3e3e3e;
            border-radius: 5px;
            padding: 15px;
            cursor: pointer;
            transition: all 0.2s;
        }
        
        .terminal-card:hover {
            background: #3e3e3e;
            border-color: #007acc;
        }
        
        .terminal-card.active {
            border-color: #007acc;
            background: #3e3e3e;
        }
        
        .terminal-container {
            background: #000;
            border: 1px solid #3e3e3e;
            border-radius: 5px;
            padding: 10px;
            margin-top: 20px;
            display: none;
        }
        
        .terminal-container.active {
            display: block;
        }
        
        #terminal {
            height: 600px;
        }
        
        .status {
            padding: 5px 10px;
            border-radius: 3px;
            font-size: 12px;
        }
        
        .status.connected {
            background: #4caf50;
            color: white;
        }
        
        .status.disconnected {
            background: #f44336;
            color: white;
        }
        
        .controls {
            margin-top: 10px;
            display: flex;
            gap: 10px;
        }
        
        button {
            background: #007acc;
            color: white;
            border: none;
            padding: 8px 16px;
            border-radius: 3px;
            cursor: pointer;
            font-size: 14px;
        }
        
        button:hover {
            background: #005a9e;
        }
        
        button:disabled {
            background: #555;
            cursor: not-allowed;
        }
        
        .agent-info {
            font-size: 12px;
            color: #888;
            margin-top: 5px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>aany-hub Terminal Access</h1>
            <div>
                <span id="connection-status" class="status disconnected">Disconnected</span>
                <button onclick="refreshTerminals()">Refresh</button>
            </div>
        </div>
        
        <h2>Available Terminal Sessions</h2>
        <div id="terminal-list" class="terminal-list">
            <div class="loading">Loading terminals...</div>
        </div>
        
        <div id="terminal-container" class="terminal-container">
            <div class="controls">
                <button onclick="clearTerminal()">Clear</button>
                <button onclick="disconnectTerminal()">Disconnect</button>
                <span id="terminal-info" style="margin-left: 20px; color: #888;"></span>
            </div>
            <div id="terminal"></div>
        </div>
    </div>
    
    <script src="https://cdn.jsdelivr.net/npm/xterm@5.3.0/lib/xterm.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/xterm-addon-fit@0.8.0/lib/xterm-addon-fit.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/xterm-addon-web-links@0.9.0/lib/xterm-addon-web-links.js"></script>
    <script>
        let terminal;
        let fitAddon;
        let webLinksAddon;
        let ws;
        let currentSessionId = null;
        
        // Initialize xterm.js
        function initTerminal() {
            terminal = new Terminal({
                cursorBlink: true,
                fontSize: 14,
                fontFamily: 'Consolas, Monaco, monospace',
                theme: {
                    background: '#1e1e1e',
                    foreground: '#d4d4d4',
                    cursor: '#d4d4d4',
                    selection: '#264f78',
                    black: '#000000',
                    red: '#cd3131',
                    green: '#0dbc79',
                    yellow: '#e5e510',
                    blue: '#2472c8',
                    magenta: '#bc3fbc',
                    cyan: '#11a8cd',
                    white: '#e5e5e5',
                    brightBlack: '#666666',
                    brightRed: '#f14c4c',
                    brightGreen: '#23d18b',
                    brightYellow: '#f5f543',
                    brightBlue: '#3b8eea',
                    brightMagenta: '#d670d6',
                    brightCyan: '#29b8db',
                    brightWhite: '#e5e5e5'
                }
            });
            
            fitAddon = new FitAddon.FitAddon();
            webLinksAddon = new WebLinksAddon.WebLinksAddon();
            
            terminal.loadAddon(fitAddon);
            terminal.loadAddon(webLinksAddon);
            
            terminal.open(document.getElementById('terminal'));
            fitAddon.fit();
            
            // Handle terminal input
            terminal.onData(data => {
                if (ws && ws.readyState === WebSocket.OPEN) {
                    ws.send(JSON.stringify({
                        type: 'input',
                        data: data
                    }));
                }
            });
            
            // Handle window resize
            window.addEventListener('resize', () => {
                fitAddon.fit();
                if (ws && ws.readyState === WebSocket.OPEN) {
                    ws.send(JSON.stringify({
                        type: 'resize',
                        rows: terminal.rows,
                        cols: terminal.cols
                    }));
                }
            });
        }
        
        // Connect to terminal WebSocket
        function connectToTerminal(sessionId) {
            if (ws) {
                ws.close();
            }
            
            currentSessionId = sessionId;
            
            // Update UI
            document.querySelectorAll('.terminal-card').forEach(card => {
                card.classList.remove('active');
            });
            document.querySelector(`[data-session-id="${sessionId}"]`).classList.add('active');
            
            document.getElementById('terminal-container').classList.add('active');
            document.getElementById('terminal-info').textContent = `Session: ${sessionId}`;
            
            // Clear terminal
            if (terminal) {
                terminal.clear();
            } else {
                initTerminal();
            }
            
            // Connect WebSocket
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            ws = new WebSocket(`${protocol}//${window.location.host}/ws/terminal/${sessionId}`);
            
            ws.onopen = () => {
                document.getElementById('connection-status').textContent = 'Connected';
                document.getElementById('connection-status').className = 'status connected';
                terminal.writeln('\\x1b[32mConnected to terminal session\\x1b[0m');
            };
            
            ws.onmessage = (event) => {
                const data = JSON.parse(event.data);
                
                switch (data.type) {
                    case 'output':
                        terminal.write(data.data);
                        break;
                    case 'connected':
                        terminal.writeln(`\\x1b[33mConnected to ${data.agent_id} - ${data.tmux_session}\\x1b[0m`);
                        break;
                    case 'error':
                        terminal.writeln(`\\x1b[31mError: ${data.message}\\x1b[0m`);
                        break;
                }
            };
            
            ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                terminal.writeln('\\x1b[31mConnection error\\x1b[0m');
            };
            
            ws.onclose = () => {
                document.getElementById('connection-status').textContent = 'Disconnected';
                document.getElementById('connection-status').className = 'status disconnected';
                terminal.writeln('\\x1b[31mDisconnected from terminal session\\x1b[0m');
            };
        }
        
        // Refresh terminal list
        async function refreshTerminals() {
            try {
                const response = await fetch('/api/terminals');
                const data = await response.json();
                
                const terminalList = document.getElementById('terminal-list');
                
                if (data.terminals.length === 0) {
                    terminalList.innerHTML = '<div style="color: #888;">No terminal sessions available</div>';
                    return;
                }
                
                terminalList.innerHTML = data.terminals.map(term => `
                    <div class="terminal-card" data-session-id="${term.session_id}" onclick="connectToTerminal('${term.session_id}')">
                        <h3>${term.tmux_session}</h3>
                        <div class="agent-info">Agent: ${term.agent_id}</div>
                        <div class="agent-info">Status: ${term.status}</div>
                        <div class="agent-info">Created: ${new Date(term.created_at).toLocaleString()}</div>
                    </div>
                `).join('');
                
            } catch (error) {
                console.error('Error fetching terminals:', error);
                document.getElementById('terminal-list').innerHTML = '<div style="color: #f44336;">Error loading terminals</div>';
            }
        }
        
        // Clear terminal
        function clearTerminal() {
            if (terminal) {
                terminal.clear();
            }
        }
        
        // Disconnect from terminal
        function disconnectTerminal() {
            if (ws) {
                ws.close();
            }
            document.getElementById('terminal-container').classList.remove('active');
            document.querySelectorAll('.terminal-card').forEach(card => {
                card.classList.remove('active');
            });
        }
        
        // Initialize
        document.addEventListener('DOMContentLoaded', () => {
            refreshTerminals();
            
            // Auto-refresh terminal list
            setInterval(refreshTerminals, 5000);
        });
    </script>
</body>
</html>
"""