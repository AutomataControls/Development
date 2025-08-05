// BACnet MS/TP Protocol Implementation
// RS485 serial communication via USB adapter

use super::{ProtocolConfig, ConnectionType, PointValue, DataQuality};
use super::rs485_manager::{Rs485Manager, Rs485Config};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

const MSTP_HEADER_SIZE: usize = 8;
const MAX_MASTER: u8 = 127;
const MAX_INFO_FRAMES: u8 = 1;

pub struct BacnetMstpManager {
    rs485: Arc<Mutex<Rs485Manager>>,
    connections: HashMap<String, BacnetMstpConnection>,
}

struct BacnetMstpConnection {
    config: ProtocolConfig,
    mac_address: u8,
    max_master: u8,
    max_info_frames: u8,
    token_count: u8,
    device_cache: HashMap<u8, MstpDevice>,
}

#[derive(Debug, Clone)]
struct MstpDevice {
    mac_address: u8,
    vendor_id: u16,
    max_apdu: u16,
    object_list: Vec<MstpObject>,
}

#[derive(Debug, Clone)]
struct MstpObject {
    object_type: u16,
    instance: u32,
    name: String,
}

#[derive(Debug)]
enum FrameType {
    Token = 0,
    PollForMaster = 1,
    ReplyToPollForMaster = 2,
    TestRequest = 3,
    TestResponse = 4,
    BacnetDataExpectingReply = 5,
    BacnetDataNotExpectingReply = 6,
    ReplyPostponed = 7,
}

impl BacnetMstpManager {
    pub async fn new(rs485: Arc<Mutex<Rs485Manager>>) -> Result<Self, String> {
        Ok(Self {
            rs485,
            connections: HashMap::new(),
        })
    }

    pub async fn add_connection(&mut self, name: String, config: ProtocolConfig) -> Result<(), String> {
        if let ConnectionType::Serial { port, baud_rate, data_bits, stop_bits, parity } = &config.connection {
            // Configure RS485 port
            let rs485_config = Rs485Config {
                port_name: port.clone(),
                baud_rate: *baud_rate,
                data_bits: *data_bits,
                stop_bits: *stop_bits,
                parity: parity.clone(),
                flow_control: "none".to_string(),
                timeout_ms: config.timeout_ms,
            };

            // Open the RS485 port
            self.rs485.lock().await.open_port(rs485_config).await?;

            let connection = BacnetMstpConnection {
                config,
                mac_address: 1,  // Default MAC address, should be configurable
                max_master: MAX_MASTER,
                max_info_frames: MAX_INFO_FRAMES,
                token_count: 0,
                device_cache: HashMap::new(),
            };

            self.connections.insert(name, connection);
            Ok(())
        } else {
            Err("BACnet MS/TP requires serial connection type".to_string())
        }
    }

    pub async fn remove_connection(&mut self, name: &str) -> Result<(), String> {
        if let Some(connection) = self.connections.get(name) {
            if let ConnectionType::Serial { port, .. } = &connection.config.connection {
                self.rs485.lock().await.close_port(port).await?;
            }
        }
        self.connections.remove(name);
        Ok(())
    }

