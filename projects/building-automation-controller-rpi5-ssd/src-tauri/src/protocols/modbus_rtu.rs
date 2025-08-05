// Modbus RTU Protocol Implementation  
// RS485 serial communication via USB adapter

use super::{ProtocolConfig, ConnectionType, PointValue};
use super::rs485_manager::{Rs485Manager, Rs485Config};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

const MIN_FRAME_SIZE: usize = 4;  // Address + Function + CRC
const MAX_PDU_SIZE: usize = 253;
const INTER_FRAME_DELAY_MS: u64 = 4;  // 3.5 character times at 9600 baud

pub struct ModbusRtuManager {
    rs485: Arc<Mutex<Rs485Manager>>,
    connections: HashMap<String, ModbusRtuConnection>,
}

struct ModbusRtuConnection {
    config: ProtocolConfig,
    devices: HashMap<u8, ModbusRtuDevice>,  // slave_id -> device
    inter_frame_delay: Duration,
}

#[derive(Debug, Clone)]
struct ModbusRtuDevice {
    slave_id: u8,
    description: String,
    register_map: HashMap<u16, RegisterInfo>,
}

#[derive(Debug, Clone)]
struct RegisterInfo {
    register_type: RegisterType,
    data_type: DataType,
    scale: f64,
    offset: f64,
    description: String,
}

#[derive(Debug, Clone, Copy)]
enum RegisterType {
    HoldingRegister,
    InputRegister,
    Coil,
    DiscreteInput,
}

#[derive(Debug, Clone, Copy)]
enum DataType {
    UInt16,
    Int16,
    UInt32,
    Int32,
    Float32,
    Boolean,
}

#[derive(Debug, Copy, Clone)]
enum FunctionCode {
    ReadCoils = 0x01,
    ReadDiscreteInputs = 0x02,
    ReadHoldingRegisters = 0x03,
    ReadInputRegisters = 0x04,
    WriteSingleCoil = 0x05,
    WriteSingleRegister = 0x06,
    WriteMultipleCoils = 0x0F,
    WriteMultipleRegisters = 0x10,
}

impl ModbusRtuManager {
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

            // Calculate inter-frame delay based on baud rate
            // Modbus RTU requires 3.5 character times between frames
            let bits_per_char = 1 + *data_bits + *stop_bits + 
                               if parity != "none" { 1 } else { 0 };
            let char_time_us = (bits_per_char as f64 * 1_000_000.0) / (*baud_rate as f64);
            let inter_frame_delay = Duration::from_micros((char_time_us * 3.5) as u64);

            let connection = ModbusRtuConnection {
                config,
                devices: HashMap::new(),
                inter_frame_delay,
            };

