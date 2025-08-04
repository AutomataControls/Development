#!/usr/bin/env python3
"""
Sequent Microsystems MegaBAS HAT Interface for 0-10V Inputs
Uses official megabas library for P499 pressure transducer readings
"""

import sys
import time

try:
    import megabas
except ImportError:
    print("Error: megabas library not installed")
    print("Install with: pip3 install megabas")
    sys.exit(1)

# Constants
ANALOG_IN_CH_NO = 8

def get_0_10v(stack_level, channel):
    """
    Read 0-10V input from specified channel
    
    Args:
        stack_level: HAT stack position (0-8)
        channel: Input channel (0-7, converted to 1-8 for megabas)
    
    Returns:
        Voltage reading (0.0-10.0)
    """
    if channel < 0 or channel >= ANALOG_IN_CH_NO:
        raise ValueError(f"Invalid channel: {channel}")
    
    if stack_level < 0 or stack_level > 8:
        raise ValueError(f"Invalid stack level: {stack_level}")
    
    try:
        # megabas uses 1-based channel numbering
        voltage = megabas.getUIn(stack_level, channel + 1)
        return voltage
    except Exception as e:
        raise Exception(f"Failed to read 0-10V channel {channel}: {e}")

def scan_all_channels(stack_level):
    """
    Scan all 8 channels and return voltage readings
    
    Args:
        stack_level: HAT stack position (0-8)
    
    Returns:
        List of 8 voltage readings
    """
    readings = []
    
    for channel in range(ANALOG_IN_CH_NO):
        try:
            value = get_0_10v(stack_level, channel)
            readings.append(value)
        except Exception as e:
            readings.append(None)
    
    return readings

def test_hat(stack_level=0):
    """Test if HAT is accessible"""
    try:
        # Try to read first channel
        megabas.getUIn(stack_level, 1)
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
        print("  scan [stack]              - Scan all channels")
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
        
        elif command == "scan":
            stack = int(sys.argv[2])
            readings = scan_all_channels(stack)
            for i, reading in enumerate(readings):
                if reading is not None:
                    print(f"Channel {i}: {reading:.3f} V")
                else:
                    print(f"Channel {i}: Error")
        
        else:
            print(f"Unknown command: {command}")
            sys.exit(1)
            
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)