    pub async fn discover_devices(&mut self, connection_name: &str) -> Result<Vec<String>, String> {
        let connection = self.connections.get_mut(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        let port_name = if let ConnectionType::Serial { port, .. } = &connection.config.connection {
            port.clone()
        } else {
            return Err("Invalid connection type".to_string());
        };

        let mut devices = Vec::new();

        // Poll for masters on the MS/TP network
        for mac in 0..=connection.max_master {
            if mac == connection.mac_address {
                continue; // Skip our own address
            }

            // Send Poll For Master frame
            let frame = self.create_poll_for_master_frame(connection.mac_address, mac);
            
            match self.rs485.lock().await.write_read(&port_name, &frame, 256).await {
                Ok(response) => {
                    if self.is_reply_to_poll_for_master(&response) {
                        let device = MstpDevice {
                            mac_address: mac,
                            vendor_id: 0,  // Would parse from response
                            max_apdu: 480,
                            object_list: vec![],
                        };
                        
                        connection.device_cache.insert(mac, device);
                        devices.push(format!("MS/TP Device at MAC {}", mac));
                    }
                }
                Err(_) => {
                    // No response, device not present
                }
            }

            // Small delay between polls
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Ok(devices)
    }

    pub async fn read_point(&self, connection_name: &str, device_id: &str, point: &str) -> Result<PointValue, String> {
        let connection = self.connections.get(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        let port_name = if let ConnectionType::Serial { port, .. } = &connection.config.connection {
            port.clone()
        } else {
            return Err("Invalid connection type".to_string());
        };

        // Parse device MAC address
        let device_mac: u8 = device_id.parse()
            .map_err(|_| format!("Invalid device MAC: {}", device_id))?;

        // Parse point (e.g., "AV:1")
        let parts: Vec<&str> = point.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid point format: {}", point));
        }

        let object_type = self.parse_object_type(parts[0])?;
        let instance: u32 = parts[1].parse()
            .map_err(|_| format!("Invalid instance: {}", parts[1]))?;

        // Create BACnet APDU for ReadProperty
        let apdu = self.create_read_property_apdu(object_type, instance, 85); // Property 85 = Present Value

        // Wrap in MS/TP frame
        let frame = self.create_data_frame(
            connection.mac_address,
            device_mac,
            true,  // Expecting reply
            &apdu
        );

        // Send and wait for response
        let response = self.rs485.lock().await
            .write_read(&port_name, &frame, 512).await
            .map_err(|e| format!("Failed to read point: {}", e))?;

        // Parse response
        self.parse_read_property_response(&response)
    }

    pub async fn write_point(&self, connection_name: &str, device_id: &str, point: &str, value: PointValue) -> Result<(), String> {
        let connection = self.connections.get(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        let port_name = if let ConnectionType::Serial { port, .. } = &connection.config.connection {
            port.clone()
        } else {
            return Err("Invalid connection type".to_string());
        };

        // Parse device MAC address
        let device_mac: u8 = device_id.parse()
            .map_err(|_| format!("Invalid device MAC: {}", device_id))?;

        // Parse point
        let parts: Vec<&str> = point.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid point format: {}", point));
        }

        let object_type = self.parse_object_type(parts[0])?;
        let instance: u32 = parts[1].parse()
            .map_err(|_| format!("Invalid instance: {}", parts[1]))?;

        // Create BACnet APDU for WriteProperty
        let apdu = self.create_write_property_apdu(object_type, instance, 85, value);

        // Wrap in MS/TP frame
        let frame = self.create_data_frame(
            connection.mac_address,
            device_mac,
            true,  // Expecting reply
            &apdu
        );

        // Send and wait for ACK
        let response = self.rs485.lock().await
            .write_read(&port_name, &frame, 256).await
            .map_err(|e| format!("Failed to write point: {}", e))?;

        // Verify ACK
        if response.len() > MSTP_HEADER_SIZE {
            Ok(())
        } else {
            Err("No acknowledgment received".to_string())
        }
    }

    pub async fn get_status(&self, connection_name: &str) -> Result<HashMap<String, String>, String> {
        let connection = self.connections.get(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        let mut status = HashMap::new();
        status.insert("mac_address".to_string(), connection.mac_address.to_string());
        status.insert("max_master".to_string(), connection.max_master.to_string());
        status.insert("devices_found".to_string(), connection.device_cache.len().to_string());
        
        if let ConnectionType::Serial { port, baud_rate, .. } = &connection.config.connection {
            status.insert("port".to_string(), port.clone());
            status.insert("baud_rate".to_string(), baud_rate.to_string());
        }

        Ok(status)
    }

    // MS/TP Frame Creation Methods

    fn create_poll_for_master_frame(&self, source: u8, dest: u8) -> Vec<u8> {
        self.create_mstp_frame(
            FrameType::PollForMaster as u8,
            dest,
            source,
            &[]
        )
    }

    fn create_data_frame(&self, source: u8, dest: u8, expecting_reply: bool, data: &[u8]) -> Vec<u8> {
        let frame_type = if expecting_reply {
            FrameType::BacnetDataExpectingReply as u8
        } else {
            FrameType::BacnetDataNotExpectingReply as u8
        };

        self.create_mstp_frame(frame_type, dest, source, data)
    }

    fn create_mstp_frame(&self, frame_type: u8, dest: u8, source: u8, data: &[u8]) -> Vec<u8> {
        let mut frame = Vec::with_capacity(MSTP_HEADER_SIZE + data.len() + 2);

        // Preamble
        frame.push(0x55);
        frame.push(0xFF);

        // Frame Type
        frame.push(frame_type);

        // Destination
        frame.push(dest);

        // Source
        frame.push(source);

        // Length (MSB, LSB)
        let len = data.len() as u16;
        frame.push((len >> 8) as u8);
        frame.push((len & 0xFF) as u8);

        // Header CRC
        let header_crc = self.calculate_crc8(&frame[2..7]);
        frame.push(header_crc);

        // Data
        frame.extend_from_slice(data);

        // Data CRC (16-bit)
        if !data.is_empty() {
            let data_crc = self.calculate_crc16(data);
            frame.push((data_crc >> 8) as u8);
            frame.push((data_crc & 0xFF) as u8);
        }

        frame
    }

    fn is_reply_to_poll_for_master(&self, data: &[u8]) -> bool {
        data.len() >= MSTP_HEADER_SIZE && 
        data[0] == 0x55 && 
        data[1] == 0xFF && 
        data[2] == FrameType::ReplyToPollForMaster as u8
    }

    // BACnet APDU Creation Methods

    fn create_read_property_apdu(&self, object_type: u16, instance: u32, property_id: u32) -> Vec<u8> {
        // Simplified BACnet APDU for ReadProperty
        // Real implementation would use proper BACnet encoding
        vec![
            0x00, // PDU Type: Confirmed Request
            0x04, // Max segments, max APDU
            0x00, // Invoke ID
            0x0C, // Service Choice: ReadProperty
            // Object Identifier
            0x0C, // Context tag 0, 4 bytes
            ((object_type >> 2) & 0xFF) as u8,
            (((object_type & 0x03) << 6) | ((instance >> 16) & 0x3F)) as u8,
            ((instance >> 8) & 0xFF) as u8,
            (instance & 0xFF) as u8,
            // Property Identifier
            0x19, // Context tag 1, 1 byte
            property_id as u8,
        ]
    }

    fn create_write_property_apdu(&self, object_type: u16, instance: u32, property_id: u32, value: PointValue) -> Vec<u8> {
        // Simplified BACnet APDU for WriteProperty
        let mut apdu = vec![
            0x00, // PDU Type: Confirmed Request
            0x04, // Max segments, max APDU
            0x00, // Invoke ID
            0x0F, // Service Choice: WriteProperty
            // Object Identifier
            0x0C, // Context tag 0, 4 bytes
            ((object_type >> 2) & 0xFF) as u8,
            (((object_type & 0x03) << 6) | ((instance >> 16) & 0x3F)) as u8,
            ((instance >> 8) & 0xFF) as u8,
            (instance & 0xFF) as u8,
            // Property Identifier
            0x19, // Context tag 1, 1 byte
            property_id as u8,
        ];

        // Add value (simplified encoding)
        match value {
            PointValue::Float(f) => {
                apdu.push(0x3E); // Opening tag 3
                apdu.push(0x44); // REAL tag
                let bytes = f.to_be_bytes();
                apdu.extend_from_slice(&bytes);
                apdu.push(0x3F); // Closing tag 3
            }
            PointValue::Bool(b) => {
                apdu.push(0x3E); // Opening tag 3
                apdu.push(0x11); // BOOLEAN tag
                apdu.push(if b { 1 } else { 0 });
                apdu.push(0x3F); // Closing tag 3
            }
            _ => {
                // Other types would be encoded here
            }
        }

        apdu
    }

    fn parse_read_property_response(&self, data: &[u8]) -> Result<PointValue, String> {
        // Skip MS/TP header and check for valid response
        if data.len() < MSTP_HEADER_SIZE + 5 {
            return Err("Response too short".to_string());
        }

        // Very simplified parsing - real implementation would properly decode BACnet
        let apdu_start = MSTP_HEADER_SIZE;
        
        // Look for REAL value tag (0x44)
        for i in apdu_start..data.len()-4 {
            if data[i] == 0x44 {
                let value_bytes = &data[i+1..i+5];
                let value = f32::from_be_bytes([
                    value_bytes[0], value_bytes[1], 
                    value_bytes[2], value_bytes[3]
                ]);
                return Ok(PointValue::Float(value as f64));
            }
        }

        Err("Could not parse response".to_string())
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

    // CRC Calculation Methods

    fn calculate_crc8(&self, data: &[u8]) -> u8 {
        let mut crc: u8 = 0xFF;
        
        for byte in data {
            crc ^= *byte;
            for _ in 0..8 {
                if crc & 0x80 != 0 {
                    crc = (crc << 1) ^ 0x07;
                } else {
                    crc <<= 1;
                }
            }
        }
        
        !crc
    }

    fn calculate_crc16(&self, data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        
        for byte in data {
            crc ^= (*byte as u16) << 8;
            for _ in 0..8 {
                if crc & 0x8000 != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        
        !crc
    }
}