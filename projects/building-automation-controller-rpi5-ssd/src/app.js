// Tauri API
const { invoke } = window.__TAURI__.tauri;

// Global state
let boards = [];
let boardStatus = {};
let exportConfigs = { bms: null, processing: null };
let refreshInterval = null;

// Initialize
document.addEventListener('DOMContentLoaded', async () => {
    await scanBoards();
    await loadExportConfigs();
    await loadLogicFiles();
    
    // Start auto-refresh
    refreshInterval = setInterval(() => {
        refreshStatus();
        refreshLogicStatus();
    }, 5000);
});

// Tab switching
function switchTab(tabName) {
    // Update tab buttons
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    event.target.classList.add('active');
    
    // Update tab content
    document.querySelectorAll('.tab-content').forEach(content => {
        content.classList.remove('active');
    });
    document.getElementById(`${tabName}-tab`).classList.add('active');
}

// Board scanning
async function scanBoards() {
    try {
        updateConnectionStatus('scanning', 'Scanning...');
        boards = await invoke('scan_boards');
        
        if (boards.length > 0) {
            updateConnectionStatus('connected', `Found ${boards.length} board(s)`);
            await refreshStatus();
        } else {
            updateConnectionStatus('error', 'No boards found');
        }
    } catch (error) {
        console.error('Scan error:', error);
        updateConnectionStatus('error', 'Scan failed');
    }
}

// Refresh board status
async function refreshStatus() {
    try {
        // Get status for all boards
        const statusPromises = boards.map(board => 
            invoke('get_board_status', { 
                boardType: board.board_type, 
                stack: board.stack 
            })
        );
        
        const statuses = await Promise.all(statusPromises);
        
        // Update status object
        boardStatus = {};
        statuses.forEach(status => {
            boardStatus[`${status.board_type}_${status.stack}`] = status.data;
        });
        
        // Update UI
        updateBoardsUI();
        updateSystemStatus();
        
    } catch (error) {
        console.error('Status refresh error:', error);
    }
}

// Update boards UI
function updateBoardsUI() {
    // Update MegaBAS board
    const megabasData = boardStatus['megabas_0'];
    if (megabasData) {
        updateMegabasUI(megabasData);
    }
    
    // Update expansion boards
    updateExpansionBoardsUI();
}

