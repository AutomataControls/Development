// BACnet IP Protocol Implementation
// Network-based communication (NOT RS485)

use super::{ProtocolConfig, ConnectionType, PointValue, DataQuality};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::UdpSocket;
use std::net::{IpAddr, SocketAddr};

const BACNET_PORT: u16 = 47808;  // Standard BACnet/IP port
const BACNET_BROADCAST_PORT: u16 = 47809;

pub struct BacnetIpManager {
    connections: HashMap<String, BacnetIpConnection>,
}

struct BacnetIpConnection {
    config: ProtocolConfig,
    socket: Arc<UdpSocket>,
    device_cache: HashMap<u32, BacnetDevice>,
}

#[derive(Debug, Clone)]
struct BacnetDevice {
    instance: u32,
    ip_address: IpAddr,
    port: u16,
    vendor_id: u16,
    object_list: Vec<BacnetObject>,
}

#[derive(Debug, Clone)]
struct BacnetObject {
    object_type: u16,
    instance: u32,
    name: String,
}

impl BacnetIpManager {
    pub async fn new(_rs485: Arc<Mutex<super::rs485_manager::Rs485Manager>>) -> Result<Self, String> {
        // Note: rs485 parameter included for API consistency but not used for IP
        Ok(Self {
            connections: HashMap::new(),
        })
    }

    pub async fn add_connection(&mut self, name: String, config: ProtocolConfig) -> Result<(), String> {
        if let ConnectionType::Network { ip_address, port, interface } = &config.connection {
            // Bind to the specified interface or any available
            let bind_addr = if let Some(iface) = interface {
                // TODO: Get IP address of specific interface
                format!("0.0.0.0:{}", port)
            } else {
                format!("0.0.0.0:{}", port)
            };

            let socket = UdpSocket::bind(&bind_addr).await
                .map_err(|e| format!("Failed to bind BACnet/IP socket: {}", e))?;

            // Enable broadcast
            socket.set_broadcast(true)
                .map_err(|e| format!("Failed to enable broadcast: {}", e))?;

            let connection = BacnetIpConnection {
                config,
                socket: Arc::new(socket),
                device_cache: HashMap::new(),
            };

            self.connections.insert(name, connection);
            Ok(())
        } else {
            Err("BACnet/IP requires network connection type".to_string())
        }
    }

    pub async fn remove_connection(&mut self, name: &str) -> Result<(), String> {
        self.connections.remove(name);
        Ok(())
    }

    pub async fn discover_devices(&mut self, connection_name: &str) -> Result<Vec<String>, String> {
        let connection = self.connections.get_mut(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        // Send Who-Is broadcast
        let who_is = self.create_who_is_message();
        let broadcast_addr = "255.255.255.255:47808".parse::<SocketAddr>()
            .map_err(|e| format!("Invalid broadcast address: {}", e))?;

        connection.socket.send_to(&who_is, broadcast_addr).await
            .map_err(|e| format!("Failed to send Who-Is: {}", e))?;

        // Listen for I-Am responses
        let mut devices = Vec::new();
        let mut buffer = vec![0u8; 1500];
        
        // Set a timeout for discovery
        let timeout = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.collect_i_am_responses(&connection.socket, &mut buffer, &mut connection.device_cache)
        ).await;

        match timeout {
            Ok(Ok(_)) => {
                for (instance, device) in &connection.device_cache {
                    devices.push(format!("Device {} at {}:{}", 
                        instance, device.ip_address, device.port));
                }
                Ok(devices)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                // Timeout is normal - return what we found
                for (instance, device) in &connection.device_cache {
                    devices.push(format!("Device {} at {}:{}", 
                        instance, device.ip_address, device.port));
                }
                Ok(devices)
            }
        }
    }

    pub async fn read_point(&self, connection_name: &str, device_id: &str, point: &str) -> Result<PointValue, String> {
        let connection = self.connections.get(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        // Parse device ID (instance number)
        let device_instance: u32 = device_id.parse()
            .map_err(|_| format!("Invalid device ID: {}", device_id))?;

        // Get device from cache
        let device = connection.device_cache.get(&device_instance)
            .ok_or_else(|| format!("Device {} not found", device_id))?;

        // Parse point (e.g., "AV:1" for Analog Value instance 1)
        let parts: Vec<&str> = point.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid point format: {}", point));
        }

        let object_type = self.parse_object_type(parts[0])?;
        let instance: u32 = parts[1].parse()
            .map_err(|_| format!("Invalid instance: {}", parts[1]))?;

        // Create ReadProperty request
        let request = self.create_read_property_request(
            device_instance,
            object_type,
            instance,
            85  // Present Value property
        );

        // Send request
        let device_addr = SocketAddr::new(device.ip_address, device.port);
        connection.socket.send_to(&request, device_addr).await
            .map_err(|e| format!("Failed to send ReadProperty: {}", e))?;

        // Wait for response
        let mut buffer = vec![0u8; 1500];
        let timeout = tokio::time::timeout(
            std::time::Duration::from_millis(connection.config.timeout_ms as u64),
            connection.socket.recv_from(&mut buffer)
        ).await;

        match timeout {
            Ok(Ok((len, _addr))) => {
                self.parse_read_property_response(&buffer[..len])
            }
            Ok(Err(e)) => Err(format!("Failed to receive response: {}", e)),
            Err(_) => Err("Read timeout".to_string()),
        }
    }

