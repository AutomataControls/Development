// Modbus TCP Protocol Implementation
// Network-based communication (NOT RS485)

use super::{ProtocolConfig, ConnectionType, PointValue};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;

const MODBUS_TCP_PORT: u16 = 502;  // Standard Modbus TCP port
const MODBUS_HEADER_SIZE: usize = 7;
const MAX_READ_REGISTERS: u16 = 125;
const MAX_WRITE_REGISTERS: u16 = 123;

pub struct ModbusTcpManager {
    connections: HashMap<String, ModbusTcpConnection>,
}

struct ModbusTcpConnection {
    config: ProtocolConfig,
    devices: HashMap<String, ModbusTcpDevice>,
}

struct ModbusTcpDevice {
    address: SocketAddr,
    unit_id: u8,
    connection: Option<Arc<Mutex<TcpStream>>>,
    transaction_id: Arc<Mutex<u16>>,
}

#[derive(Debug)]
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

impl ModbusTcpManager {
    pub async fn new(_rs485: Arc<Mutex<super::rs485_manager::Rs485Manager>>) -> Result<Self, String> {
        // Note: rs485 parameter included for API consistency but not used for TCP
        Ok(Self {
            connections: HashMap::new(),
        })
    }

    pub async fn add_connection(&mut self, name: String, config: ProtocolConfig) -> Result<(), String> {
        if let ConnectionType::Network { .. } = &config.connection {
            let connection = ModbusTcpConnection {
                config,
                devices: HashMap::new(),
            };

            self.connections.insert(name, connection);
            Ok(())
        } else {
            Err("Modbus TCP requires network connection type".to_string())
        }
    }

