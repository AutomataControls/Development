#!/usr/bin/env python3
"""
HVAC Diagnostic System - Sensor Interface
Interfaces with SM-I-002 Building Automation HAT using megabas library
"""

import megabas
import json
import time
import sys
from datetime import datetime
from typing import Dict, List, Optional

class P499TransducerConfig:
    """Configuration for P499 Series Electronic Pressure Transducers"""
    
    PRESSURE_RANGES = {
        "P499VAP-101C": {"min": 0, "max": 100},    # 0-100 psi
        "P499VAP-102C": {"min": 0, "max": 200},    # 0-200 psi  
        "P499VAP-105C": {"min": 0, "max": 500},    # 0-500 psi
        "P499VAP-107C": {"min": 0, "max": 750},    # 0-750 psi
        "P499VAPS100C": {"min": -10, "max": 100},  # -10 to 100 psi
    }
    
    def __init__(self, model: str = "P499VAP-105C"):
        if model not in self.PRESSURE_RANGES:
            raise ValueError(f"Unknown transducer model: {model}")
        
        self.model = model
        self.min_pressure = self.PRESSURE_RANGES[model]["min"]
        self.max_pressure = self.PRESSURE_RANGES[model]["max"]
    
    def voltage_to_pressure(self, voltage: float) -> float:
        """Convert 0-10V signal to pressure using P499 formula"""
        # P499 0-10V formula: Vout = 10.0 × (P - Pmin) / (Pmax - Pmin)
        # Rearranged: P = Pmin + (Vout / 10.0) × (Pmax - Pmin)
        if voltage < 0 or voltage > 10:
            raise ValueError(f"Voltage out of range: {voltage}V")
        
        pressure = self.min_pressure + (voltage / 10.0) * (self.max_pressure - self.min_pressure)
        return round(pressure, 2)
    
    def pressure_to_voltage(self, pressure: float) -> float:
        """Convert pressure to expected voltage for calibration"""
        if pressure < self.min_pressure or pressure > self.max_pressure:
            raise ValueError(f"Pressure out of range: {pressure} psi")
        
        voltage = 10.0 * (pressure - self.min_pressure) / (self.max_pressure - self.min_pressure)
        return round(voltage, 3)