// Update MegaBAS UI
function updateMegabasUI(data) {
    const container = document.getElementById('megabasBoard');
    
    let html = `
        <div class="card">
            <div class="card-header">
                <h3 class="card-title">MegaBAS HAT (Stack 0)</h3>
                <span style="font-size: 12px; color: #666;">Firmware: ${data.firmware || 'Unknown'}</span>
            </div>
            
            <!-- Analog Inputs -->
            <h4 style="margin: 1.5rem 0 1rem; color: #2a5298;">Analog Inputs (0-10V)</h4>
            <div class="io-grid">
    `;
    
    // Add analog inputs
    for (let i = 1; i <= 8; i++) {
        const input = data.analog_inputs?.[`ch${i}`] || {};
        html += `
            <div class="io-item">
                <div class="io-label">Channel ${i}</div>
                <div class="io-value">${input.voltage?.toFixed(2) || '0.00'} V</div>
                <div style="font-size: 12px; color: #666;">
                    R1K: ${input.r1k?.toFixed(1) || '-'} kΩ | 
                    R10K: ${input.r10k?.toFixed(1) || '-'} kΩ
                </div>
            </div>
        `;
    }
    
    html += `
            </div>
            
            <!-- Analog Outputs -->
            <h4 style="margin: 1.5rem 0 1rem; color: #2a5298;">Analog Outputs (0-10V)</h4>
            <div class="io-grid">
    `;
    
    // Add analog outputs
    for (let i = 1; i <= 4; i++) {
        const value = data.analog_outputs?.[`ch${i}`] || 0;
        html += `
            <div class="io-item">
                <div class="io-label">Channel ${i}</div>
                <div class="io-value">${value.toFixed(2)} V</div>
                <div class="io-control">
                    <input type="range" min="0" max="10" step="0.1" 
                           value="${value}" 
                           onchange="setAnalogOutput(0, ${i}, this.value)"
                           id="analog-out-${i}">
                    <input type="number" min="0" max="10" step="0.1" 
                           value="${value}" 
                           style="width: 70px;"
                           onchange="setAnalogOutput(0, ${i}, this.value); 
                                   document.getElementById('analog-out-${i}').value = this.value;">
                </div>
            </div>
        `;
    }
    
    html += `
            </div>
            
            <!-- Triacs -->
            <h4 style="margin: 1.5rem 0 1rem; color: #2a5298;">Triac Outputs</h4>
            <div class="io-grid">
    `;
    
    // Add triacs
    for (let i = 1; i <= 4; i++) {
        const state = data.triacs?.[`ch${i}`] || false;
        html += `
            <div class="io-item">
                <div class="io-label">Triac ${i}</div>
                <div class="toggle-switch ${state ? 'active' : ''}" 
                     onclick="toggleTriac(0, ${i}, ${!state})">
                    <div class="toggle-slider"></div>
                </div>
            </div>
        `;
    }
    
    html += `
            </div>
            
            <!-- Dry Contacts -->
            <h4 style="margin: 1.5rem 0 1rem; color: #2a5298;">Dry Contact Inputs</h4>
            <div class="io-grid">
    `;
    
    // Add dry contacts
    for (let i = 1; i <= 4; i++) {
        const contact = data.contacts?.[`ch${i}`] || {};
        html += `
            <div class="io-item">
                <div class="io-label">Contact ${i}</div>
                <div class="io-value" style="color: ${contact.state ? '#4CAF50' : '#999'};">
                    ${contact.state ? 'CLOSED' : 'OPEN'}
                </div>
                <div style="font-size: 12px; color: #666;">
                    Count: ${contact.counter || 0}
                </div>
            </div>
        `;
    }
    
    html += `
            </div>
            
            <!-- Sensors -->
            <h4 style="margin: 1.5rem 0 1rem; color: #2a5298;">System Sensors</h4>
            <div class="io-grid">
                <div class="io-item">
                    <div class="io-label">Power Supply</div>
                    <div class="io-value">${data.sensors?.power_supply_v?.toFixed(2) || '0.00'} V</div>
                </div>
                <div class="io-item">
                    <div class="io-label">Raspberry Pi</div>
                    <div class="io-value">${data.sensors?.raspberry_v?.toFixed(2) || '0.00'} V</div>
                </div>
                <div class="io-item">
                    <div class="io-label">CPU Temperature</div>
                    <div class="io-value">${data.sensors?.cpu_temp_c?.toFixed(1) || '0.0'} °C</div>
                </div>
            </div>
        </div>
    `;
    
    container.innerHTML = html;
}

// Update expansion boards UI
function updateExpansionBoardsUI() {
    const container = document.getElementById('expansionBoards');
    let html = '';
    
    boards.forEach(board => {
        if (board.board_type === 'megabas') return; // Skip MegaBAS
        
        const key = `${board.board_type}_${board.stack}`;
        const data = boardStatus[key];
        
        if (!data) return;
        
        html += `<div class="board-card">`;
        html += `<div class="board-header">
            <span class="board-type">${getBoardTypeName(board.board_type)}</span>
            <span class="board-stack">Stack ${board.stack}</span>
        </div>`;
        
        // Render based on board type
        if (board.board_type === '16relay' || board.board_type === '8relay') {
            html += renderRelayBoard(board, data);
        } else if (board.board_type === '16univin') {
            html += renderUniversalInputBoard(board, data);
        } else if (board.board_type === '16uout') {
            html += renderAnalogOutputBoard(board, data);
        }
        
        html += `</div>`;
    });
    
    container.innerHTML = html || '<p style="text-align: center; color: #999;">No expansion boards detected</p>';
}

// Render relay board
function renderRelayBoard(board, data) {
    const numRelays = board.board_type === '16relay' ? 16 : 8;
    let html = '<div class="io-grid">';
    
    for (let i = 1; i <= numRelays; i++) {
        const state = data.relays?.[`ch${i}`] || false;
        html += `
            <div class="io-item">
                <div class="io-label">Relay ${i}</div>
                <div class="toggle-switch ${state ? 'active' : ''}" 
                     onclick="toggleRelay('${board.board_type}', ${board.stack}, ${i}, ${!state})">
                    <div class="toggle-slider"></div>
                </div>
            </div>
        `;
    }
    
    html += '</div>';
    return html;
}

