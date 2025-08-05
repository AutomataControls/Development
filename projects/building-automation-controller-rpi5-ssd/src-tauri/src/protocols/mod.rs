// BACnet and Modbus Protocol Support Module
// RS485 protocols (BACnet MS/TP, Modbus RTU) via USB Serial Adapter
// IP protocols (BACnet IP, Modbus TCP) via Ethernet

pub mod bacnet_ip;
pub mod bacnet_mstp;
pub mod modbus_tcp;
pub mod modbus_rtu;
pub mod rs485_manager;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub protocol_type: ProtocolType,
    pub connection: ConnectionType,
    pub timeout_ms: u32,
    pub retry_count: u8,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Serial {
        port: String,      // /dev/ttyUSB0, /dev/ttyUSB1, etc.
        baud_rate: u32,
        data_bits: u8,
        stop_bits: u8,
        parity: String,
    },
    Network {
        ip_address: String,
        port: u16,
        interface: Option<String>,  // eth0, wlan0, etc.
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProtocolType {
    BacnetIp,
    BacnetMstp,
    ModbusTcp,
    ModbusRtu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePoint {
    pub device_id: String,
    pub object_type: String,
    pub instance: u32,
    pub property: String,
    pub value: PointValue,
    pub last_update: chrono::DateTime<chrono::Utc>,
    pub quality: DataQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PointValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataQuality {
    Good,
    Uncertain,
    Bad,
    Offline,
}

#[derive(Debug, Clone)]
pub struct ProtocolManager {
    configs: Arc<Mutex<HashMap<String, ProtocolConfig>>>,
    bacnet_ip: Arc<Mutex<bacnet_ip::BacnetIpManager>>,
    bacnet_mstp: Arc<Mutex<bacnet_mstp::BacnetMstpManager>>,
    modbus_tcp: Arc<Mutex<modbus_tcp::ModbusTcpManager>>,
    modbus_rtu: Arc<Mutex<modbus_rtu::ModbusRtuManager>>,
    rs485_manager: Arc<Mutex<rs485_manager::Rs485Manager>>,
}

impl ProtocolManager {
    pub async fn new() -> Result<Self, String> {
        let rs485_manager = Arc::new(Mutex::new(
            rs485_manager::Rs485Manager::new().await?
        ));

        Ok(Self {
            configs: Arc::new(Mutex::new(HashMap::new())),
            bacnet_ip: Arc::new(Mutex::new(
                bacnet_ip::BacnetIpManager::new(rs485_manager.clone()).await?
            )),
            bacnet_mstp: Arc::new(Mutex::new(
                bacnet_mstp::BacnetMstpManager::new(rs485_manager.clone()).await?
            )),
            modbus_tcp: Arc::new(Mutex::new(
                modbus_tcp::ModbusTcpManager::new(rs485_manager.clone()).await?
            )),
            modbus_rtu: Arc::new(Mutex::new(
                modbus_rtu::ModbusRtuManager::new(rs485_manager.clone()).await?
            )),
            rs485_manager,
        })
    }

    pub async fn add_protocol(&self, name: String, config: ProtocolConfig) -> Result<(), String> {
        // Validate serial port exists if using serial connection
        if let ConnectionType::Serial { port, .. } = &config.connection {
            self.rs485_manager.lock().await.validate_port(port)?;
        }
        
        // Store config
        self.configs.lock().await.insert(name.clone(), config.clone());
        
        // Initialize the appropriate protocol handler
        match config.protocol_type {
            ProtocolType::BacnetIp => {
                self.bacnet_ip.lock().await.add_connection(name, config).await?;
            }
            ProtocolType::BacnetMstp => {
                self.bacnet_mstp.lock().await.add_connection(name, config).await?;
            }
            ProtocolType::ModbusTcp => {
                self.modbus_tcp.lock().await.add_connection(name, config).await?;
            }
            ProtocolType::ModbusRtu => {
                self.modbus_rtu.lock().await.add_connection(name, config).await?;
            }
        }
        
        Ok(())
    }

    pub async fn remove_protocol(&self, name: &str) -> Result<(), String> {
        let configs = self.configs.lock().await;
        if let Some(config) = configs.get(name) {
            match config.protocol_type {
                ProtocolType::BacnetIp => {
                    self.bacnet_ip.lock().await.remove_connection(name).await?;
                }
                ProtocolType::BacnetMstp => {
                    self.bacnet_mstp.lock().await.remove_connection(name).await?;
                }
                ProtocolType::ModbusTcp => {
                    self.modbus_tcp.lock().await.remove_connection(name).await?;
                }
                ProtocolType::ModbusRtu => {
                    self.modbus_rtu.lock().await.remove_connection(name).await?;
                }
            }
        }
        drop(configs);
        self.configs.lock().await.remove(name);
        Ok(())
    }

    pub async fn read_point(&self, protocol: &str, device_id: &str, point: &str) -> Result<PointValue, String> {
        let configs = self.configs.lock().await;
        let config = configs.get(protocol)
            .ok_or_else(|| format!("Protocol {} not found", protocol))?;
        
        match config.protocol_type {
            ProtocolType::BacnetIp => {
                self.bacnet_ip.lock().await.read_point(protocol, device_id, point).await
            }
            ProtocolType::BacnetMstp => {
                self.bacnet_mstp.lock().await.read_point(protocol, device_id, point).await
            }
            ProtocolType::ModbusTcp => {
                self.modbus_tcp.lock().await.read_point(protocol, device_id, point).await
            }
            ProtocolType::ModbusRtu => {
                self.modbus_rtu.lock().await.read_point(protocol, device_id, point).await
            }
        }
    }

    pub async fn write_point(&self, protocol: &str, device_id: &str, point: &str, value: PointValue) -> Result<(), String> {
        let configs = self.configs.lock().await;
        let config = configs.get(protocol)
            .ok_or_else(|| format!("Protocol {} not found", protocol))?;
        
        match config.protocol_type {
            ProtocolType::BacnetIp => {
                self.bacnet_ip.lock().await.write_point(protocol, device_id, point, value).await
            }
            ProtocolType::BacnetMstp => {
                self.bacnet_mstp.lock().await.write_point(protocol, device_id, point, value).await
            }
            ProtocolType::ModbusTcp => {
                self.modbus_tcp.lock().await.write_point(protocol, device_id, point, value).await
            }
            ProtocolType::ModbusRtu => {
                self.modbus_rtu.lock().await.write_point(protocol, device_id, point, value).await
            }
        }
    }

    pub async fn discover_devices(&self, protocol: &str) -> Result<Vec<String>, String> {
        let configs = self.configs.lock().await;
        let config = configs.get(protocol)
            .ok_or_else(|| format!("Protocol {} not found", protocol))?;
        
        match config.protocol_type {
            ProtocolType::BacnetIp => {
                self.bacnet_ip.lock().await.discover_devices(protocol).await
            }
            ProtocolType::BacnetMstp => {
                self.bacnet_mstp.lock().await.discover_devices(protocol).await
            }
            ProtocolType::ModbusTcp => {
                self.modbus_tcp.lock().await.discover_devices(protocol).await
            }
            ProtocolType::ModbusRtu => {
                self.modbus_rtu.lock().await.discover_devices(protocol).await
            }
        }
    }

    pub async fn get_available_ports(&self) -> Result<Vec<String>, String> {
        self.rs485_manager.lock().await.list_available_ports().await
    }

    pub async fn get_protocol_status(&self, protocol: &str) -> Result<HashMap<String, String>, String> {
        let configs = self.configs.lock().await;
        let config = configs.get(protocol)
            .ok_or_else(|| format!("Protocol {} not found", protocol))?;
        
        let mut status = HashMap::new();
        status.insert("protocol".to_string(), format!("{:?}", config.protocol_type));
        
        // Extract connection details based on type
        match &config.connection {
            ConnectionType::Serial { port, baud_rate, .. } => {
                status.insert("connection_type".to_string(), "serial".to_string());
                status.insert("port".to_string(), port.clone());
                status.insert("baud_rate".to_string(), baud_rate.to_string());
            }
            ConnectionType::Network { ip_address, port, .. } => {
                status.insert("connection_type".to_string(), "network".to_string());
                status.insert("ip_address".to_string(), ip_address.clone());
                status.insert("port".to_string(), port.to_string());
            }
        }
        
        status.insert("enabled".to_string(), config.enabled.to_string());
        
        // Get protocol-specific status
        let specific_status = match config.protocol_type {
            ProtocolType::BacnetIp => {
                self.bacnet_ip.lock().await.get_status(protocol).await?
            }
            ProtocolType::BacnetMstp => {
                self.bacnet_mstp.lock().await.get_status(protocol).await?
            }
            ProtocolType::ModbusTcp => {
                self.modbus_tcp.lock().await.get_status(protocol).await?
            }
            ProtocolType::ModbusRtu => {
                self.modbus_rtu.lock().await.get_status(protocol).await?
            }
        };
        
        status.extend(specific_status);
        Ok(status)
    }
}