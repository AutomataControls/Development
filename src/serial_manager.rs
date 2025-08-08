// Serial Number Management System for Automata Nexus Controllers

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use chrono::{DateTime, Utc};
use reqwest;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialInfo {
    pub serial: String,
    pub fingerprint: String,
    pub subdomain: String,
    pub full_domain: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerialCache {
    serial: String,
    fingerprint: String,
    created_at: DateTime<Utc>,
    device_info: DeviceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeviceInfo {
    hostname: String,
    platform: String,
    system: String,
}

pub struct SerialManager {
    config: HashMap<String, String>,
    api_token: Option<String>,
    domain: String,
    cache_file: PathBuf,
    device_fingerprint: String,
    registry_url: String,
}

impl SerialManager {
    pub fn new(config: HashMap<String, String>) -> Result<Self> {
        let api_token = Self::get_api_token(&config)?;
        let domain = config.get("CLOUDFLARE_DOMAIN")
            .unwrap_or(&"automatacontrols.com".to_string())
            .clone();
        
        let cache_file = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".automata-nexus")
            .join("serial-cache.json");
        
        // Ensure cache directory exists
        if let Some(parent) = cache_file.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let device_fingerprint = Self::generate_device_fingerprint()?;
        let registry_url = "https://api.automatacontrols.com/serials".to_string();
        
        Ok(Self {
            config,
            api_token,
            domain,
            cache_file,
            device_fingerprint,
            registry_url,
        })
    }
    
    fn get_api_token(config: &HashMap<String, String>) -> Result<Option<String>> {
        // Check for token in config
        if let Some(token) = config.get("CLOUDFLARE_API_TOKEN") {
            if !token.is_empty() {
                return Ok(Some(token.clone()));
            }
        }
        
        // Check for token file
        if let Some(token_path) = config.get("CLOUDFLARE_API_TOKEN_PATH") {
            if Path::new(token_path).exists() {
                let token = fs::read_to_string(token_path)?.trim().to_string();
                if !token.is_empty() {
                    return Ok(Some(token));
                }
            }
        }
        
        Ok(None)
    }
    
    fn generate_device_fingerprint() -> Result<String> {
        let mut components = Vec::new();
        
        // Get Raspberry Pi serial number
        if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
            for line in cpuinfo.lines() {
                if line.starts_with("Serial") {
                    if let Some(serial) = line.split(':').nth(1) {
                        components.push(serial.trim().to_string());
                        break;
                    }
                }
            }
        }
        
        // Get MAC address
        if let Ok(output) = std::process::Command::new("ip")
            .args(&["link", "show"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("ether") {
                    if let Some(mac) = line.split("ether").nth(1) {
                        if let Some(mac_addr) = mac.split_whitespace().next() {
                            components.push(mac_addr.replace(':', ""));
                            break;
                        }
                    }
                }
            }
        }
        
        // Get disk UUID
        if let Ok(output) = std::process::Command::new("blkid")
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("mmcblk0p2") || line.contains("nvme0n1p2") {
                    if let Some(uuid_part) = line.split("UUID=\"").nth(1) {
                        if let Some(uuid) = uuid_part.split('"').next() {
                            components.push(uuid[..8.min(uuid.len())].to_string());
                            break;
                        }
                    }
                }
            }
        }
        
        // Generate fingerprint
        if !components.is_empty() {
            let fingerprint_str = components.join("-");
            let mut hasher = Sha256::new();
            hasher.update(fingerprint_str.as_bytes());
            let result = hasher.finalize();
            Ok(format!("{:X}", &result[..8])
                .chars()
                .take(16)
                .collect::<String>())
        } else {
            // Fallback to UUID
            Ok(uuid::Uuid::new_v4().to_string()[..16].to_uppercase())
        }
    }
    
    pub async fn check_serial_cloudflare_dns(&self, serial: &str) -> Result<bool> {
        if let Some(api_token) = &self.api_token {
            let client = reqwest::Client::new();
            
            // Get zone ID
            let zones_url = format!("https://api.cloudflare.com/client/v4/zones?name={}", self.domain);
            let response = client.get(&zones_url)
                .bearer_auth(api_token)
                .send()
                .await?;
            
            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                
                if let Some(zones) = data["result"].as_array() {
                    if let Some(zone) = zones.first() {
                        if let Some(zone_id) = zone["id"].as_str() {
                            // Check for DNS record
                            let subdomain = format!("nexuscontroller-{}", serial.to_lowercase());
                            let dns_url = format!(
                                "https://api.cloudflare.com/client/v4/zones/{}/dns_records?name={}.{}",
                                zone_id, subdomain, self.domain
                            );
                            
                            let dns_response = client.get(&dns_url)
                                .bearer_auth(api_token)
                                .send()
                                .await?;
                            
                            if dns_response.status().is_success() {
                                let dns_data: serde_json::Value = dns_response.json().await?;
                                if let Some(records) = dns_data["result"].as_array() {
                                    return Ok(!records.is_empty());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }
    
    pub fn load_existing_serial(&self) -> Option<String> {
        // Check cache file
        if self.cache_file.exists() {
            if let Ok(contents) = fs::read_to_string(&self.cache_file) {
                if let Ok(cache) = serde_json::from_str::<SerialCache>(&contents) {
                    if cache.fingerprint == self.device_fingerprint {
                        return Some(cache.serial);
                    }
                }
            }
        }
        
        // Check system serial file
        let system_serial_file = Path::new("/etc/automata-nexus/serial");
        if system_serial_file.exists() {
            if let Ok(serial) = fs::read_to_string(system_serial_file) {
                return Some(serial.trim().to_string());
            }
        }
        
        None
    }
    
    pub async fn generate_unique_serial(&self) -> Result<String> {
        // Check for existing serial
        if let Some(existing) = self.load_existing_serial() {
            println!("📋 Using existing serial: {}", existing);
            return Ok(existing);
        }
        
        let prefix = self.config.get("CONTROLLER_SERIAL_PREFIX")
            .unwrap_or(&"ANC".to_string())
            .clone();
        
        for attempt in 0..10 {
            let serial = if attempt == 0 {
                // First try: use device fingerprint
                format!("{}-{}", prefix, &self.device_fingerprint[..8])
            } else {
                // Subsequent tries: random
                let random: String = (0..6)
                    .map(|_| {
                        let n = rand::random::<u8>() % 36;
                        if n < 10 {
                            (b'0' + n) as char
                        } else {
                            (b'A' + n - 10) as char
                        }
                    })
                    .collect();
                format!("{}-{}", prefix, random)
            };
            
            println!("🔍 Checking serial: {}", serial);
            
            // Check if serial exists
            let exists = self.check_serial_cloudflare_dns(&serial).await.unwrap_or(false);
            
            if !exists {
                println!("   ✅ Serial {} is available!", serial);
                self.save_serial(&serial)?;
                return Ok(serial);
            } else {
                println!("   ❌ Serial {} already registered", serial);
            }
        }
        
        // Fallback: timestamp-based serial
        let timestamp = Utc::now().format("%y%m%d%H%M").to_string();
        let serial = format!("{}-{}", prefix, timestamp);
        self.save_serial(&serial)?;
        Ok(serial)
    }
    
    fn save_serial(&self, serial: &str) -> Result<()> {
        // Save to cache
        let cache_data = SerialCache {
            serial: serial.to_string(),
            fingerprint: self.device_fingerprint.clone(),
            created_at: Utc::now(),
            device_info: DeviceInfo {
                hostname: hostname::get()?.to_string_lossy().to_string(),
                platform: std::env::consts::ARCH.to_string(),
                system: std::env::consts::OS.to_string(),
            },
        };
        
        let json = serde_json::to_string_pretty(&cache_data)?;
        fs::write(&self.cache_file, json)?;
        
        // Try to save to system location
        let system_dir = Path::new("/etc/automata-nexus");
        if !system_dir.exists() {
            if let Err(e) = fs::create_dir_all(system_dir) {
                eprintln!("⚠️  Could not create system directory: {}", e);
            }
        }
        
        let serial_file = system_dir.join("serial");
        if let Err(e) = fs::write(&serial_file, serial) {
            eprintln!("⚠️  Could not save to system location: {}", e);
        } else {
            println!("✅ Serial saved to system: {:?}", serial_file);
        }
        
        Ok(())
    }
    
    pub async fn register_installation(&self, serial: &str, metadata: HashMap<String, String>) -> Result<bool> {
        let registration_data = serde_json::json!({
            "serial": serial,
            "fingerprint": self.device_fingerprint,
            "registered_at": Utc::now(),
            "metadata": metadata
        });
        
        // Try to register with API
        let client = reqwest::Client::new();
        match client.post(&self.registry_url)
            .header("X-API-Key", self.config.get("REGISTRY_API_KEY").unwrap_or(&String::new()))
            .json(&registration_data)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                println!("✅ Registered with central registry");
                return Ok(true);
            }
            _ => {
                println!("⚠️  Could not register with central registry (offline mode)");
            }
        }
        
        Ok(false)
    }
    
    pub async fn get_serial_info(&mut self) -> Result<SerialInfo> {
        let serial = self.generate_unique_serial().await?;
        
        let info = SerialInfo {
            serial: serial.clone(),
            fingerprint: self.device_fingerprint.clone(),
            subdomain: format!("nexuscontroller-{}", serial.to_lowercase()),
            full_domain: format!("nexuscontroller-{}.{}", serial.to_lowercase(), self.domain),
            created_at: Utc::now(),
        };
        
        // Register the installation
        let mut metadata = HashMap::new();
        metadata.insert("subdomain".to_string(), info.subdomain.clone());
        metadata.insert("domain".to_string(), self.domain.clone());
        
        self.register_installation(&serial, metadata).await?;
        
        Ok(info)
    }
}