// Render universal input board
function renderUniversalInputBoard(board, data) {
    let html = '<div style="max-height: 400px; overflow-y: auto;">';
    html += '<div class="io-grid">';
    
    for (let i = 1; i <= 16; i++) {
        const input = data.inputs?.[`ch${i}`] || {};
        html += `
            <div class="io-item">
                <div class="io-label">Input ${i}</div>
                <div class="io-value">${input.voltage?.toFixed(2) || '0.00'} V</div>
                <div style="font-size: 11px; color: #666;">
                    Dig: ${input.digital ? 'HIGH' : 'LOW'} | 
                    Cnt: ${input.counter || 0}
                </div>
            </div>
        `;
    }
    
    html += '</div></div>';
    return html;
}

// Render analog output board
function renderAnalogOutputBoard(board, data) {
    let html = '<div style="max-height: 400px; overflow-y: auto;">';
    html += '<div class="io-grid">';
    
    for (let i = 1; i <= 16; i++) {
        const value = data.outputs?.[`ch${i}`] || 0;
        html += `
            <div class="io-item">
                <div class="io-label">Output ${i}</div>
                <div class="io-value">${value.toFixed(2)} V</div>
                <div class="io-control">
                    <input type="range" min="0" max="10" step="0.1" 
                           value="${value}" 
                           onchange="setExpansionOutput(${board.stack}, ${i}, this.value)">
                </div>
            </div>
        `;
    }
    
    html += '</div></div>';
    return html;
}

// Board type names
function getBoardTypeName(type) {
    const names = {
        '16relay': '16-Channel Relay',
        '8relay': '8-Channel Relay',
        '16univin': '16 Universal Inputs',
        '16uout': '16 Analog Outputs'
    };
    return names[type] || type;
}

// Control functions
async function setAnalogOutput(stack, channel, value) {
    try {
        await invoke('set_output', {
            boardType: 'megabas-analog',
            stack: stack,
            channel: channel,
            value: parseFloat(value)
        });
        logActivity(`Set analog output ${channel} to ${value}V`);
    } catch (error) {
        console.error('Set analog output error:', error);
        alert('Failed to set analog output: ' + error);
    }
}

async function toggleTriac(stack, channel, state) {
    try {
        await invoke('set_output', {
            boardType: 'megabas-triac',
            stack: stack,
            channel: channel,
            value: state ? 1.0 : 0.0
        });
        logActivity(`Turned triac ${channel} ${state ? 'ON' : 'OFF'}`);
        await refreshStatus();
    } catch (error) {
        console.error('Toggle triac error:', error);
        alert('Failed to toggle triac: ' + error);
    }
}

async function toggleRelay(boardType, stack, channel, state) {
    try {
        await invoke('set_relay', {
            boardType: boardType,
            stack: stack,
            channel: channel,
            state: state
        });
        logActivity(`Turned relay ${channel} on board ${stack} ${state ? 'ON' : 'OFF'}`);
        await refreshStatus();
    } catch (error) {
        console.error('Toggle relay error:', error);
        alert('Failed to toggle relay: ' + error);
    }
}

async function setExpansionOutput(stack, channel, value) {
    try {
        await invoke('set_output', {
            boardType: '16uout',
            stack: stack,
            channel: channel,
            value: parseFloat(value)
        });
        logActivity(`Set expansion output ${channel} on board ${stack} to ${value}V`);
    } catch (error) {
        console.error('Set expansion output error:', error);
        alert('Failed to set output: ' + error);
    }
}

// Export configuration
async function loadExportConfigs() {
    try {
        exportConfigs = await invoke('get_export_configs');
        updateExportStatus();
    } catch (error) {
        console.error('Load export configs error:', error);
    }
}

