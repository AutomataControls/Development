#!/usr/bin/env python3
"""
Test script to verify firmware interface functionality
"""

import sys
import os
import json

# Add the src directory to path
sys.path.insert(0, 'src')

# Test the firmware interface
try:
    # Import the interface module
    import firmware_interface
    
    print("Firmware Interface Test")
    print("=" * 50)
    
    # Create interface instance
    interface = firmware_interface.SequentBoardInterface()
    
    # Test 1: Check available libraries
    print("\n1. Available Board Libraries:")
    for board_type, lib in interface.boards.items():
        print(f"   ✓ {board_type}: {lib.__name__ if hasattr(lib, '__name__') else 'loaded'}")
    
    if not interface.boards:
        print("   ✗ No board libraries available")
        print("\nTo install board libraries, run:")
        print("   sudo ./installer/install_nexus_complete.sh")
        sys.exit(1)
    
    # Test 2: Scan for boards
    print("\n2. Scanning for Connected Boards:")
    boards = interface.scan_boards()
    
    if boards:
        print(f"   Found {len(boards)} board(s):")
        for board in boards:
            print(f"   • {board['name']} (Type: {board['type']}, Stack: {board['stack']})")
            if 'version' in board:
                print(f"     Version: {board['version']}")
    else:
        print("   No boards detected")
        print("\n   Note: Boards must be physically connected and powered")
    
    # Test 3: Test MegaBAS if available
    if 'megabas' in interface.boards and any(b['type'] == 'megabas' for b in boards):
        print("\n3. Testing MegaBAS Functions:")
        megabas_board = next(b for b in boards if b['type'] == 'megabas')
        stack = megabas_board['stack']
        
        status = interface.get_megabas_status(stack)
        if 'error' not in status:
            print(f"   • Power Supply: {status['sensors']['power_supply_v']:.2f}V")
            print(f"   • Raspberry Pi: {status['sensors']['raspberry_v']:.2f}V")
            print(f"   • CPU Temperature: {status['sensors']['cpu_temp_c']:.1f}°C")
            
            # Show analog inputs
            print("\n   Analog Inputs:")
            for ch, data in status['analog_inputs'].items():
                print(f"     {ch}: {data['voltage']:.3f}V")
            
            # Show contacts
            print("\n   Dry Contacts:")
            for ch, data in status['contacts'].items():
                state = "CLOSED" if data['state'] else "OPEN"
                print(f"     {ch}: {state} (Counter: {data['counter']})")
        else:
            print(f"   Error: {status['error']}")
    
    # Test 4: Check equipment logic
    print("\n4. Equipment Logic Scripts:")
    equipment_dir = "/home/Automata/firmware/equipment-logic"
    if os.path.exists(equipment_dir):
        scripts = [f for f in os.listdir(equipment_dir) if f.endswith('.js')]
        if scripts:
            print(f"   Found {len(scripts)} equipment control script(s):")
            for script in scripts:
                print(f"   • {script}")
        else:
            print("   No equipment scripts found")
    else:
        print("   Equipment logic directory not found")
    
    print("\n" + "=" * 50)
    print("Firmware Interface Test Complete")
    
    # Test command-line interface
    print("\n5. Testing Command-Line Interface:")
    test_commands = [
        "python3 src/firmware_interface.py scan",
        "python3 src/firmware_interface.py status megabas 0",
        "python3 src/firmware_interface.py emergency_stop"
    ]
    
    for cmd in test_commands:
        print(f"\n   Testing: {cmd}")
        result = os.popen(cmd).read()
        try:
            data = json.loads(result)
            if 'error' in data:
                print(f"   ✗ Error: {data['error']}")
            else:
                print(f"   ✓ Success")
        except:
            print(f"   ✗ Invalid response: {result[:100]}")
    
    print("\n✓ All tests completed")
    
except ImportError as e:
    print(f"✗ Failed to import firmware_interface: {e}")
    print("\nMake sure the firmware interface is properly installed")
    sys.exit(1)
    
except Exception as e:
    print(f"✗ Test failed with error: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)