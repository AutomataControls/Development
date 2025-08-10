// System Commands - Reboot, USB scan, terminal commands, watchdog
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub uptime: String,
    pub load_average: String,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub cpu_temp: f32,
    pub processes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct USBDevice {
    pub bus: String,
    pub device: String,
    pub id: String,
    pub description: String,
}

pub struct SystemCommands;

impl SystemCommands {
    // System reboot with delay
    pub async fn reboot(delay_seconds: u64) -> Result<()> {
        if delay_seconds > 0 {
            tokio::time::sleep(tokio::time::Duration::from_secs(delay_seconds)).await;
        }
        
        Command::new("sudo")
            .args(&["shutdown", "-r", "now"])
            .spawn()?;
        
        Ok(())
    }
    
    // Get system status
    pub async fn get_status() -> Result<SystemStatus> {
        // Uptime
        let uptime_output = Command::new("uptime").output()?;
        let uptime = String::from_utf8_lossy(&uptime_output.stdout).trim().to_string();
        
        // Load average
        let loadavg = fs::read_to_string("/proc/loadavg")?;
        let load_average = loadavg.split_whitespace().take(3).collect::<Vec<_>>().join(" ");
        
        // Memory
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        let mut mem_total = 0u64;
        let mut mem_available = 0u64;
        
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                mem_total = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                mem_available = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            }
        }
        
        let memory_used = mem_total - mem_available;
        
        // Disk usage
        let df_output = Command::new("df")
            .args(&["-B1", "/"])
            .output()?;
        let df_str = String::from_utf8_lossy(&df_output.stdout);
        let df_lines: Vec<&str> = df_str.lines().collect();
        
        let mut disk_total = 0u64;
        let mut disk_used = 0u64;
        
        if df_lines.len() > 1 {
            let parts: Vec<&str> = df_lines[1].split_whitespace().collect();
            if parts.len() >= 3 {
                disk_total = parts[1].parse().unwrap_or(0);
                disk_used = parts[2].parse().unwrap_or(0);
            }
        }
        
        // CPU temperature (Raspberry Pi specific)
        let cpu_temp = if Path::new("/sys/class/thermal/thermal_zone0/temp").exists() {
            let temp_str = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")?;
            temp_str.trim().parse::<f32>().unwrap_or(0.0) / 1000.0
        } else {
            0.0
        };
        
        // Process count
        let ps_output = Command::new("ps")
            .args(&["aux"])
            .output()?;
        let processes = String::from_utf8_lossy(&ps_output.stdout).lines().count() as u32 - 1;
        
        Ok(SystemStatus {
            uptime,
            load_average,
            memory_used,
            memory_total: mem_total,
            disk_used,
            disk_total,
            cpu_temp,
            processes,
        })
    }
    
    // Scan USB devices
    pub async fn scan_usb() -> Result<Vec<USBDevice>> {
        let output = Command::new("lsusb").output()?;
        let usb_list = String::from_utf8_lossy(&output.stdout);
        
        let mut devices = Vec::new();
        
        for line in usb_list.lines() {
            // Parse lsusb output: Bus 001 Device 001: ID 1d6b:0002 Linux Foundation 2.0 root hub
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 7 {
                let bus = parts[1].to_string();
                let device = parts[3].trim_end_matches(':').to_string();
                let id = parts[5].to_string();
                let description = parts[6..].join(" ");
                
                devices.push(USBDevice {
                    bus,
                    device,
                    id,
                    description,
                });
            }
        }
        
        Ok(devices)
    }
    
    // Execute shell command (with restrictions)
    pub async fn execute_command(command: &str) -> Result<String> {
        // Whitelist safe commands
        let allowed_commands = vec![
            "ls", "pwd", "date", "uptime", "df", "free", "ps", 
            "systemctl", "journalctl", "tail", "head", "cat", "grep"
        ];
        
        let cmd_parts: Vec<&str> = command.split_whitespace().collect();
        if cmd_parts.is_empty() {
            return Err(anyhow!("Empty command"));
        }
        
        let base_cmd = cmd_parts[0];
        if !allowed_commands.contains(&base_cmd) {
            return Err(anyhow!("Command not allowed: {}", base_cmd));
        }
        
        let output = Command::new("bash")
            .arg("-c")
            .arg(command)
            .output()?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!("Command failed: {}", 
                String::from_utf8_lossy(&output.stderr)))
        }
    }
    
    // Watchdog management
    pub async fn configure_watchdog(enabled: bool, timeout_seconds: u32) -> Result<()> {
        if enabled {
            // Enable hardware watchdog
            Command::new("sudo")
                .args(&["modprobe", "bcm2835_wdt"])
                .output()?;
            
            // Configure timeout
            fs::write("/dev/watchdog", format!("V{}", timeout_seconds))?;
        } else {
            // Disable watchdog
            fs::write("/dev/watchdog", "V")?;
        }
        
        Ok(())
    }
    
    // Kick watchdog (keep system alive)
    pub async fn kick_watchdog() -> Result<()> {
        if Path::new("/dev/watchdog").exists() {
            fs::write("/dev/watchdog", ".")?;
        }
        Ok(())
    }
}