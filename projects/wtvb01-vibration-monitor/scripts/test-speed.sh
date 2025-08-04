#!/bin/bash

# WTVB01-485 Speed Test Script
# Tests sensor communication at different baud rates

echo "========================================"
echo "WTVB01-485 Sensor Speed Test"
echo "========================================"
echo ""

# Check for USB ports
echo "Checking for USB serial ports..."
ls -la /dev/ttyUSB* 2>/dev/null || {
    echo "No USB serial ports found!"
    echo "Please connect WTVB01-485 sensor via USB-RS485 adapter."
    exit 1
}

# Function to test sensor at specific baud rate
test_baud_rate() {
    local port=$1
    local baud=$2
    
    echo ""
    echo "Testing $port at $baud baud..."
    
    # Send Modbus command to read temperature (register 0x40)
    # Command: 50 03 00 40 00 01 [CRC]
    # This reads 1 register starting at 0x40 (temperature)
    
    stty -F $port $baud cs8 -cstopb -parenb 2>/dev/null
    
    # Send command and wait for response
    echo -ne '\x50\x03\x00\x40\x00\x01\x90\x6F' > $port
    
    # Try to read response (timeout after 1 second)
    timeout 1 cat $port | od -An -tx1 | head -1
    
    if [ $? -eq 0 ]; then
        echo "✓ Sensor responded at $baud baud"
        return 0
    else
        echo "✗ No response at $baud baud"
        return 1
    fi
}

# Function to measure read speed
measure_speed() {
    local port=$1
    local baud=$2
    local reads=10
    
    echo ""
    echo "Measuring read speed at $baud baud ($reads reads)..."
    
    stty -F $port $baud cs8 -cstopb -parenb 2>/dev/null
    
    start_time=$(date +%s%N)
    
    for i in $(seq 1 $reads); do
        # Read temperature register
        echo -ne '\x50\x03\x00\x40\x00\x01\x90\x6F' > $port
        timeout 0.1 cat $port > /dev/null 2>&1
    done
    
    end_time=$(date +%s%N)
    elapsed=$((($end_time - $start_time) / 1000000))
    
    echo "Completed $reads reads in ${elapsed}ms"
    echo "Average: $((elapsed / reads))ms per read"
}

# Main test sequence
for port in /dev/ttyUSB*; do
    echo ""
    echo "========================================"
    echo "Testing port: $port"
    echo "========================================"
    
    # Test different baud rates
    for baud in 9600 19200 38400 57600 115200 230400; do
        if test_baud_rate $port $baud; then
            # If sensor responds, measure speed
            measure_speed $port $baud
            
            echo ""
            echo "Speed comparison:"
            case $baud in
                9600)
                    echo "  Factory default speed"
                    ;;
                115200)
                    echo "  12x faster than factory (optimized default)"
                    ;;
                230400)
                    echo "  24x faster than factory (maximum speed!)"
                    ;;
            esac
            
            break # Found working baud rate
        fi
    done
done

echo ""
echo "========================================"
echo "Speed Test Complete"
echo "========================================"
echo ""
echo "Recommendations:"
echo "1. Use 115200 baud for good performance (12x faster)"
echo "2. Use 230400 baud for maximum speed (24x faster)"
echo "3. Enable burst reading to read all 19 registers at once"
echo "4. Enable 1000Hz mode for critical applications"
echo ""
echo "To optimize your sensors:"
echo "1. Run the Tauri app"
echo "2. Click 'Optimize Speed' to set 230400 baud"
echo "3. Click 'Enable 1000Hz Mode' for fastest sampling"