function updateExportStatus() {
    // Update BMS status
    const bmsStatus = document.getElementById('bmsStatus');
    if (exportConfigs.bms) {
        bmsStatus.innerHTML = `
            <p><strong>Status:</strong> ${exportConfigs.bms.enabled ? 
                '<span style="color: #4CAF50;">Enabled</span>' : 
                '<span style="color: #999;">Disabled</span>'}</p>
            <p><strong>Location:</strong> ${exportConfigs.bms.location_name}</p>
            <p><strong>System:</strong> ${exportConfigs.bms.system_name}</p>
            <p><strong>URL:</strong> ${exportConfigs.bms.url}</p>
        `;
    } else {
        bmsStatus.innerHTML = '<p style="color: #999;">Not configured</p>';
    }
    
    // Update Processing status
    const processingStatus = document.getElementById('processingStatus');
    if (exportConfigs.processing) {
        processingStatus.innerHTML = `
            <p><strong>Status:</strong> ${exportConfigs.processing.enabled ? 
                '<span style="color: #4CAF50;">Enabled</span>' : 
                '<span style="color: #999;">Disabled</span>'}</p>
            <p><strong>Location:</strong> ${exportConfigs.processing.location_name}</p>
            <p><strong>System:</strong> ${exportConfigs.processing.system_name}</p>
            <p><strong>URL:</strong> ${exportConfigs.processing.url}:${exportConfigs.processing.port}</p>
        `;
    } else {
        processingStatus.innerHTML = '<p style="color: #999;">Not configured</p>';
    }
}

// Modal functions
function openBMSModal() {
    const modal = document.getElementById('bmsModal');
    modal.classList.add('active');
    
    // Load existing config
    if (exportConfigs.bms) {
        document.getElementById('bmsEnabled').checked = exportConfigs.bms.enabled;
        document.getElementById('bmsLocationName').value = exportConfigs.bms.location_name;
        document.getElementById('bmsSystemName').value = exportConfigs.bms.system_name;
        document.getElementById('bmsLocationId').value = exportConfigs.bms.location_id;
        document.getElementById('bmsEquipmentId').value = exportConfigs.bms.equipment_id;
        document.getElementById('bmsEquipmentType').value = exportConfigs.bms.equipment_type;
        document.getElementById('bmsZone').value = exportConfigs.bms.zone;
        document.getElementById('bmsUrl').value = exportConfigs.bms.url;
        
        // Load mappings
        const mappingsDiv = document.getElementById('bmsMappings');
        mappingsDiv.innerHTML = '';
        for (const [key, value] of Object.entries(exportConfigs.bms.mappings || {})) {
            addBMSMapping(key, value);
        }
    }
}

function openProcessingModal() {
    const modal = document.getElementById('processingModal');
    modal.classList.add('active');
    
    // Load existing config
    if (exportConfigs.processing) {
        document.getElementById('processingEnabled').checked = exportConfigs.processing.enabled;
        document.getElementById('processingLocationName').value = exportConfigs.processing.location_name;
        document.getElementById('processingSystemName').value = exportConfigs.processing.system_name;
        document.getElementById('processingLocationId').value = exportConfigs.processing.location_id;
        document.getElementById('processingEquipmentId').value = exportConfigs.processing.equipment_id;
        document.getElementById('processingEquipmentType').value = exportConfigs.processing.equipment_type;
        document.getElementById('processingZone').value = exportConfigs.processing.zone;
        document.getElementById('processingUrl').value = exportConfigs.processing.url;
        document.getElementById('processingPort').value = exportConfigs.processing.port;
        
        // Load mappings
        const mappingsDiv = document.getElementById('processingMappings');
        mappingsDiv.innerHTML = '';
        for (const [key, value] of Object.entries(exportConfigs.processing.mappings || {})) {
            addProcessingMapping(key, value);
        }
    }
}

function closeModal(modalId) {
    document.getElementById(modalId).classList.remove('active');
}

// Mapping functions
function addBMSMapping(key = '', value = '') {
    const mappingsDiv = document.getElementById('bmsMappings');
    const row = document.createElement('div');
    row.className = 'mapping-row';
    row.innerHTML = `
        <input type="text" placeholder="Export Name" value="${key}">
        <input type="text" placeholder="Source Value" value="${value}">
        <button class="btn btn-danger" onclick="this.parentElement.remove()">Remove</button>
    `;
    mappingsDiv.appendChild(row);
}

function addProcessingMapping(key = '', value = '') {
    const mappingsDiv = document.getElementById('processingMappings');
    const row = document.createElement('div');
    row.className = 'mapping-row';
    row.innerHTML = `
        <input type="text" placeholder="Export Name" value="${key}">
        <input type="text" placeholder="Source Value" value="${value}">
        <button class="btn btn-danger" onclick="this.parentElement.remove()">Remove</button>
    `;
    mappingsDiv.appendChild(row);
}