class HVACDiagnosticInterface:
    """Main interface class for HVAC diagnostic system"""
    
    def __init__(self, stack_level: int = 0):
        self.stack_level = stack_level
        self.transducer_configs = {}
        self.last_readings = {}
        
        # Default transducer configurations
        self.configure_transducer(1, "P499VAP-105C")  # Suction pressure
        self.configure_transducer(2, "P499VAP-105C")  # Discharge pressure
        self.configure_transducer(3, "P499VAP-101C")  # Liquid line pressure
        self.configure_transducer(4, "P499VAP-101C")  # Evaporator pressure
    
    def configure_transducer(self, channel: int, model: str):
        """Configure a pressure transducer for a specific channel"""
        if channel < 1 or channel > 8:
            raise ValueError("Channel must be between 1 and 8")
        
        self.transducer_configs[channel] = P499TransducerConfig(model)
        print(f"Configured channel {channel} for {model} ({self.transducer_configs[channel].min_pressure}-{self.transducer_configs[channel].max_pressure} psi)")
    
    def read_single_pressure(self, channel: int) -> Dict:
        """Read pressure from a single channel"""
        try:
            # Read voltage from megabas
            voltage = megabas.getUIn(self.stack_level, channel)
            
            # Get transducer configuration
            if channel not in self.transducer_configs:
                raise ValueError(f"Channel {channel} not configured")
            
            config = self.transducer_configs[channel]
            
            # Convert to pressure
            pressure = config.voltage_to_pressure(voltage)
            
            reading = {
                "channel": channel,
                "voltage": voltage,
                "pressure": pressure,
                "model": config.model,
                "range": f"{config.min_pressure}-{config.max_pressure} psi",
                "timestamp": datetime.utcnow().isoformat(),
                "status": "OK"
            }
            
            self.last_readings[channel] = reading
            return reading
            
        except Exception as e:
            error_reading = {
                "channel": channel,
                "voltage": 0.0,
                "pressure": 0.0,
                "model": "Unknown",
                "range": "Unknown",
                "timestamp": datetime.utcnow().isoformat(),
                "status": "ERROR",
                "error": str(e)
            }
            return error_reading
    
    def read_all_pressures(self) -> List[Dict]:
        """Read pressures from all configured channels"""
        readings = []
        for channel in range(1, 9):
            if channel in self.transducer_configs:
                reading = self.read_single_pressure(channel)
                readings.append(reading)
        return readings
    
    def calibrate_transducer(self, channel: int, known_pressure: float) -> Dict:
        """Calibrate a transducer against a known pressure"""
        if channel not in self.transducer_configs:
            raise ValueError(f"Channel {channel} not configured")
        
        config = self.transducer_configs[channel]
        
        # Read current voltage
        measured_voltage = megabas.getUIn(self.stack_level, channel)
        
        # Calculate expected voltage
        expected_voltage = config.pressure_to_voltage(known_pressure)
        
        # Calculate error
        voltage_error = measured_voltage - expected_voltage
        pressure_error = config.voltage_to_pressure(measured_voltage) - known_pressure
        error_percent = abs(pressure_error / known_pressure * 100) if known_pressure != 0 else 0
        
        calibration_result = {
            "channel": channel,
            "known_pressure": known_pressure,
            "measured_voltage": measured_voltage,
            "expected_voltage": expected_voltage,
            "voltage_error": voltage_error,
            "pressure_error": pressure_error,
            "error_percent": error_percent,
            "within_spec": error_percent <= 1.0,  # P499 spec is ±1%
            "timestamp": datetime.utcnow().isoformat()
        }
        
        return calibration_result
    
    def get_system_status(self) -> Dict:
        """Get overall system status"""
        try:
            # Read system voltages
            input_voltage = megabas.getInVolt(self.stack_level)
            pi_voltage = megabas.getRaspVolt(self.stack_level)
            cpu_temp = megabas.getCpuTemp(self.stack_level)
            
            # Get firmware version
            firmware_version = megabas.getVer(self.stack_level)
            
            status = {
                "stack_level": self.stack_level,
                "input_voltage": input_voltage,
                "pi_voltage": pi_voltage,
                "cpu_temperature": cpu_temp,
                "firmware_version": firmware_version,
                "configured_channels": list(self.transducer_configs.keys()),
                "timestamp": datetime.utcnow().isoformat(),
                "status": "OK"
            }
            
            return status
            
        except Exception as e:
            return {
                "status": "ERROR",
                "error": str(e),
                "timestamp": datetime.utcnow().isoformat()
            }

def main():
    """Main function for command-line usage"""
    if len(sys.argv) < 2:
        print("Usage: python3 sensor_interface.py <command> [args]")
        print("Commands:")
        print("  read_all - Read all configured pressure sensors")
        print("  read_single <channel> - Read single channel")
        print("  calibrate <channel> <known_pressure> - Calibrate transducer")
        print("  status - Get system status")
        print("  configure <channel> <model> - Configure transducer")
        return
    
    interface = HVACDiagnosticInterface()
    command = sys.argv[1]
    
    try:
        if command == "read_all":
            readings = interface.read_all_pressures()
            print(json.dumps(readings, indent=2))
        
        elif command == "read_single":
            if len(sys.argv) < 3:
                print("Error: Channel number required")
                return
            channel = int(sys.argv[2])
            reading = interface.read_single_pressure(channel)
            print(json.dumps(reading, indent=2))
        
        elif command == "calibrate":
            if len(sys.argv) < 4:
                print("Error: Channel and known pressure required")
                return
            channel = int(sys.argv[2])
            known_pressure = float(sys.argv[3])
            result = interface.calibrate_transducer(channel, known_pressure)
            print(json.dumps(result, indent=2))
        
        elif command == "status":
            status = interface.get_system_status()
            print(json.dumps(status, indent=2))
        
        elif command == "configure":
            if len(sys.argv) < 4:
                print("Error: Channel and model required")
                return
            channel = int(sys.argv[2])
            model = sys.argv[3]
            interface.configure_transducer(channel, model)
            print(f"Configured channel {channel} for {model}")
        
        else:
            print(f"Unknown command: {command}")
    
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    main()