    pub async fn write_point(&self, connection_name: &str, device_id: &str, point: &str, value: PointValue) -> Result<(), String> {
        let connection = self.connections.get(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        // Parse device ID
        let device_instance: u32 = device_id.parse()
            .map_err(|_| format!("Invalid device ID: {}", device_id))?;

        // Get device from cache
        let device = connection.device_cache.get(&device_instance)
            .ok_or_else(|| format!("Device {} not found", device_id))?;

        // Parse point
        let parts: Vec<&str> = point.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid point format: {}", point));
        }

        let object_type = self.parse_object_type(parts[0])?;
        let instance: u32 = parts[1].parse()
            .map_err(|_| format!("Invalid instance: {}", parts[1]))?;

        // Create WriteProperty request
        let request = self.create_write_property_request(
            device_instance,
            object_type,
            instance,
            85,  // Present Value property
            value
        );

        // Send request
        let device_addr = SocketAddr::new(device.ip_address, device.port);
        connection.socket.send_to(&request, device_addr).await
            .map_err(|e| format!("Failed to send WriteProperty: {}", e))?;

        // Wait for ACK
        let mut buffer = vec![0u8; 1500];
        let timeout = tokio::time::timeout(
            std::time::Duration::from_millis(connection.config.timeout_ms as u64),
            connection.socket.recv_from(&mut buffer)
        ).await;

        match timeout {
            Ok(Ok((len, _addr))) => {
                self.parse_write_property_ack(&buffer[..len])
            }
            Ok(Err(e)) => Err(format!("Failed to receive ACK: {}", e)),
            Err(_) => Err("Write timeout".to_string()),
        }
    }

    pub async fn get_status(&self, connection_name: &str) -> Result<HashMap<String, String>, String> {
        let connection = self.connections.get(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        let mut status = HashMap::new();
        status.insert("connected".to_string(), "true".to_string());
        status.insert("devices_discovered".to_string(), connection.device_cache.len().to_string());
        
        if let ConnectionType::Network { ip_address, port, .. } = &connection.config.connection {
            status.insert("bind_address".to_string(), format!("{}:{}", ip_address, port));
        }

        Ok(status)
    }

    // Helper methods for BACnet protocol

    fn create_who_is_message(&self) -> Vec<u8> {
        // Simplified BACnet Who-Is message
        // This is a basic implementation - real BACnet requires proper encoding
        vec![
            0x81, 0x0b, 0x00, 0x0c, // BVLC header
            0x01, 0x20, 0x00, 0x00, // NPDU
            0x10, 0x08,             // Who-Is service
        ]
    }

    async fn collect_i_am_responses(
        &self, 
        socket: &UdpSocket, 
        buffer: &mut [u8],
        device_cache: &mut HashMap<u32, BacnetDevice>
    ) -> Result<(), String> {
        loop {
            match socket.recv_from(buffer).await {
                Ok((len, addr)) => {
                    // Parse I-Am response (simplified)
                    if len > 10 && buffer[8] == 0x10 && buffer[9] == 0x00 {
                        // Extract device instance (simplified parsing)
                        let instance = ((buffer[10] as u32) << 16) | 
                                     ((buffer[11] as u32) << 8) | 
                                     (buffer[12] as u32);
                        
                        let device = BacnetDevice {
                            instance,
                            ip_address: addr.ip(),
                            port: addr.port(),
                            vendor_id: 0,  // Would parse from message
                            object_list: vec![],
                        };
                        
                        device_cache.insert(instance, device);
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        return Err(format!("Error receiving I-Am: {}", e));
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    fn parse_object_type(&self, type_str: &str) -> Result<u16, String> {
        match type_str.to_uppercase().as_str() {
            "AI" => Ok(0),   // Analog Input
            "AO" => Ok(1),   // Analog Output
            "AV" => Ok(2),   // Analog Value
            "BI" => Ok(3),   // Binary Input
            "BO" => Ok(4),   // Binary Output
            "BV" => Ok(5),   // Binary Value
            "MI" => Ok(13),  // Multi-state Input
            "MO" => Ok(14),  // Multi-state Output
            "MV" => Ok(19),  // Multi-state Value
            _ => Err(format!("Unknown object type: {}", type_str)),
        }
    }

    fn create_read_property_request(&self, device: u32, object_type: u16, instance: u32, property: u32) -> Vec<u8> {
        // Simplified BACnet ReadProperty request
        // Real implementation would use proper BACnet encoding
        vec![
            0x81, 0x0a, 0x00, 0x17, // BVLC header
            0x01, 0x24, 0x00, 0x00, // NPDU
            0x00, 0x00, 0x00, 0x00, // Invoke ID
            0x0c,                   // ReadProperty
            // Object identifier, property ID, etc. would go here
        ]
    }

    fn create_write_property_request(&self, device: u32, object_type: u16, instance: u32, property: u32, value: PointValue) -> Vec<u8> {
        // Simplified BACnet WriteProperty request
        vec![
            0x81, 0x0a, 0x00, 0x17, // BVLC header
            0x01, 0x24, 0x00, 0x00, // NPDU
            0x00, 0x00, 0x00, 0x00, // Invoke ID
            0x0f,                   // WriteProperty
            // Object identifier, property ID, value would go here
        ]
    }

    fn parse_read_property_response(&self, data: &[u8]) -> Result<PointValue, String> {
        // Simplified parsing - real implementation would decode BACnet properly
        if data.len() < 12 {
            return Err("Invalid response".to_string());
        }
        
        // Mock response parsing
        Ok(PointValue::Float(72.5))
    }

    fn parse_write_property_ack(&self, data: &[u8]) -> Result<(), String> {
        // Simplified parsing
        if data.len() < 8 {
            return Err("Invalid ACK".to_string());
        }
        Ok(())
    }
}