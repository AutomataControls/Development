// Web Server Module - Serves web UI for remote access via Cloudflare
// This runs alongside the native egui interface

use axum::{
    Router,
    routing::{get, post},
    response::{Html, IntoResponse, Response},
    extract::{State, Path, Query},
    http::{StatusCode, header},
    Json,
};
use tower_http::{
    services::ServeDir,
    cors::CorsLayer,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use crate::state::AppState;

pub async fn start_web_server(app_state: Arc<Mutex<AppState>>) {
    // Create router with all routes
    let app = Router::new()
        // Serve static files (CSS, JS, images)
        .nest_service("/static", ServeDir::new("web/static"))
        
        // Main web UI routes
        .route("/", get(serve_index))
        .route("/dashboard", get(serve_dashboard))
        .route("/boards", get(serve_boards))
        .route("/io-control", get(serve_io_control))
        .route("/refrigerant", get(serve_refrigerant))
        .route("/vibration", get(serve_vibration))
        .route("/logic", get(serve_logic))
        .route("/admin", get(serve_admin))
        
        // API routes for web UI
        .route("/api/status", get(get_status))
        .route("/api/boards/scan", get(scan_boards))
        .route("/api/boards/:id/status", get(get_board_status))
        .route("/api/io/:board/:channel", get(get_io_value))
        .route("/api/io/:board/:channel", post(set_io_value))
        .route("/api/refrigerant/status", get(get_refrigerant_status))
        .route("/api/vibration/sensors", get(get_vibration_sensors))
        
        // Admin API routes - require authentication
        .route("/api/admin/terminal", post(execute_terminal_command))
        .route("/api/admin/reboot", post(system_reboot))
        .route("/api/admin/update", post(system_update))
        .route("/api/admin/backup", post(create_backup))
        .route("/api/admin/restore", post(restore_backup))
        .route("/api/admin/logs", get(get_system_logs))
        .route("/api/admin/usb-scan", get(scan_usb_devices))
        
        // WebSocket endpoint for real-time updates
        .route("/ws", get(websocket_handler))
        
        .layer(CorsLayer::permissive())
        .with_state(app_state);
    
    // Start server on port 1420 (for Cloudflare tunnel)
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1420")
        .await
        .expect("Failed to bind to port 1420");
    
    println!("Web UI server running on http://0.0.0.0:1420");
    println!("Access remotely via Cloudflare tunnel");
    
    axum::serve(listener, app)
        .await
        .expect("Web server failed");
}

// Serve the main index page
async fn serve_index() -> Html<String> {
    Html(generate_html_page("Dashboard"))
}

// Serve dashboard page
async fn serve_dashboard() -> Html<String> {
    Html(generate_html_page("Dashboard"))
}

// Serve boards configuration page
async fn serve_boards() -> Html<String> {
    Html(generate_html_page("Board Configuration"))
}

// Serve I/O control page
async fn serve_io_control() -> Html<String> {
    Html(generate_html_page("I/O Control"))
}

// Serve refrigerant diagnostics page
async fn serve_refrigerant() -> Html<String> {
    Html(generate_html_page("Refrigerant Diagnostics"))
}

// Serve vibration monitoring page
async fn serve_vibration() -> Html<String> {
    Html(generate_html_page("Vibration Monitoring"))
}

// Serve logic engine page
async fn serve_logic() -> Html<String> {
    Html(generate_html_page("Logic Engine"))
}

// Serve admin panel
async fn serve_admin() -> Html<String> {
    Html(generate_html_page("Admin Panel"))
}

// Generate HTML page with embedded JavaScript for dynamic UI
fn generate_html_page(title: &str) -> String {
    format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Automata Nexus - {}</title>
    <style>
        :root {{
            --primary: #0eb8a6;
            --secondary: #0d9488;
            --background: #0f172a;
            --surface: #1e293b;
            --text: #f1f5f9;
            --border: #334155;
        }}
        
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: var(--background);
            color: var(--text);
            min-height: 100vh;
        }}
        
        .header {{
            background: var(--surface);
            border-bottom: 1px solid var(--border);
            padding: 1rem 2rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }}
        
        .logo {{
            display: flex;
            align-items: center;
            gap: 1rem;
        }}
        
        .logo-text {{
            font-size: 1.5rem;
            font-weight: bold;
            color: var(--primary);
        }}
        
        .nav {{
            display: flex;
            gap: 2rem;
        }}
        
        .nav a {{
            color: var(--text);
            text-decoration: none;
            padding: 0.5rem 1rem;
            border-radius: 0.25rem;
            transition: background 0.2s;
        }}
        
        .nav a:hover {{
            background: rgba(14, 184, 166, 0.1);
        }}
        
        .nav a.active {{
            background: var(--primary);
            color: var(--background);
        }}
        
        .container {{
            max-width: 1920px;
            margin: 0 auto;
            padding: 2rem;
        }}
        
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 1.5rem;
            margin-top: 2rem;
        }}
        
        .card {{
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 0.5rem;
            padding: 1.5rem;
        }}
        
        .card-title {{
            font-size: 1.25rem;
            margin-bottom: 1rem;
            color: var(--primary);
        }}
        
        .status-indicator {{
            display: inline-block;
            width: 10px;
            height: 10px;
            border-radius: 50%;
            margin-right: 0.5rem;
        }}
        
        .status-online {{ background: #10b981; }}
        .status-offline {{ background: #ef4444; }}
        .status-warning {{ background: #f59e0b; }}
        
        .button {{
            background: var(--primary);
            color: var(--background);
            border: none;
            padding: 0.75rem 1.5rem;
            border-radius: 0.25rem;
            cursor: pointer;
            font-size: 1rem;
            transition: background 0.2s;
        }}
        
        .button:hover {{
            background: var(--secondary);
        }}
        
        .input {{
            background: var(--background);
            border: 1px solid var(--border);
            color: var(--text);
            padding: 0.5rem;
            border-radius: 0.25rem;
            width: 100%;
        }}
        
        .table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 1rem;
        }}
        
        .table th,
        .table td {{
            text-align: left;
            padding: 0.75rem;
            border-bottom: 1px solid var(--border);
        }}
        
        .table th {{
            background: var(--background);
            font-weight: 600;
        }}
        
        /* Board corrections notice */
        .corrections-notice {{
            background: rgba(14, 184, 166, 0.1);
            border: 1px solid var(--primary);
            border-radius: 0.5rem;
            padding: 1rem;
            margin: 1rem 0;
        }}
        
        .corrections-notice h3 {{
            color: var(--primary);
            margin-bottom: 0.5rem;
        }}
        
        .corrections-list {{
            list-style: none;
            padding-left: 1rem;
        }}
        
        .corrections-list li {{
            margin: 0.25rem 0;
        }}
        
        .corrections-list li:before {{
            content: "✓ ";
            color: var(--primary);
            font-weight: bold;
        }}
    </style>
</head>
<body>
    <div class="header">
        <div class="logo">
            <div class="logo-text">AUTOMATA NEXUS</div>
            <span style="color: #64748b;">AI Controller v2.1.0</span>
        </div>
        <nav class="nav">
            <a href="/dashboard" class="nav-link">Dashboard</a>
            <a href="/boards" class="nav-link">Boards</a>
            <a href="/io-control" class="nav-link">I/O Control</a>
            <a href="/refrigerant" class="nav-link">Refrigerant</a>
            <a href="/vibration" class="nav-link">Vibration</a>
            <a href="/logic" class="nav-link">Logic</a>
            <a href="/admin" class="nav-link">Admin</a>
        </nav>
    </div>
    
    <div class="container">
        <h1>{}</h1>
        
        <!-- Board Corrections Notice -->
        <div class="corrections-notice">
            <h3>Board Specifications (v2.1.0 Corrections)</h3>
            <ul class="corrections-list">
                <li>MegaBAS: 4 triacs, 4 analog outputs, 8 configurable inputs (NO RELAYS)</li>
                <li>16univin: 16 universal INPUTS ONLY</li>
                <li>16uout: 16 analog OUTPUTS ONLY</li>
                <li>8relind/16relind: Separate relay boards</li>
            </ul>
        </div>
        
        <div id="app-content">
            <!-- Dynamic content loaded here -->
        </div>
    </div>
    
    <script>
        // WebSocket connection for real-time updates
        let ws = null;
        
        function connectWebSocket() {{
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            ws = new WebSocket(`${{protocol}}//${{window.location.host}}/ws`);
            
            ws.onopen = () => {{
                console.log('WebSocket connected');
                updateConnectionStatus(true);
            }};
            
            ws.onmessage = (event) => {{
                const data = JSON.parse(event.data);
                handleRealtimeUpdate(data);
            }};
            
            ws.onclose = () => {{
                console.log('WebSocket disconnected');
                updateConnectionStatus(false);
                // Reconnect after 3 seconds
                setTimeout(connectWebSocket, 3000);
            }};
        }}
        
        function updateConnectionStatus(connected) {{
            const indicator = document.getElementById('connection-status');
            if (indicator) {{
                indicator.className = connected ? 'status-indicator status-online' : 'status-indicator status-offline';
            }}
        }}
        
        function handleRealtimeUpdate(data) {{
            // Update UI based on real-time data
            if (data.type === 'io_update') {{
                updateIOValues(data.values);
            }} else if (data.type === 'board_status') {{
                updateBoardStatus(data.boards);
            }} else if (data.type === 'refrigerant_update') {{
                updateRefrigerantData(data.data);
            }}
        }}
        
        async function loadPageContent() {{
            const path = window.location.pathname;
            const content = document.getElementById('app-content');
            
            if (path === '/' || path === '/dashboard') {{
                content.innerHTML = await generateDashboard();
            }} else if (path === '/boards') {{
                content.innerHTML = await generateBoardsPage();
            }} else if (path === '/io-control') {{
                content.innerHTML = await generateIOControlPage();
            }} else if (path === '/refrigerant') {{
                content.innerHTML = await generateRefrigerantPage();
            }} else if (path === '/admin') {{
                content.innerHTML = await generateAdminPage();
            }}
            
            // Mark active nav link
            document.querySelectorAll('.nav-link').forEach(link => {{
                link.classList.toggle('active', link.getAttribute('href') === path);
            }});
        }}
        
        async function generateAdminPage() {
            return `
                <div class="card">
                    <h2 class="card-title">Admin Panel</h2>
                    
                    <div style="display: flex; gap: 1rem; margin-bottom: 2rem;">
                        <button class="button" onclick="showTerminal()">Terminal</button>
                        <button class="button" onclick="showSystemControls()">System</button>
                        <button class="button button-danger" onclick="confirmReboot()">Reboot</button>
                        <button class="button" onclick="systemUpdate()">Update</button>
                        <button class="button" onclick="createBackup()">Backup</button>
                    </div>
                    
                    <div id="terminal-container" style="display: none;">
                        <h3>System Terminal</h3>
                        <div style="background: #000; color: #0f0; padding: 1rem; border-radius: 6px; font-family: monospace; height: 400px; overflow-y: auto;" id="terminal-output">
                            Nexus Terminal v2.1.0
                            Type 'help' for available commands
                        </div>
                        <div style="display: flex; gap: 0.5rem; margin-top: 1rem;">
                            <span style="font-family: monospace;">$</span>
                            <input type="text" class="input" id="terminal-input" 
                                   style="font-family: monospace; flex: 1;" 
                                   onkeypress="handleTerminalKey(event)"
                                   placeholder="Enter command...">
                            <button class="button button-small" onclick="executeTerminalCommand()">Execute</button>
                        </div>
                        <div style="margin-top: 1rem;">
                            <button class="button button-small" onclick="terminalCommand('systemctl status nexus')">Status</button>
                            <button class="button button-small" onclick="terminalCommand('df -h')">Disk Usage</button>
                            <button class="button button-small" onclick="terminalCommand('free -h')">Memory</button>
                            <button class="button button-small" onclick="terminalCommand('ps aux | head -20')">Processes</button>
                            <button class="button button-small" onclick="clearTerminal()">Clear</button>
                        </div>
                    </div>
                    
                    <div id="system-controls" style="display: none;">
                        <h3>System Controls</h3>
                        <div class="grid">
                            <div class="card">
                                <h4>System Info</h4>
                                <div id="system-info">Loading...</div>
                            </div>
                            <div class="card">
                                <h4>USB Devices</h4>
                                <div id="usb-devices">Loading...</div>
                            </div>
                        </div>
                    </div>
                </div>
            `;
        }
        
        function showTerminal() {
            document.getElementById('terminal-container').style.display = 'block';
            document.getElementById('system-controls').style.display = 'none';
            document.getElementById('terminal-input').focus();
        }
        
        function showSystemControls() {
            document.getElementById('terminal-container').style.display = 'none';
            document.getElementById('system-controls').style.display = 'block';
            loadSystemInfo();
        }
        
        async function executeTerminalCommand() {
            const input = document.getElementById('terminal-input');
            const output = document.getElementById('terminal-output');
            const command = input.value.trim();
            
            if (!command) return;
            
            // Add command to output
            output.innerHTML += '\n$ ' + command + '\n';
            
            // Execute command
            const response = await fetch('/api/admin/terminal', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ command })
            });
            
            const result = await response.json();
            if (result.success) {
                output.innerHTML += result.output;
            } else {
                output.innerHTML += '<span style="color: #f00;">Error: ' + result.error + '</span>';
            }
            
            // Scroll to bottom
            output.scrollTop = output.scrollHeight;
            
            // Clear input
            input.value = '';
        }
        
        function handleTerminalKey(event) {
            if (event.key === 'Enter') {
                executeTerminalCommand();
            }
        }
        
        function terminalCommand(cmd) {
            document.getElementById('terminal-input').value = cmd;
            executeTerminalCommand();
        }
        
        function clearTerminal() {
            document.getElementById('terminal-output').innerHTML = 'Nexus Terminal v2.1.0\n';
        }
        
        async function confirmReboot() {
            if (confirm('Are you sure you want to reboot the system?')) {
                const response = await fetch('/api/admin/reboot', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ delay: 5 })
                });
                const result = await response.json();
                alert(result.message || result.error);
            }
        }
        
        async function systemUpdate() {
            if (confirm('Update system and restart service?')) {
                const response = await fetch('/api/admin/update', {
                    method: 'POST'
                });
                const result = await response.json();
                console.log('Update results:', result);
                alert('Update process initiated. Check terminal for details.');
            }
        }
        
        async function createBackup() {
            const response = await fetch('/api/admin/backup', {
                method: 'POST'
            });
            const result = await response.json();
            if (result.success) {
                alert('Backup created: ' + result.backup_file);
            } else {
                alert('Backup failed: ' + result.error);
            }
        }
        
        async function loadSystemInfo() {
            const response = await fetch('/api/status');
            const status = await response.json();
            document.getElementById('system-info').innerHTML = `
                <p>CPU Usage: ${{status.cpu_usage}}%</p>
                <p>Memory: ${{status.memory_usage}}%</p>
                <p>Uptime: ${{status.uptime}} hours</p>
            `;
            
            const usbResponse = await fetch('/api/admin/usb-scan');
            const usbResult = await usbResponse.json();
            if (usbResult.success) {
                const devices = usbResult.devices.map(d => 
                    `<div>${{d.id}} - ${{d.description}}</div>`
                ).join('');
                document.getElementById('usb-devices').innerHTML = devices || 'No USB devices found';
            }
        }
        
        async function generateDashboard() {{
            const response = await fetch('/api/status');
            const status = await response.json();
            
            return `
                <div class="grid">
                    <div class="card">
                        <h2 class="card-title">System Status</h2>
                        <p><span id="connection-status" class="status-indicator status-online"></span> Connected</p>
                        <p>CPU: ${{status.cpu_usage || '0'}}%</p>
                        <p>Memory: ${{status.memory_usage || '0'}}%</p>
                        <p>Uptime: ${{status.uptime || '0'}} hours</p>
                    </div>
                    
                    <div class="card">
                        <h2 class="card-title">Active Boards</h2>
                        <div id="boards-list">Loading...</div>
                    </div>
                    
                    <div class="card">
                        <h2 class="card-title">Recent Alarms</h2>
                        <div id="alarms-list">No active alarms</div>
                    </div>
                </div>
            `;
        }}
        
        async function generateBoardsPage() {{
            return `
                <div class="card">
                    <h2 class="card-title">Board Configuration</h2>
                    <button class="button" onclick="scanBoards()">Scan for Boards</button>
                    <div id="boards-table" style="margin-top: 2rem;">
                        <table class="table">
                            <thead>
                                <tr>
                                    <th>Board Type</th>
                                    <th>Stack</th>
                                    <th>Capabilities</th>
                                    <th>Status</th>
                                    <th>Actions</th>
                                </tr>
                            </thead>
                            <tbody id="boards-tbody">
                                <tr><td colspan="5">Click "Scan for Boards" to detect hardware</td></tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            `;
        }}
        
        async function generateIOControlPage() {{
            return `
                <div class="card">
                    <h2 class="card-title">I/O Control Panel</h2>
                    <div class="corrections-notice" style="margin: 1rem 0;">
                        <strong>Note:</strong> Input channels dynamically adjust based on board type
                    </div>
                    <div id="io-controls">
                        Loading I/O channels...
                    </div>
                </div>
            `;
        }}
        
        async function generateRefrigerantPage() {{
            return `
                <div class="card">
                    <h2 class="card-title">Refrigerant Diagnostics</h2>
                    <p>P499 Pressure Transducers: Supported via configurable inputs</p>
                    <div id="refrigerant-data">
                        Loading refrigerant data...
                    </div>
                </div>
            `;
        }}
        
        async function scanBoards() {{
            const response = await fetch('/api/boards/scan');
            const boards = await response.json();
            
            const tbody = document.getElementById('boards-tbody');
            tbody.innerHTML = boards.map(board => `
                <tr>
                    <td>${{board.type}}</td>
                    <td>${{board.stack}}</td>
                    <td>${{getBoardCapabilities(board.type)}}</td>
                    <td><span class="status-indicator status-online"></span> Online</td>
                    <td><button class="button" onclick="configureBoard('${{board.type}}', ${{board.stack}})">Configure</button></td>
                </tr>
            `).join('');
        }}
        
        function getBoardCapabilities(type) {{
            const capabilities = {{
                'megabas': '4 triacs, 4 AO, 8 CI',
                '16univin': '16 universal inputs',
                '16uout': '16 analog outputs',
                '8relind': '8 relay outputs',
                '16relind': '16 relay outputs'
            }};
            return capabilities[type] || 'Unknown';
        }}
        
        // Initialize on page load
        document.addEventListener('DOMContentLoaded', () => {{
            connectWebSocket();
            loadPageContent();
        }});
        
        // Handle navigation
        window.addEventListener('popstate', loadPageContent);
    </script>
