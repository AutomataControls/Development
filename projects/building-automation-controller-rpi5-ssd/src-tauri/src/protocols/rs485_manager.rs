// RS485 Serial Port Manager for USB Serial Adapters
// Handles BACnet MS/TP and Modbus RTU protocols

use serialport::{SerialPort, SerialPortType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

pub struct Rs485Manager {
    ports: HashMap<String, Arc<Mutex<Box<dyn SerialPort>>>>,
    port_configs: HashMap<String, Rs485Config>,
}

#[derive(Debug, Clone)]
pub struct Rs485Config {
    pub port_name: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,
    pub flow_control: String,
    pub timeout_ms: u32,
}

impl Rs485Manager {
    pub async fn new() -> Result<Self, String> {
        Ok(Self {
            ports: HashMap::new(),
            port_configs: HashMap::new(),
        })
    }

    pub async fn list_available_ports(&self) -> Result<Vec<String>, String> {
        let ports = serialport::available_ports()
            .map_err(|e| format!("Failed to list serial ports: {}", e))?;
        
        let mut usb_ports = Vec::new();
        
        for port in ports {
            // Filter for USB serial adapters
            match &port.port_type {
                SerialPortType::UsbPort(info) => {
                    let port_info = format!("{} - {} {} (Serial: {})", 
                        port.port_name,
                        info.manufacturer.as_ref().unwrap_or(&"Unknown".to_string()),
                        info.product.as_ref().unwrap_or(&"Unknown".to_string()),
                        info.serial_number.as_ref().unwrap_or(&"N/A".to_string())
                    );
                    usb_ports.push(port_info);
                    
                    println!("Found USB serial port: {} - VID:{:04x} PID:{:04x}", 
                        port.port_name, info.vid, info.pid);
                }
                _ => {
                    // Include non-USB ports that might be RS485 adapters
                    if port.port_name.contains("ttyUSB") || port.port_name.contains("ttyACM") {
                        usb_ports.push(port.port_name.clone());
                    }
                }
            }
        }
        
        if usb_ports.is_empty() {
            return Err("No USB serial adapters found. Please connect an RS485 USB adapter.".to_string());
        }
        
        Ok(usb_ports)
    }

    pub fn validate_port(&self, port_name: &str) -> Result<(), String> {
        let ports = serialport::available_ports()
            .map_err(|e| format!("Failed to list serial ports: {}", e))?;
        
        let port_exists = ports.iter().any(|p| p.port_name == port_name);
        
        if !port_exists {
            return Err(format!("Serial port {} not found", port_name));
        }
        
        Ok(())
    }

    pub async fn open_port(&mut self, config: Rs485Config) -> Result<(), String> {
        // Check if port is already open
        if self.ports.contains_key(&config.port_name) {
            return Ok(());
        }

        // Parse parity
        let parity = match config.parity.to_lowercase().as_str() {
            "none" => serialport::Parity::None,
            "odd" => serialport::Parity::Odd,
            "even" => serialport::Parity::Even,
            _ => return Err(format!("Invalid parity: {}", config.parity)),
        };

        // Parse stop bits
        let stop_bits = match config.stop_bits {
            1 => serialport::StopBits::One,
            2 => serialport::StopBits::Two,
            _ => return Err(format!("Invalid stop bits: {}", config.stop_bits)),
        };

        // Parse data bits
        let data_bits = match config.data_bits {
            5 => serialport::DataBits::Five,
            6 => serialport::DataBits::Six,
            7 => serialport::DataBits::Seven,
            8 => serialport::DataBits::Eight,
            _ => return Err(format!("Invalid data bits: {}", config.data_bits)),
        };

        // Parse flow control
        let flow_control = match config.flow_control.to_lowercase().as_str() {
            "none" => serialport::FlowControl::None,
            "hardware" => serialport::FlowControl::Hardware,
            "software" => serialport::FlowControl::Software,
            _ => serialport::FlowControl::None,
        };

        // Open the port
        let port = serialport::new(&config.port_name, config.baud_rate)
            .data_bits(data_bits)
            .stop_bits(stop_bits)
            .parity(parity)
            .flow_control(flow_control)
            .timeout(Duration::from_millis(config.timeout_ms as u64))
            .open()
            .map_err(|e| format!("Failed to open serial port {}: {}", config.port_name, e))?;

        // Store the port
        self.ports.insert(
            config.port_name.clone(),
            Arc::new(Mutex::new(port))
        );
        self.port_configs.insert(config.port_name.clone(), config);

        Ok(())
    }

    pub async fn close_port(&mut self, port_name: &str) -> Result<(), String> {
        self.ports.remove(port_name);
        self.port_configs.remove(port_name);
        Ok(())
    }

    pub async fn write_data(&self, port_name: &str, data: &[u8]) -> Result<(), String> {
        let port = self.ports.get(port_name)
            .ok_or_else(|| format!("Port {} not open", port_name))?;
        
        let mut port = port.lock().await;
        
        // Clear any existing data in buffers
        port.clear(serialport::ClearBuffer::All)
            .map_err(|e| format!("Failed to clear buffers: {}", e))?;
        
        // Write data
        port.write_all(data)
            .map_err(|e| format!("Failed to write to port: {}", e))?;
        
        // Flush to ensure data is sent
        port.flush()
            .map_err(|e| format!("Failed to flush port: {}", e))?;
        
        Ok(())
    }

    pub async fn read_data(&self, port_name: &str, buffer_size: usize) -> Result<Vec<u8>, String> {
        let port = self.ports.get(port_name)
            .ok_or_else(|| format!("Port {} not open", port_name))?;
        
        let mut port = port.lock().await;
        let mut buffer = vec![0u8; buffer_size];
        
        match port.read(&mut buffer) {
            Ok(bytes_read) => {
                buffer.truncate(bytes_read);
                Ok(buffer)
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    Ok(vec![])  // Timeout is not an error, just no data
                } else {
                    Err(format!("Failed to read from port: {}", e))
                }
            }
        }
    }

    pub async fn write_read(&self, port_name: &str, data: &[u8], response_size: usize) -> Result<Vec<u8>, String> {
        // Write data
        self.write_data(port_name, data).await?;
        
        // Small delay for response
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Read response
        self.read_data(port_name, response_size).await
    }

    pub fn get_port_config(&self, port_name: &str) -> Option<&Rs485Config> {
        self.port_configs.get(port_name)
    }

    pub async fn set_rs485_mode(&self, port_name: &str, enable: bool) -> Result<(), String> {
        // For USB RS485 adapters, this is usually automatic
        // Some adapters may require RTS control for direction switching
        
        let port = self.ports.get(port_name)
            .ok_or_else(|| format!("Port {} not open", port_name))?;
        
        let mut port = port.lock().await;
        
        // Set RTS for transmit enable (adapter-specific)
        port.write_request_to_send(enable)
            .map_err(|e| format!("Failed to set RTS: {}", e))?;
        
        Ok(())
    }

    pub async fn test_port(&self, port_name: &str) -> Result<String, String> {
        // Simple loopback test or echo test
        let test_data = b"TEST123";
        
        self.write_data(port_name, test_data).await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let response = self.read_data(port_name, 128).await?;
        
        if response.is_empty() {
            Ok("Port opened successfully, no loopback detected".to_string())
        } else {
            Ok(format!("Port opened successfully, received {} bytes", response.len()))
        }
    }
}