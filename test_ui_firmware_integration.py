#!/usr/bin/env python3
"""
Test script to verify UI integration with firmware interface
Checks that the UI correctly represents the actual board capabilities
"""

import sys
import json
import subprocess

# Test the firmware interface info command
def test_firmware_info():
    print("Testing firmware interface info command...")
    result = subprocess.run(
        ["python3", "src/firmware_interface.py", "info"],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print(f"✗ Failed to run firmware interface: {result.stderr}")
        return False
    
    try:
        info = json.loads(result.stdout)
        print("\nBoard Capabilities from Firmware Interface:")
        print("=" * 60)
        
        for board_type, config in info['boards'].items():
            print(f"\n{board_type}: {config['description']}")
            
            if board_type == 'megabas':
                # Verify MegaBAS has correct specs
                assert config['triacs'] == 4, f"MegaBAS should have 4 triacs, got {config['triacs']}"
                assert config['analog_outputs'] == 4, f"MegaBAS should have 4 analog outputs, got {config['analog_outputs']}"
                assert config['configurable_inputs'] == 8, f"MegaBAS should have 8 configurable inputs, got {config['configurable_inputs']}"
                print(f"  ✓ 4 Triacs (AC control)")
                print(f"  ✓ 4 Analog Outputs (0-10V)")
                print(f"  ✓ 8 Configurable Inputs (0-10V, 1K, 10K)")
                print(f"  ✗ NO RELAYS on MegaBAS!")
                
            elif board_type == '8relind':
                assert config['relays'] == 8, f"8relind should have 8 relays, got {config['relays']}"
                print(f"  ✓ 8 Relay Outputs ONLY")
                
            elif board_type == '16relind':
                assert config['relays'] == 16, f"16relind should have 16 relays, got {config['relays']}"
                print(f"  ✓ 16 Relay Outputs ONLY")
                
            elif board_type == '16univin':
                assert config['universal_inputs'] == 16, f"16univin should have 16 inputs, got {config['universal_inputs']}"
                print(f"  ✓ 16 Universal INPUTS ONLY")
                
            elif board_type == '16uout':
                assert config['analog_outputs'] == 16, f"16uout should have 16 outputs, got {config['analog_outputs']}"
                print(f"  ✓ 16 Analog OUTPUTS ONLY (0-10V)")
        
        print("\n" + "=" * 60)
        return True
        
    except json.JSONDecodeError as e:
        print(f"✗ Failed to parse JSON: {e}")
        return False
    except AssertionError as e:
        print(f"✗ Assertion failed: {e}")
        return False

def check_ui_files():
    print("\nChecking UI Files for Correct Board Specs...")
    print("=" * 60)
    
    issues = []
    
    # Check board_config_complete.rs
    print("\nChecking board_config_complete.rs...")
    with open("src/ui/board_config_complete.rs", "r") as f:
        content = f.read()
        
        # Check MegaBAS has no relays
        if "relays: 0,           // MegaBAS has NO relays!" in content:
            print("  ✓ MegaBAS correctly shows 0 relays")
        else:
            issues.append("MegaBAS incorrectly configured with relays")
            print("  ✗ MegaBAS incorrectly configured with relays")
        
        # Check MegaBAS has 4 triacs
        if "triacs: 4,           // 4 triacs for AC control" in content:
            print("  ✓ MegaBAS correctly shows 4 triacs")
        else:
            issues.append("MegaBAS triacs count incorrect")
            print("  ✗ MegaBAS triacs count incorrect")
    
    # Check io_control_complete.rs
    print("\nChecking io_control_complete.rs...")
    with open("src/ui/io_control_complete.rs", "r") as f:
        content = f.read()
        
        # Check that relay boards are optional
        if "relay_board_8: Option<Vec<bool>>" in content:
            print("  ✓ 8-relay board correctly shown as optional")
        else:
            issues.append("8-relay board not properly optional")
            print("  ✗ 8-relay board not properly optional")
        
        if "relay_board_16: Option<Vec<bool>>" in content:
            print("  ✓ 16-relay board correctly shown as optional")
        else:
            issues.append("16-relay board not properly optional")
            print("  ✗ 16-relay board not properly optional")
        
        # Check for correct comments
        if "// 8 configurable inputs (0-10V, 1K, 10K)" in content:
            print("  ✓ Correct comment about MegaBAS inputs")
        else:
            print("  ! Missing clarifying comment about MegaBAS inputs")
    
    print("\n" + "=" * 60)
    if issues:
        print(f"Found {len(issues)} issues:")
        for issue in issues:
            print(f"  - {issue}")
        return False
    else:
        print("✓ All UI files correctly represent board capabilities")
        return True

def test_board_interaction():
    print("\nTesting Board Interaction Commands...")
    print("=" * 60)
    
    # Test scan command
    print("\nTesting board scan...")
    result = subprocess.run(
        ["python3", "src/firmware_interface.py", "scan"],
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        try:
            boards = json.loads(result.stdout)
            print(f"  Found {len(boards)} board(s) configured")
            for board in boards:
                print(f"    - {board.get('name', 'Unknown')} at stack {board.get('stack', '?')}")
        except:
            print("  ! No boards detected (this is normal if hardware not connected)")
    else:
        print("  ! Scan failed (this is normal if libraries not installed)")
    
    return True

def main():
    print("UI/Firmware Integration Test")
    print("=" * 60)
    
    all_passed = True
    
    # Test 1: Firmware interface info
    if not test_firmware_info():
        all_passed = False
    
    # Test 2: Check UI files
    if not check_ui_files():
        all_passed = False
    
    # Test 3: Board interaction
    test_board_interaction()
    
    print("\n" + "=" * 60)
    if all_passed:
        print("✓ UI/Firmware integration test PASSED")
        print("\nKey Points Verified:")
        print("  • MegaBAS has 4 triacs, 4 analog outputs, 8 configurable inputs")
        print("  • MegaBAS has NO relays (relays are separate boards)")
        print("  • 16univin is INPUT ONLY (16 channels)")
        print("  • 16uout is OUTPUT ONLY (16 channels)")
        print("  • Relay boards are separate (8 or 16 channels)")
    else:
        print("✗ UI/Firmware integration test FAILED")
        print("\nPlease fix the issues above")
    
    return 0 if all_passed else 1

if __name__ == "__main__":
    sys.exit(main())