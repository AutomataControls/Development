// Firmware Manager - Manages Sequent Microsystems board firmware
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareInfo {
    pub board_name: String,
    pub repo_url: String,
    pub installed_version: Option<String>,
    pub latest_version: Option<String>,
    pub needs_update: bool,
}

pub struct FirmwareManager {
    firmware_path: String,
}

impl FirmwareManager {
    pub fn new() -> Self {
        Self {
            firmware_path: "/opt/nexus/firmware".to_string(),
        }
    }
    
    // Get all Sequent Microsystems repos
    fn get_board_repos() -> Vec<(&'static str, &'static str)> {
        vec![
            ("megabas-rpi", "https://github.com/SequentMicrosystems/megabas-rpi.git"),
            ("16relind-rpi", "https://github.com/SequentMicrosystems/16relind-rpi.git"),
            ("8relind-rpi", "https://github.com/SequentMicrosystems/8relind-rpi.git"),
            ("16inpind-rpi", "https://github.com/SequentMicrosystems/16inpind-rpi.git"),
            ("16univin-rpi", "https://github.com/SequentMicrosystems/16univin-rpi.git"),
            ("16uout-rpi", "https://github.com/SequentMicrosystems/16uout-rpi.git"),
            ("4-20mA-rpi", "https://github.com/SequentMicrosystems/4-20mA-rpi.git"),
            ("rtd-rpi", "https://github.com/SequentMicrosystems/rtd-rpi.git"),
            ("thermo-rpi", "https://github.com/SequentMicrosystems/thermo-rpi.git"),
        ]
    }
    
    // Clone or update all repos
    pub async fn update_all_repos() -> Result<Vec<FirmwareInfo>> {
        let mut firmware_list = Vec::new();
        
        for (board_name, repo_url) in Self::get_board_repos() {
            let repo_path = format!("/opt/nexus/firmware/{}", board_name);
            
            if Path::new(&repo_path).exists() {
                // Pull latest
                Command::new("git")
                    .args(&["pull"])
                    .current_dir(&repo_path)
                    .output()?;
            } else {
                // Clone repo
                Command::new("git")
                    .args(&["clone", repo_url, &repo_path])
                    .output()?;
            }
            
            // Get version info
            let version_output = Command::new("git")
                .args(&["describe", "--tags", "--always"])
                .current_dir(&repo_path)
                .output()?;
            
            let version = String::from_utf8_lossy(&version_output.stdout).trim().to_string();
            
            firmware_list.push(FirmwareInfo {
                board_name: board_name.to_string(),
                repo_url: repo_url.to_string(),
                installed_version: Some(version.clone()),
                latest_version: Some(version),
                needs_update: false,
            });
        }
        
        Ok(firmware_list)
    }
    
    // Install board firmware
    pub async fn install_board(board_name: &str) -> Result<()> {
        let repo_path = format!("/opt/nexus/firmware/{}", board_name);
        
        if !Path::new(&repo_path).exists() {
            return Err(anyhow!("Board firmware not found: {}", board_name));
        }
        
        // Check for install script
        let install_script = format!("{}/install.sh", repo_path);
        let make_path = format!("{}/Makefile", repo_path);
        let python_setup = format!("{}/setup.py", repo_path);
        
        if Path::new(&install_script).exists() {
            // Run install script
            Command::new("sudo")
                .args(&["bash", &install_script])
                .current_dir(&repo_path)
                .status()?;
        } else if Path::new(&make_path).exists() {
            // Run make install
            Command::new("make")
                .current_dir(&repo_path)
                .status()?;
            
            Command::new("sudo")
                .args(&["make", "install"])
                .current_dir(&repo_path)
                .status()?;
        } else if Path::new(&python_setup).exists() {
            // Python setup
            Command::new("sudo")
                .args(&["python3", "setup.py", "install"])
                .current_dir(&repo_path)
                .status()?;
        } else {
            return Err(anyhow!("No installation method found for {}", board_name));
        }
        
        Ok(())
    }
    
    // Get firmware status
    pub async fn get_status() -> Result<Vec<FirmwareInfo>> {
        let mut firmware_list = Vec::new();
        
        for (board_name, repo_url) in Self::get_board_repos() {
            let repo_path = format!("/opt/nexus/firmware/{}", board_name);
            
            if Path::new(&repo_path).exists() {
                // Get current version
                let current_output = Command::new("git")
                    .args(&["describe", "--tags", "--always"])
                    .current_dir(&repo_path)
                    .output()?;
                
                let current = String::from_utf8_lossy(&current_output.stdout).trim().to_string();
                
                // Fetch latest
                Command::new("git")
                    .args(&["fetch"])
                    .current_dir(&repo_path)
                    .output()?;
                
                // Get latest version
                let latest_output = Command::new("git")
                    .args(&["describe", "--tags", "--always", "origin/master"])
                    .current_dir(&repo_path)
                    .output()?;
                
                let latest = String::from_utf8_lossy(&latest_output.stdout).trim().to_string();
                
                firmware_list.push(FirmwareInfo {
                    board_name: board_name.to_string(),
                    repo_url: repo_url.to_string(),
                    installed_version: Some(current.clone()),
                    latest_version: Some(latest.clone()),
                    needs_update: current != latest,
                });
            } else {
                firmware_list.push(FirmwareInfo {
                    board_name: board_name.to_string(),
                    repo_url: repo_url.to_string(),
                    installed_version: None,
                    latest_version: None,
                    needs_update: false,
                });
            }
        }
        
        Ok(firmware_list)
    }
}