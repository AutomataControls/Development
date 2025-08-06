#!/usr/bin/env node

const express = require('express');
const cors = require('cors');
const { exec, spawn } = require('child_process');
const fs = require('fs').promises;
const path = require('path');
const sqlite3 = require('sqlite3').verbose();
const { open } = require('sqlite');

const app = express();
const PORT = process.env.PORT || 3001;

// Middleware
app.use(cors());
app.use(express.json());

// Database connection
let db;

async function initDatabase() {
    db = await open({
        filename: path.join(process.env.HOME || '/home/Automata', '.automata-nexus', 'metrics.db'),
        driver: sqlite3.Database
    });
    console.log('Connected to metrics database');
}

// Hardware interfaces
const hardwareInterfaces = {
    megabas: { available: false, error: null },
    univin: { available: false, error: null },
    relind16: { available: false, error: null },
    relind8: { available: false, error: null },
    uout: { available: false, error: null }
};

// Check hardware availability
async function checkHardware() {
    const commands = {
        megabas: 'megabas 0 board',
        univin: '16univin 0 board',
        relind16: '16relind 0 board',
        relind8: '8relind 0 board',
        uout: '16uout 0 board'
    };

    for (const [key, cmd] of Object.entries(commands)) {
        try {
            await new Promise((resolve, reject) => {
                exec(cmd, (error, stdout, stderr) => {
                    if (error) {
                        hardwareInterfaces[key].available = false;
                        hardwareInterfaces[key].error = error.message;
                        reject(error);
                    } else {
                        hardwareInterfaces[key].available = true;
                        hardwareInterfaces[key].error = null;
                        resolve(stdout);
                    }
                });
            });
        } catch (e) {
            console.log(`Hardware ${key} not available:`, e.message);
        }
    }
}

// API Routes

// System info
app.get('/api/system-info', async (req, res) => {
    try {
        const systemInfo = {
            platform: process.platform,
            arch: process.arch,
            uptime: process.uptime(),
            memory: process.memoryUsage(),
            hardwareInterfaces
        };
        res.json(systemInfo);
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Boards detection
app.get('/api/boards', async (req, res) => {
    const boards = [];
    
    // Check Megabas boards
    for (let i = 0; i < 8; i++) {
        try {
            await new Promise((resolve, reject) => {
                exec(`megabas ${i} board`, (error) => {
                    if (!error) {
                        boards.push({
                            type: 'megabas',
                            address: i,
                            name: `Building Automation Board ${i}`
                        });
                    }
                    resolve();
                });
            });
        } catch (e) {}
    }
    
    // Check 16 Universal Input boards
    for (let i = 0; i < 8; i++) {
        try {
            await new Promise((resolve, reject) => {
                exec(`16univin ${i} board`, (error) => {
                    if (!error) {
                        boards.push({
                            type: '16univin',
                            address: i,
                            name: `16 Universal Input Board ${i}`
                        });
                    }
                    resolve();
                });
            });
        } catch (e) {}
    }
    
    res.json(boards);
});

// Read inputs
app.get('/api/inputs/:board/:channel', async (req, res) => {
    const { board, channel } = req.params;
    
    try {
        const result = await new Promise((resolve, reject) => {
            exec(`megabas ${board} aread ${channel}`, (error, stdout, stderr) => {
                if (error) reject(error);
                else resolve(parseFloat(stdout.trim()));
            });
        });
        
        res.json({ value: result });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Write outputs
app.post('/api/outputs/:board/:channel', async (req, res) => {
    const { board, channel } = req.params;
    const { value } = req.body;
    
    try {
        await new Promise((resolve, reject) => {
            exec(`megabas ${board} awrite ${channel} ${value}`, (error, stdout, stderr) => {
                if (error) reject(error);
                else resolve(stdout);
            });
        });
        
        res.json({ success: true });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Relay control
app.post('/api/relays/:board/:relay', async (req, res) => {
    const { board, relay } = req.params;
    const { state } = req.body;
    
    try {
        const cmd = state ? 'on' : 'off';
        await new Promise((resolve, reject) => {
            exec(`megabas ${board} rwrite ${relay} ${cmd}`, (error, stdout, stderr) => {
                if (error) reject(error);
                else resolve(stdout);
            });
        });
        
        res.json({ success: true });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Metrics
app.get('/api/metrics/:name', async (req, res) => {
    const { name } = req.params;
    const { hours = 24 } = req.query;
    
    try {
        const rows = await db.all(
            `SELECT timestamp, value FROM metrics 
             WHERE name = ? AND timestamp > datetime('now', '-${hours} hours')
             ORDER BY timestamp DESC`,
            [name]
        );
        
        res.json(rows);
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

app.post('/api/metrics', async (req, res) => {
    const { name, value } = req.body;
    
    try {
        await db.run(
            'INSERT INTO metrics (name, value, timestamp) VALUES (?, ?, datetime("now"))',
            [name, value]
        );
        
        res.json({ success: true });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Get all point mappings
app.get('/api/points', async (req, res) => {
    try {
        const configPath = path.join(process.env.HOME || '/home/Automata', '.automata-nexus', 'point-config.json');
        const data = await fs.readFile(configPath, 'utf8');
        res.json(JSON.parse(data));
    } catch (error) {
        res.json({ points: [] });
    }
});

// Save point mappings
app.post('/api/points', async (req, res) => {
    try {
        const configPath = path.join(process.env.HOME || '/home/Automata', '.automata-nexus', 'point-config.json');
        await fs.mkdir(path.dirname(configPath), { recursive: true });
        await fs.writeFile(configPath, JSON.stringify(req.body, null, 2));
        res.json({ success: true });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Logic files
app.get('/api/logic-files', async (req, res) => {
    try {
        const logicDir = path.join(process.env.HOME || '/home/Automata', '.automata-nexus', 'logic');
        await fs.mkdir(logicDir, { recursive: true });
        const files = await fs.readdir(logicDir);
        const logicFiles = files.filter(f => f.endsWith('.js')).map(f => ({
            name: f,
            path: path.join(logicDir, f)
        }));
        res.json(logicFiles);
    } catch (error) {
        res.json([]);
    }
});

// BMS Integration
app.get('/api/bms/config', async (req, res) => {
    try {
        const configPath = path.join(process.env.HOME || '/home/Automata', '.automata-nexus', 'bms-config.json');
        const data = await fs.readFile(configPath, 'utf8');
        res.json(JSON.parse(data));
    } catch (error) {
        res.json({ influxdb_url: '', influxdb_token: '', influxdb_org: '', influxdb_bucket: '' });
    }
});

app.post('/api/bms/config', async (req, res) => {
    try {
        const configPath = path.join(process.env.HOME || '/home/Automata', '.automata-nexus', 'bms-config.json');
        await fs.mkdir(path.dirname(configPath), { recursive: true });
        await fs.writeFile(configPath, JSON.stringify(req.body, null, 2));
        res.json({ success: true });
    } catch (error) {
        res.status(500).json({ error: error.message });
    }
});

// Start server
async function start() {
    await initDatabase();
    await checkHardware();
    
    app.listen(PORT, '0.0.0.0', () => {
        console.log(`Backend API server running on port ${PORT}`);
        console.log('Hardware status:', hardwareInterfaces);
    });
}

start().catch(console.error);