// Save configurations
async function saveBMSConfig() {
    try {
        // Collect mappings
        const mappings = {};
        document.querySelectorAll('#bmsMappings .mapping-row').forEach(row => {
            const inputs = row.querySelectorAll('input');
            if (inputs[0].value && inputs[1].value) {
                mappings[inputs[0].value] = inputs[1].value;
            }
        });
        
        const config = {
            enabled: document.getElementById('bmsEnabled').checked,
            location_name: document.getElementById('bmsLocationName').value,
            system_name: document.getElementById('bmsSystemName').value,
            location_id: document.getElementById('bmsLocationId').value,
            equipment_id: document.getElementById('bmsEquipmentId').value,
            equipment_type: document.getElementById('bmsEquipmentType').value,
            zone: document.getElementById('bmsZone').value,
            url: document.getElementById('bmsUrl').value,
            mappings: mappings
        };
        
        await invoke('save_bms_config', { config });
        await loadExportConfigs();
        closeModal('bmsModal');
        alert('BMS configuration saved successfully');
    } catch (error) {
        console.error('Save BMS config error:', error);
        alert('Failed to save configuration: ' + error);
    }
}

async function saveProcessingConfig() {
    try {
        // Collect mappings
        const mappings = {};
        document.querySelectorAll('#processingMappings .mapping-row').forEach(row => {
            const inputs = row.querySelectorAll('input');
            if (inputs[0].value && inputs[1].value) {
                mappings[inputs[0].value] = inputs[1].value;
            }
        });
        
        const config = {
            enabled: document.getElementById('processingEnabled').checked,
            location_name: document.getElementById('processingLocationName').value,
            system_name: document.getElementById('processingSystemName').value,
            location_id: document.getElementById('processingLocationId').value,
            equipment_id: document.getElementById('processingEquipmentId').value,
            equipment_type: document.getElementById('processingEquipmentType').value,
            zone: document.getElementById('processingZone').value,
            url: document.getElementById('processingUrl').value,
            port: parseInt(document.getElementById('processingPort').value),
            mappings: mappings
        };
        
        await invoke('save_processing_config', { config });
        await loadExportConfigs();
        closeModal('processingModal');
        alert('Processing configuration saved successfully');
    } catch (error) {
        console.error('Save processing config error:', error);
        alert('Failed to save configuration: ' + error);
    }
}

// Test connections
async function testBMSConnection() {
    try {
        const result = await invoke('test_connection', { endpointType: 'bms' });
        alert(result);
    } catch (error) {
        alert('Connection test failed: ' + error);
    }
}

async function testProcessingConnection() {
    try {
        const result = await invoke('test_connection', { endpointType: 'processing' });
        alert(result);
    } catch (error) {
        alert('Connection test failed: ' + error);
    }
}

// Status updates
function updateConnectionStatus(status, text) {
    const dot = document.getElementById('connectionStatus');
    const textEl = document.getElementById('connectionText');
    
    dot.className = 'status-dot';
    if (status === 'error') {
        dot.classList.add('error');
    }
    
    textEl.textContent = text;
}

function updateSystemStatus() {
    const container = document.getElementById('systemStatus');
    const megabas = boardStatus['megabas_0'];
    
    if (!megabas) {
        container.innerHTML = '<p style="color: #999;">No system data available</p>';
        return;
    }
    
    container.innerHTML = `
        <div class="io-grid">
            <div class="io-item">
                <div class="io-label">Boards Connected</div>
                <div class="io-value">${boards.length}</div>
            </div>
            <div class="io-item">
                <div class="io-label">Power Supply</div>
                <div class="io-value">${megabas.sensors?.power_supply_v?.toFixed(2) || '0.00'} V</div>
            </div>
            <div class="io-item">
                <div class="io-label">CPU Temperature</div>
                <div class="io-value">${megabas.sensors?.cpu_temp_c?.toFixed(1) || '0.0'} °C</div>
            </div>
            <div class="io-item">
                <div class="io-label">Watchdog Resets</div>
                <div class="io-value">${megabas.watchdog?.reset_count || 0}</div>
            </div>
        </div>
    `;
}