    pub async fn remove_connection(&mut self, name: &str) -> Result<(), String> {
        if let Some(mut connection) = self.connections.remove(name) {
            // Close all device connections
            for (_, device) in connection.devices.iter_mut() {
                if let Some(conn) = &device.connection {
                    if let Ok(mut stream) = conn.lock().await.shutdown().await {
                        // Connection closed
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn add_device(&mut self, connection_name: &str, device_id: String, ip_address: String, unit_id: u8) -> Result<(), String> {
        let connection = self.connections.get_mut(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        let port = if let ConnectionType::Network { port, .. } = &connection.config.connection {
            *port
        } else {
            MODBUS_TCP_PORT
        };

        let address: SocketAddr = format!("{}:{}", ip_address, port)
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?;

        let device = ModbusTcpDevice {
            address,
            unit_id,
            connection: None,
            transaction_id: Arc::new(Mutex::new(1)),
        };

        connection.devices.insert(device_id, device);
        Ok(())
    }

    pub async fn discover_devices(&mut self, connection_name: &str) -> Result<Vec<String>, String> {
        let connection = self.connections.get(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        // For Modbus TCP, devices must be manually configured
        // Return list of configured devices
        let devices: Vec<String> = connection.devices.keys()
            .map(|k| k.clone())
            .collect();

        Ok(devices)
    }

    async fn connect_to_device(&self, device: &mut ModbusTcpDevice) -> Result<(), String> {
        if device.connection.is_none() {
            let stream = TcpStream::connect(device.address).await
                .map_err(|e| format!("Failed to connect to {}: {}", device.address, e))?;
            
            device.connection = Some(Arc::new(Mutex::new(stream)));
        }
        Ok(())
    }

    pub async fn read_point(&self, connection_name: &str, device_id: &str, point: &str) -> Result<PointValue, String> {
        let connection = self.connections.get(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        let device = connection.devices.get(device_id)
            .ok_or_else(|| format!("Device {} not found", device_id))?;

        // Parse point format: "HR:100" (Holding Register 100), "IR:200" (Input Register 200), etc.
        let parts: Vec<&str> = point.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid point format: {}", point));
        }

        let register_type = parts[0];
        let address: u16 = parts[1].parse()
            .map_err(|_| format!("Invalid address: {}", parts[1]))?;

        match register_type {
            "HR" => self.read_holding_registers(device, address, 1).await,
            "IR" => self.read_input_registers(device, address, 1).await,
            "C" => self.read_coils(device, address, 1).await,
            "DI" => self.read_discrete_inputs(device, address, 1).await,
            _ => Err(format!("Unknown register type: {}", register_type)),
        }
    }

    pub async fn write_point(&self, connection_name: &str, device_id: &str, point: &str, value: PointValue) -> Result<(), String> {
        let connection = self.connections.get(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        let device = connection.devices.get(device_id)
            .ok_or_else(|| format!("Device {} not found", device_id))?;

        // Parse point format
        let parts: Vec<&str> = point.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid point format: {}", point));
        }

        let register_type = parts[0];
        let address: u16 = parts[1].parse()
            .map_err(|_| format!("Invalid address: {}", parts[1]))?;

        match register_type {
            "HR" => self.write_holding_register(device, address, value).await,
            "C" => self.write_coil(device, address, value).await,
            _ => Err(format!("Cannot write to {} registers", register_type)),
        }
    }

    async fn read_holding_registers(&self, device: &ModbusTcpDevice, address: u16, count: u16) -> Result<PointValue, String> {
        let response = self.modbus_transaction(
            device,
            FunctionCode::ReadHoldingRegisters,
            address,
            count,
            &[]
        ).await?;

        // Parse response (skip header and byte count)
        if response.len() >= 9 && count == 1 {
            let value = u16::from_be_bytes([response[9], response[10]]);
            Ok(PointValue::Int(value as i64))
        } else {
            Err("Invalid response".to_string())
        }
    }

    async fn read_input_registers(&self, device: &ModbusTcpDevice, address: u16, count: u16) -> Result<PointValue, String> {
        let response = self.modbus_transaction(
            device,
            FunctionCode::ReadInputRegisters,
            address,
            count,
            &[]
        ).await?;

        // Parse response
        if response.len() >= 9 && count == 1 {
            let value = u16::from_be_bytes([response[9], response[10]]);
            Ok(PointValue::Int(value as i64))
        } else {
            Err("Invalid response".to_string())
        }
    }

    async fn read_coils(&self, device: &ModbusTcpDevice, address: u16, count: u16) -> Result<PointValue, String> {
        let response = self.modbus_transaction(
            device,
            FunctionCode::ReadCoils,
            address,
            count,
            &[]
        ).await?;

        // Parse response
        if response.len() >= 9 && count == 1 {
            let value = (response[9] & 0x01) != 0;
            Ok(PointValue::Bool(value))
        } else {
            Err("Invalid response".to_string())
        }
    }

    async fn read_discrete_inputs(&self, device: &ModbusTcpDevice, address: u16, count: u16) -> Result<PointValue, String> {
        let response = self.modbus_transaction(
            device,
            FunctionCode::ReadDiscreteInputs,
            address,
            count,
            &[]
        ).await?;

        // Parse response
        if response.len() >= 9 && count == 1 {
            let value = (response[9] & 0x01) != 0;
            Ok(PointValue::Bool(value))
        } else {
            Err("Invalid response".to_string())
        }
    }

    async fn write_holding_register(&self, device: &ModbusTcpDevice, address: u16, value: PointValue) -> Result<(), String> {
        let register_value = match value {
            PointValue::Int(i) => i as u16,
            PointValue::Float(f) => f as u16,
            _ => return Err("Invalid value type for register".to_string()),
        };

        let data = register_value.to_be_bytes();
        
        self.modbus_transaction(
            device,
            FunctionCode::WriteSingleRegister,
            address,
            register_value,
            &data
        ).await?;

        Ok(())
    }

    async fn write_coil(&self, device: &ModbusTcpDevice, address: u16, value: PointValue) -> Result<(), String> {
        let coil_value = match value {
            PointValue::Bool(b) => if b { 0xFF00u16 } else { 0x0000u16 },
            _ => return Err("Invalid value type for coil".to_string()),
        };

        self.modbus_transaction(
            device,
            FunctionCode::WriteSingleCoil,
            address,
            coil_value,
            &[]
        ).await?;

        Ok(())
    }

    async fn modbus_transaction(
        &self,
        device: &ModbusTcpDevice,
        function: FunctionCode,
        address: u16,
        value: u16,
        data: &[u8]
    ) -> Result<Vec<u8>, String> {
        // Get or create connection
        let connection = device.connection.as_ref()
            .ok_or("Device not connected")?;

        let mut stream = connection.lock().await;

        // Get transaction ID
        let transaction_id = {
            let mut tid = device.transaction_id.lock().await;
            let current = *tid;
            *tid = tid.wrapping_add(1);
            current
        };

        // Build Modbus TCP frame
        let mut frame = Vec::new();
        
        // Transaction ID
        frame.extend_from_slice(&transaction_id.to_be_bytes());
        
        // Protocol ID (0 for Modbus)
        frame.extend_from_slice(&0u16.to_be_bytes());
        
        // Length (will be updated)
        let length_pos = frame.len();
        frame.extend_from_slice(&0u16.to_be_bytes());
        
        // Unit ID
        frame.push(device.unit_id);
        
        // Function code
        frame.push(function as u8);
        
        // Address
        frame.extend_from_slice(&address.to_be_bytes());
        
        // Value/Count
        frame.extend_from_slice(&value.to_be_bytes());
        
        // Additional data (for write multiple)
        frame.extend_from_slice(data);
        
        // Update length field
        let pdu_length = (frame.len() - 6) as u16;
        frame[length_pos..length_pos+2].copy_from_slice(&pdu_length.to_be_bytes());
        
        // Send request
        stream.write_all(&frame).await
            .map_err(|e| format!("Failed to send request: {}", e))?;
        
        // Read response
        let mut response = vec![0u8; 256];
        let n = stream.read(&mut response).await
            .map_err(|e| format!("Failed to read response: {}", e))?;
        
        response.truncate(n);
        
        // Verify transaction ID
        if response.len() >= 2 {
            let resp_tid = u16::from_be_bytes([response[0], response[1]]);
            if resp_tid != transaction_id {
                return Err("Transaction ID mismatch".to_string());
            }
        }
        
        // Check for exception
        if response.len() >= 8 && response[7] & 0x80 != 0 {
            let exception_code = response[8];
            return Err(format!("Modbus exception: {:02X}", exception_code));
        }
        
        Ok(response)
    }

    pub async fn get_status(&self, connection_name: &str) -> Result<HashMap<String, String>, String> {
        let connection = self.connections.get(connection_name)
            .ok_or_else(|| format!("Connection {} not found", connection_name))?;

        let mut status = HashMap::new();
        status.insert("protocol".to_string(), "Modbus TCP".to_string());
        status.insert("devices_configured".to_string(), connection.devices.len().to_string());
        
        // Check connection status for each device
        let mut connected_count = 0;
        for (device_id, device) in &connection.devices {
            if device.connection.is_some() {
                connected_count += 1;
            }
            status.insert(
                format!("device_{}_address", device_id),
                device.address.to_string()
            );
        }
        
        status.insert("devices_connected".to_string(), connected_count.to_string());
        
        Ok(status)
    }
}