#!/usr/bin/env python3
"""
Sequent Microsystems 0-10V/4-20mA HAT Interface
Provides functions to read analog inputs from the HAT
"""

import smbus
import time
import sys

# I2C device address
DEVICE_ADDRESS = 0x51  # Default address, can be changed with jumpers

# Register addresses
ANALOG_IN_VAL1_ADD = 0x00
ANALOG_IN_VAL2_ADD = 0x02
ANALOG_IN_VAL3_ADD = 0x04
ANALOG_IN_VAL4_ADD = 0x06
ANALOG_IN_VAL5_ADD = 0x08
ANALOG_IN_VAL6_ADD = 0x0A
ANALOG_IN_VAL7_ADD = 0x0C
ANALOG_IN_VAL8_ADD = 0x0E

# Calibration registers
ANALOG_IN_CAL_CMD_ADD = 0x10
ANALOG_IN_CAL_VAL_ADD = 0x11

# Constants
ANALOG_IN_CH_NO = 8
ANALOG_VAL_MAX = 4095  # 12-bit ADC
ANALOG_VREF = 3.3      # Reference voltage

def get_address(stack_level):
    """Calculate I2C address based on stack level (0-7)"""
    return DEVICE_ADDRESS + stack_level

def read_word(bus, address, register):
    """Read 16-bit word from I2C device"""
    try:
        high = bus.read_byte_data(address, register)
        low = bus.read_byte_data(address, register + 1)
        return (high << 8) + low
    except Exception as e:
        raise Exception(f"I2C read error: {e}")

def get_0_10v(stack_level, channel):
    """
    Read 0-10V input from specified channel
    
    Args:
        stack_level: HAT stack position (0-7)
        channel: Input channel (0-7)
    
    Returns:
        Voltage reading (0.0-10.0)
    """
    if channel < 0 or channel >= ANALOG_IN_CH_NO:
        raise ValueError(f"Invalid channel: {channel}")
    
    if stack_level < 0 or stack_level > 7:
        raise ValueError(f"Invalid stack level: {stack_level}")
    
    try:
        bus = smbus.SMBus(1)  # Use I2C bus 1
        address = get_address(stack_level)
        
        # Calculate register address for channel
        register = ANALOG_IN_VAL1_ADD + (channel * 2)
        
        # Read raw ADC value
        raw_value = read_word(bus, address, register)
        
        # Convert to voltage (0-10V range)
        voltage = (raw_value / ANALOG_VAL_MAX) * 10.0
        
        bus.close()
        return voltage
        
    except Exception as e:
        raise Exception(f"Failed to read 0-10V channel {channel}: {e}")

def get_4_20ma(stack_level, channel):
    """
    Read 4-20mA input from specified channel
    
    Args:
        stack_level: HAT stack position (0-7)
        channel: Input channel (0-7)
    
    Returns:
        Current reading in mA (0.0-20.0)
    """
    if channel < 0 or channel >= ANALOG_IN_CH_NO:
        raise ValueError(f"Invalid channel: {channel}")
    
    if stack_level < 0 or stack_level > 7:
        raise ValueError(f"Invalid stack level: {stack_level}")
    
    try:
        bus = smbus.SMBus(1)
        address = get_address(stack_level)
        
        # Calculate register address for channel
        register = ANALOG_IN_VAL1_ADD + (channel * 2)
        
        # Read raw ADC value
        raw_value = read_word(bus, address, register)
        
        # Convert to current (0-20mA range)
        # The HAT uses a 250 ohm resistor for 4-20mA conversion
        # 20mA * 250 ohm = 5V, which is scaled to 10V range
        current = (raw_value / ANALOG_VAL_MAX) * 20.0
        
        bus.close()
        return current
        
    except Exception as e:
        raise Exception(f"Failed to read 4-20mA channel {channel}: {e}")

def calibrate_channel(stack_level, channel, cal_value):
    """
    Calibrate a channel with known input value
    
    Args:
        stack_level: HAT stack position (0-7)
        channel: Input channel (0-7)
        cal_value: Calibration value in mV (0-10000)
    """
    if channel < 0 or channel >= ANALOG_IN_CH_NO:
        raise ValueError(f"Invalid channel: {channel}")
    
    if cal_value < 0 or cal_value > 10000:
        raise ValueError(f"Invalid calibration value: {cal_value}")
    
    try:
        bus = smbus.SMBus(1)
        address = get_address(stack_level)
        
        # Write calibration value (in mV)
        bus.write_word_data(address, ANALOG_IN_CAL_VAL_ADD, cal_value)
        
        # Execute calibration command
        bus.write_byte_data(address, ANALOG_IN_CAL_CMD_ADD, channel + 1)
        
        # Wait for calibration to complete
        time.sleep(0.1)
        
        bus.close()
        
    except Exception as e:
        raise Exception(f"Failed to calibrate channel {channel}: {e}")

def scan_all_channels(stack_level, mode='voltage'):
    """
    Scan all 8 channels and return readings
    
    Args:
        stack_level: HAT stack position (0-7)
        mode: 'voltage' for 0-10V, 'current' for 4-20mA
    
    Returns:
        List of 8 readings
    """
    readings = []
    
    for channel in range(ANALOG_IN_CH_NO):
        try:
            if mode == 'voltage':
                value = get_0_10v(stack_level, channel)
            else:
                value = get_4_20ma(stack_level, channel)
            readings.append(value)
        except Exception as e:
            readings.append(None)
    
    return readings

def test_hat(stack_level=0):
    """Test if HAT is accessible"""
    try:
        bus = smbus.SMBus(1)
        address = get_address(stack_level)
        
        # Try to read first channel
        read_word(bus, address, ANALOG_IN_VAL1_ADD)
        
        bus.close()
        return True
    except:
        return False

# Command line interface
if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 sm_4_20ma.py [command] [args]")
        print("Commands:")
        print("  test [stack_level]        - Test HAT connection")
        print("  voltage [stack] [channel] - Read 0-10V input")
        print("  current [stack] [channel] - Read 4-20mA input")
        print("  scan [stack] [mode]       - Scan all channels")
        sys.exit(1)
    
    command = sys.argv[1]
    
    try:
        if command == "test":
            stack = int(sys.argv[2]) if len(sys.argv) > 2 else 0
            if test_hat(stack):
                print("HAT detected successfully")
            else:
                print("HAT not found")
        
        elif command == "voltage":
            stack = int(sys.argv[2])
            channel = int(sys.argv[3])
            voltage = get_0_10v(stack, channel)
            print(f"{voltage:.3f}")
        
        elif command == "current":
            stack = int(sys.argv[2])
            channel = int(sys.argv[3])
            current = get_4_20ma(stack, channel)
            print(f"{current:.3f}")
        
        elif command == "scan":
            stack = int(sys.argv[2])
            mode = sys.argv[3] if len(sys.argv) > 3 else 'voltage'
            readings = scan_all_channels(stack, mode)
            for i, reading in enumerate(readings):
                if reading is not None:
                    unit = "V" if mode == 'voltage' else "mA"
                    print(f"Channel {i}: {reading:.3f} {unit}")
                else:
                    print(f"Channel {i}: Error")
        
        else:
            print(f"Unknown command: {command}")
            sys.exit(1)
            
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)