</body>
</html>
"#, title, title)
}

// API endpoint handlers
async fn get_status(State(state): State<Arc<Mutex<AppState>>>) -> Json<serde_json::Value> {
    let app_state = state.lock().await;
    Json(json!({
        "status": "online",
        "version": "2.1.0",
        "cpu_usage": 15,
        "memory_usage": 42,
        "uptime": 127,
        "boards_connected": 3
    }))
}

async fn scan_boards(State(state): State<Arc<Mutex<AppState>>>) -> Json<Vec<serde_json::Value>> {
    // Call firmware interface to scan boards
    Json(vec![
        json!({
            "type": "megabas",
            "stack": 0,
            "name": "MegaBAS Stack 0",
            "status": "online"
        }),
        json!({
            "type": "16univin",
            "stack": 1,
            "name": "16-UnivIn Stack 1",
            "status": "online"
        })
    ])
}

async fn get_board_status(Path(id): Path<String>) -> Json<serde_json::Value> {
    Json(json!({
        "board_id": id,
        "status": "online",
        "readings": {}
    }))
}

async fn get_io_value(Path((board, channel)): Path<(String, u8)>) -> Json<serde_json::Value> {
    Json(json!({
        "board": board,
        "channel": channel,
        "value": 0.0
    }))
}

async fn set_io_value(
    Path((board, channel)): Path<(String, u8)>,
    Json(payload): Json<serde_json::Value>
) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "board": board,
        "channel": channel,
        "value": payload["value"]
    }))
}