// Activity logging
const activityLog = [];

function logActivity(message) {
    const timestamp = new Date().toLocaleTimeString();
    activityLog.unshift({ timestamp, message });
    
    // Keep only last 50 entries
    if (activityLog.length > 50) {
        activityLog.pop();
    }
    
    updateActivityLog();
}

function updateActivityLog() {
    const container = document.getElementById('activityLog');
    
    if (activityLog.length === 0) {
        container.innerHTML = '<p style="color: #999;">No recent activity</p>';
        return;
    }
    
    const html = activityLog.slice(0, 10).map(entry => `
        <div style="padding: 0.5rem 0; border-bottom: 1px solid rgba(0,0,0,0.05);">
            <span style="color: #666; font-size: 12px;">${entry.timestamp}</span>
            <span style="margin-left: 1rem;">${entry.message}</span>
        </div>
    `).join('');
    
    container.innerHTML = html;
}

// Logic file management
let logicFiles = [];
let controlStatus = [];

async function loadLogicFiles() {
    try {
        logicFiles = await invoke('get_logic_files');
        updateLogicFilesUI();
    } catch (error) {
        console.error('Load logic files error:', error);
    }
}

async function refreshLogicStatus() {
    try {
        controlStatus = await invoke('get_control_status');
        updateControlStatusUI();
    } catch (error) {
        console.error('Refresh logic status error:', error);
    }
}

function updateLogicFilesUI() {
    const container = document.getElementById('logicFilesList');
    
    if (logicFiles.length === 0) {
        container.innerHTML = '<p style="color: #999; padding: 1rem;">No logic files uploaded. Click "Upload Logic File" to add control logic.</p>';
        return;
    }
    
    let html = '<div style="overflow-x: auto;"><table style="width: 100%; border-collapse: collapse;">';
    html += `
        <thead>
            <tr style="border-bottom: 2px solid rgba(0,0,0,0.1);">
                <th style="text-align: left; padding: 0.75rem;">Name</th>
                <th style="text-align: left; padding: 0.75rem;">Equipment</th>
                <th style="text-align: left; padding: 0.75rem;">Type</th>
                <th style="text-align: center; padding: 0.75rem;">Status</th>
                <th style="text-align: center; padding: 0.75rem;">Actions</th>
            </tr>
        </thead>
        <tbody>
    `;
    
    logicFiles.forEach(file => {
        const status = controlStatus.find(s => s.logic_file.id === file.id);
        const isRunning = status?.running || false;
        const lastResult = status?.last_result;
        
        html += `
            <tr style="border-bottom: 1px solid rgba(0,0,0,0.05);">
                <td style="padding: 0.75rem;">
                    <div style="font-weight: 500;">${file.name}</div>
                    <div style="font-size: 12px; color: #666;">${file.description}</div>
                </td>
                <td style="padding: 0.75rem;">${file.equipment_id}</td>
                <td style="padding: 0.75rem;">${file.equipment_type}</td>
                <td style="text-align: center; padding: 0.75rem;">
                    ${file.enabled ? 
                        `<span style="color: #4CAF50;">Enabled</span>` : 
                        `<span style="color: #999;">Disabled</span>`}
                    ${isRunning ? ' <span class="loading"></span>' : ''}
                    ${lastResult && !lastResult.success ? 
                        `<div style="color: #f44336; font-size: 11px;">Error</div>` : ''}
                </td>
                <td style="text-align: center; padding: 0.75rem;">
                    <button class="btn btn-secondary" style="padding: 0.5rem 1rem; margin-right: 0.5rem;"
                            onclick="toggleLogic('${file.id}', ${!file.enabled})">
                        ${file.enabled ? 'Disable' : 'Enable'}
                    </button>
                    <button class="btn btn-danger" style="padding: 0.5rem 1rem;"
                            onclick="removeLogic('${file.id}')">
                        Remove
                    </button>
                </td>
            </tr>
        `;
    });
    
    html += '</tbody></table></div>';
    container.innerHTML = html;
}