            self.connections.insert(name, connection);
            Ok(())
        } else {
            Err("Modbus RTU requires serial connection type".to_string())
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

    pub async fn add_device(&mut self, connection_name: &str, slave_id: u8, description: String) -> Result<(), String> {
        let connection = self.connections.get_mut(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        let device = ModbusRtuDevice {
            slave_id,
            description,
            register_map: HashMap::new(),
        };

        connection.devices.insert(slave_id, device);
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

        // Scan common slave addresses (1-247)
        for slave_id in 1..=247 {
            // Try to read device identification (if supported)
            // Using Read Holding Registers at address 0 as a simple test
            let request = self.build_request(slave_id, FunctionCode::ReadHoldingRegisters, 0, 1, &[]);
            
            // Add delay between requests
            tokio::time::sleep(connection.inter_frame_delay).await;
            
            match self.rs485.lock().await.write_read(&port_name, &request, 256).await {
                Ok(response) => {
                    if self.validate_response(&response, slave_id).is_ok() {
                        devices.push(format!("Modbus RTU Device at address {}", slave_id));
                        
                        // Add to device list
                        let device = ModbusRtuDevice {
                            slave_id,
                            description: format!("Device {}", slave_id),
                            register_map: HashMap::new(),
                        };
                        connection.devices.insert(slave_id, device);
                    }
                }
                Err(_) => {
                    // No response or error - device not present
                }
            }

            // Limit scan to first 10 addresses for speed
            if slave_id >= 10 && devices.is_empty() {
                break;
            }
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

        // Parse device ID (slave address)
        let slave_id: u8 = device_id.parse()
            .map_err(|_| format!("Invalid device ID: {}", device_id))?;

        // Parse point format: "HR:100" (Holding Register 100), "IR:200" (Input Register 200), etc.
        let parts: Vec<&str> = point.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid point format: {}", point));
        }

        let register_type = parts[0];
        let address: u16 = parts[1].parse()
            .map_err(|_| format!("Invalid address: {}", parts[1]))?;

        // Determine function code and read
        let (function_code, value) = match register_type {
            "HR" => {
                let data = self.read_registers(
                    &port_name, 
                    slave_id, 
                    FunctionCode::ReadHoldingRegisters, 
                    address, 
                    1,
                    connection.inter_frame_delay
                ).await?;
                (FunctionCode::ReadHoldingRegisters, self.parse_register_value(&data, DataType::UInt16))
            }
            "IR" => {
                let data = self.read_registers(
                    &port_name, 
                    slave_id, 
                    FunctionCode::ReadInputRegisters, 
                    address, 
                    1,
                    connection.inter_frame_delay
                ).await?;
                (FunctionCode::ReadInputRegisters, self.parse_register_value(&data, DataType::UInt16))
            }
            "C" => {
                let data = self.read_bits(
                    &port_name, 
                    slave_id, 
                    FunctionCode::ReadCoils, 
                    address, 
                    1,
                    connection.inter_frame_delay
                ).await?;
                (FunctionCode::ReadCoils, PointValue::Bool(data[0]))
            }
            "DI" => {
                let data = self.read_bits(
                    &port_name, 
                    slave_id, 
                    FunctionCode::ReadDiscreteInputs, 
                    address, 
                    1,
                    connection.inter_frame_delay
                ).await?;
                (FunctionCode::ReadDiscreteInputs, PointValue::Bool(data[0]))
            }
            _ => return Err(format!("Unknown register type: {}", register_type)),
        };

        Ok(value)
    }

    pub async fn write_point(&self, connection_name: &str, device_id: &str, point: &str, value: PointValue) -> Result<(), String> {
        let connection = self.connections.get(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        let port_name = if let ConnectionType::Serial { port, .. } = &connection.config.connection {
            port.clone()
        } else {
            return Err("Invalid connection type".to_string());
        };

        // Parse device ID
        let slave_id: u8 = device_id.parse()
            .map_err(|_| format!("Invalid device ID: {}", device_id))?;

        // Parse point format
        let parts: Vec<&str> = point.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid point format: {}", point));
        }

        let register_type = parts[0];
        let address: u16 = parts[1].parse()
            .map_err(|_| format!("Invalid address: {}", parts[1]))?;

        // Write based on type
        match register_type {
            "HR" => {
                self.write_register(
                    &port_name, 
                    slave_id, 
                    address, 
                    value,
                    connection.inter_frame_delay
                ).await
            }
            "C" => {
                self.write_coil(
                    &port_name, 
                    slave_id, 
                    address, 
                    value,
                    connection.inter_frame_delay
                ).await
            }
            _ => Err(format!("Cannot write to {} registers", register_type)),
        }
    }

    // Low-level Modbus RTU functions

    async fn read_registers(
        &self, 
        port: &str, 
        slave_id: u8, 
        function: FunctionCode, 
        address: u16, 
        count: u16,
        delay: Duration
    ) -> Result<Vec<u8>, String> {
        let request = self.build_request(slave_id, function, address, count, &[]);
        
        tokio::time::sleep(delay).await;
        
        let response = self.rs485.lock().await
            .write_read(port, &request, 256).await?;
        
        self.validate_response(&response, slave_id)?;
        
        // Extract register data
        if response.len() >= 5 {
            let byte_count = response[2] as usize;
            if response.len() >= 3 + byte_count + 2 {
                Ok(response[3..3+byte_count].to_vec())
            } else {
                Err("Response too short".to_string())
            }
        } else {
            Err("Invalid response format".to_string())
        }
    }

    async fn read_bits(
        &self, 
        port: &str, 
        slave_id: u8, 
        function: FunctionCode, 
        address: u16, 
        count: u16,
        delay: Duration
    ) -> Result<Vec<bool>, String> {
        let request = self.build_request(slave_id, function, address, count, &[]);
        
        tokio::time::sleep(delay).await;
        
        let response = self.rs485.lock().await
            .write_read(port, &request, 256).await?;
        
        self.validate_response(&response, slave_id)?;
        
        // Extract bit data
        if response.len() >= 5 {
            let byte_count = response[2] as usize;
            if response.len() >= 3 + byte_count + 2 {
                let mut bits = Vec::new();
                for i in 0..count {
                    let byte_idx = (i / 8) as usize;
                    let bit_idx = (i % 8) as usize;
                    if byte_idx < byte_count {
                        bits.push((response[3 + byte_idx] & (1 << bit_idx)) != 0);
                    }
                }
                Ok(bits)
            } else {
                Err("Response too short".to_string())
            }
        } else {
            Err("Invalid response format".to_string())
        }
    }

    async fn write_register(
        &self, 
        port: &str, 
        slave_id: u8, 
        address: u16, 
        value: PointValue,
        delay: Duration
    ) -> Result<(), String> {
        let register_value = match value {
            PointValue::Int(i) => i as u16,
            PointValue::Float(f) => f as u16,
            _ => return Err("Invalid value type for register".to_string()),
        };

        let data = register_value.to_be_bytes();
        let request = self.build_request(
            slave_id, 
            FunctionCode::WriteSingleRegister, 
            address, 
            register_value, 
            &data
        );
        
        tokio::time::sleep(delay).await;
        
        let response = self.rs485.lock().await
            .write_read(port, &request, 256).await?;
        
        self.validate_response(&response, slave_id)?;
        Ok(())
    }

    async fn write_coil(
        &self, 
        port: &str, 
        slave_id: u8, 
        address: u16, 
        value: PointValue,
        delay: Duration
    ) -> Result<(), String> {
        let coil_value = match value {
            PointValue::Bool(b) => if b { 0xFF00u16 } else { 0x0000u16 },
            _ => return Err("Invalid value type for coil".to_string()),
        };

        let request = self.build_request(
            slave_id, 
            FunctionCode::WriteSingleCoil, 
            address, 
            coil_value, 
            &[]
        );
        
        tokio::time::sleep(delay).await;
        
        let response = self.rs485.lock().await
            .write_read(port, &request, 256).await?;
        
        self.validate_response(&response, slave_id)?;
        Ok(())
    }

    fn build_request(&self, slave_id: u8, function: FunctionCode, address: u16, value: u16, data: &[u8]) -> Vec<u8> {
        let mut request = Vec::new();
        
        // Slave address
        request.push(slave_id);
        
        // Function code
        request.push(function as u8);
        
        // Address
        request.extend_from_slice(&address.to_be_bytes());
        
        // Value/Count
        request.extend_from_slice(&value.to_be_bytes());
        
        // Additional data
        request.extend_from_slice(data);
        
        // CRC
        let crc = self.calculate_crc16(&request);
        request.extend_from_slice(&crc.to_le_bytes());
        
        request
    }

    fn validate_response(&self, response: &[u8], expected_slave: u8) -> Result<(), String> {
        if response.len() < MIN_FRAME_SIZE {
            return Err("Response too short".to_string());
        }
        
        // Check slave address
        if response[0] != expected_slave {
            return Err(format!("Slave address mismatch: expected {}, got {}", 
                             expected_slave, response[0]));
        }
        
        // Check for exception
        if response[1] & 0x80 != 0 {
            let exception_code = if response.len() > 2 { response[2] } else { 0 };
            return Err(format!("Modbus exception: {:02X}", exception_code));
        }
        
        // Verify CRC
        let data_len = response.len() - 2;
        let calculated_crc = self.calculate_crc16(&response[..data_len]);
        let received_crc = u16::from_le_bytes([response[data_len], response[data_len + 1]]);
        
        if calculated_crc != received_crc {
            return Err(format!("CRC error: calculated {:04X}, received {:04X}", 
                             calculated_crc, received_crc));
        }
        
        Ok(())
    }

    fn calculate_crc16(&self, data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        
        for byte in data {
            crc ^= *byte as u16;
            for _ in 0..8 {
                if crc & 0x0001 != 0 {
                    crc = (crc >> 1) ^ 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }
        
        crc
    }

    fn parse_register_value(&self, data: &[u8], data_type: DataType) -> PointValue {
        match data_type {
            DataType::UInt16 => {
                if data.len() >= 2 {
                    PointValue::Int(u16::from_be_bytes([data[0], data[1]]) as i64)
                } else {
                    PointValue::Int(0)
                }
            }
            DataType::Int16 => {
                if data.len() >= 2 {
                    PointValue::Int(i16::from_be_bytes([data[0], data[1]]) as i64)
                } else {
                    PointValue::Int(0)
                }
            }
            DataType::Float32 => {
                if data.len() >= 4 {
                    let value = f32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                    PointValue::Float(value as f64)
                } else {
                    PointValue::Float(0.0)
                }
            }
            _ => PointValue::Int(0),
        }
    }

    pub async fn get_status(&self, connection_name: &str) -> Result<HashMap<String, String>, String> {
        let connection = self.connections.get(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        let mut status = HashMap::new();
        status.insert("protocol".to_string(), "Modbus RTU".to_string());
        status.insert("devices_found".to_string(), connection.devices.len().to_string());
        
        if let ConnectionType::Serial { port, baud_rate, data_bits, stop_bits, parity } = &connection.config.connection {
            status.insert("port".to_string(), port.clone());
            status.insert("baud_rate".to_string(), baud_rate.to_string());
            status.insert("data_bits".to_string(), data_bits.to_string());
            status.insert("stop_bits".to_string(), stop_bits.to_string());
            status.insert("parity".to_string(), parity.clone());
            status.insert("inter_frame_delay_us".to_string(), 
                         connection.inter_frame_delay.as_micros().to_string());
        }

        // List devices
        for (slave_id, device) in &connection.devices {
            status.insert(
                format!("device_{}", slave_id),
                device.description.clone()
            );
        }
        
        Ok(status)
    }
}