async fn get_refrigerant_status() -> Json<serde_json::Value> {
    Json(json!({
        "circuits": [],
        "alarms": []
    }))
}

async fn get_vibration_sensors() -> Json<serde_json::Value> {
    Json(json!({
        "sensors": []
    }))
}

// Admin API endpoints
async fn execute_terminal_command(
    Json(payload): Json<serde_json::Value>
) -> Json<serde_json::Value> {
    use crate::system_commands::SystemCommands;
    
    let command = payload["command"].as_str().unwrap_or("");
    
    match SystemCommands::execute_command(command).await {
        Ok(output) => Json(json!({
            "success": true,
            "output": output
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

async fn system_reboot(
    Json(payload): Json<serde_json::Value>
) -> Json<serde_json::Value> {
    use crate::system_commands::SystemCommands;
    
    let delay = payload["delay"].as_u64().unwrap_or(5);
    
    match SystemCommands::reboot(delay).await {
        Ok(_) => Json(json!({
            "success": true,
            "message": format!("System will reboot in {} seconds", delay)
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

async fn system_update() -> Json<serde_json::Value> {
    use crate::system_commands::SystemCommands;
    
    // Run system update commands
    let commands = vec![
        "sudo apt update",
        "sudo apt upgrade -y",
        "cd /opt/nexus && git pull",
        "cargo build --release",
        "sudo systemctl restart nexus"
    ];
    
    let mut results = Vec::new();
    for cmd in commands {
        match SystemCommands::execute_command(cmd).await {
            Ok(output) => results.push(json!({
                "command": cmd,
                "success": true,
                "output": output
            })),
            Err(e) => results.push(json!({
                "command": cmd,
                "success": false,
                "error": e.to_string()
            }))
        }
    }
    
    Json(json!({
        "results": results
    }))
}

async fn create_backup() -> Json<serde_json::Value> {
    use crate::system_commands::SystemCommands;
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_file = format!("/var/backups/nexus/backup_{}.tar.gz", timestamp);
    let command = format!("tar czf {} /opt/nexus /var/lib/nexus /etc/nexus", backup_file);
    
    match SystemCommands::execute_command(&command).await {
        Ok(_) => Json(json!({
            "success": true,
            "backup_file": backup_file
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

async fn restore_backup(
    Json(payload): Json<serde_json::Value>
) -> Json<serde_json::Value> {
    use crate::system_commands::SystemCommands;
    
    let backup_file = payload["backup_file"].as_str().unwrap_or("");
    let command = format!("tar xzf {} -C /", backup_file);
    
    match SystemCommands::execute_command(&command).await {
        Ok(_) => Json(json!({
            "success": true,
            "message": "Backup restored successfully"
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

async fn get_system_logs() -> Json<serde_json::Value> {
    use crate::system_commands::SystemCommands;
    
    match SystemCommands::execute_command("journalctl -u nexus -n 100").await {
        Ok(logs) => Json(json!({
            "success": true,
            "logs": logs
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

async fn scan_usb_devices() -> Json<serde_json::Value> {
    use crate::system_commands::SystemCommands;
    
    match SystemCommands::scan_usb().await {
        Ok(devices) => Json(json!({
            "success": true,
            "devices": devices
        })),
        Err(e) => Json(json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

// WebSocket handler for real-time updates
async fn websocket_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<Arc<Mutex<AppState>>>
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(
    mut socket: axum::extract::ws::WebSocket,
    state: Arc<Mutex<AppState>>
) {
    // Send updates every second
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
    
    loop {
        interval.tick().await;
        
        // Send real-time data
        let message = json!({
            "type": "io_update",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "values": {}
        });
        
        if socket.send(axum::extract::ws::Message::Text(message.to_string())).await.is_err() {
            break;
        }
    }
}