function updateControlStatusUI() {
    const container = document.getElementById('controlStatus');
    
    if (controlStatus.length === 0) {
        container.innerHTML = '<p style="color: #999;">No active control loops</p>';
        return;
    }
    
    let html = '<div class="io-grid">';
    
    controlStatus.forEach(status => {
        const lastResult = status.last_result;
        const outputs = lastResult?.outputs || {};
        
        html += `
            <div class="card" style="background: rgba(0,0,0,0.03);">
                <h4 style="margin-bottom: 1rem; color: #2a5298;">${status.logic_file.name}</h4>
                <div style="font-size: 14px;">
                    <p><strong>Equipment:</strong> ${status.logic_file.equipment_id}</p>
                    <p><strong>Interval:</strong> ${status.interval_seconds}s</p>
                    <p><strong>Status:</strong> ${status.running ? 
                        '<span style="color: #4CAF50;">Running</span>' : 
                        '<span style="color: #666;">Waiting</span>'}</p>
                    ${lastResult ? `
                        <p><strong>Last Run:</strong> ${new Date(lastResult.timestamp).toLocaleTimeString()}</p>
                        <p><strong>Result:</strong> ${lastResult.success ? 
                            '<span style="color: #4CAF50;">Success</span>' : 
                            `<span style="color: #f44336;">Failed: ${lastResult.error}</span>`}</p>
                    ` : ''}
                </div>
                ${Object.keys(outputs).length > 0 ? `
                    <div style="margin-top: 1rem; padding-top: 1rem; border-top: 1px solid rgba(0,0,0,0.1);">
                        <strong>Control Outputs:</strong>
                        <div style="margin-top: 0.5rem; font-size: 13px;">
                            ${Object.entries(outputs).map(([key, value]) => `
                                <div style="display: flex; justify-content: space-between; padding: 0.25rem 0;">
                                    <span>${key}:</span>
                                    <span style="font-weight: 500;">${value.toFixed(1)}%</span>
                                </div>
                            `).join('')}
                        </div>
                    </div>
                ` : ''}
            </div>
        `;
    });
    
    html += '</div>';
    container.innerHTML = html;
}

// Logic file upload
function openUploadLogicModal() {
    document.getElementById('logicUploadModal').classList.add('active');
}

function handleFileSelect(event) {
    const file = event.target.files[0];
    if (file) {
        document.getElementById('selectedFileName').textContent = file.name;
        
        const reader = new FileReader();
        reader.onload = (e) => {
            document.getElementById('logicContent').value = e.target.result;
        };
        reader.readAsText(file);
    }
}

async function uploadLogicFile() {
    try {
        const name = document.getElementById('logicFileName').value;
        const content = document.getElementById('logicContent').value;
        const equipmentType = document.getElementById('logicEquipmentType').value;
        const equipmentId = document.getElementById('logicEquipmentId').value;
        const locationId = document.getElementById('logicLocationId').value;
        const description = document.getElementById('logicDescription').value;
        
        if (!name || !content || !equipmentType || !equipmentId || !locationId) {
            alert('Please fill all required fields');
            return;
        }
        
        await invoke('upload_logic_file', {
            name,
            content,
            equipmentType,
            equipmentId,
            locationId,
            description
        });
        
        // Clear form
        document.getElementById('logicUploadForm').reset();
        document.getElementById('selectedFileName').textContent = 'No file selected';
        
        // Close modal and refresh
        closeModal('logicUploadModal');
        await loadLogicFiles();
        
        logActivity(`Uploaded logic file: ${name}`);
        alert('Logic file uploaded successfully');
        
    } catch (error) {
        console.error('Upload logic file error:', error);
        alert('Failed to upload logic file: ' + error);
    }
}

async function toggleLogic(id, enabled) {
    try {
        await invoke('toggle_logic_file', { id, enabled });
        await loadLogicFiles();
        logActivity(`${enabled ? 'Enabled' : 'Disabled'} logic file ${id}`);
    } catch (error) {
        console.error('Toggle logic error:', error);
        alert('Failed to toggle logic: ' + error);
    }
}

async function removeLogic(id) {
    if (!confirm('Are you sure you want to remove this logic file?')) {
        return;
    }
    
    try {
        await invoke('remove_logic_file', { id });
        await loadLogicFiles();
        logActivity(`Removed logic file ${id}`);
    } catch (error) {
        console.error('Remove logic error:', error);
        alert('Failed to remove logic: ' + error);
    }
}