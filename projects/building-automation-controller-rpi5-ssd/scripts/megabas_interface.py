#!/usr/bin/env python3
"""
Unified interface for Sequent Microsystems building automation hardware
Provides JSON API for all supported boards
"""

import sys
import json
import time

# Import all Sequent Microsystems libraries
try:
    import megabas
except ImportError:
    print(json.dumps({"error": "megabas library not installed"}))
    sys.exit(1)

# Try to import expansion board libraries
expansion_libs = {}
try:
    import lib8relind
    expansion_libs['8relay'] = lib8relind
except:
    pass

try:
    import SM16relind
    expansion_libs['16relay'] = SM16relind.SM16relind
except:
    pass

try:
    import lib16univin
    expansion_libs['16univin'] = lib16univin.SM16univin
except:
    pass

try:
    import SM16uout.SM16uout as SM16uout
    expansion_libs['16uout'] = SM16uout
except:
    pass


def get_megabas_status(stack=0):
    """Get complete status of MegaBAS board"""
    try:
        status = {
            "type": "megabas",
            "stack": stack,
            "firmware": megabas.getVer(stack),
            "analog_inputs": {},
            "analog_outputs": {},
            "triacs": {},
            "contacts": {},
            "sensors": {},
            "rtc": None,
            "watchdog": {}
        }
        
        # Read analog inputs
        for ch in range(1, 9):
            status["analog_inputs"][f"ch{ch}"] = {
                "voltage": megabas.getUIn(stack, ch),
                "r1k": megabas.getRIn1K(stack, ch),
                "r10k": megabas.getRIn10K(stack, ch)
            }
        
        # Read analog outputs
        for ch in range(1, 5):
            status["analog_outputs"][f"ch{ch}"] = megabas.getUOut(stack, ch)
        
        # Read triacs
        triacs_state = megabas.getTriacs(stack)
        for ch in range(1, 5):
            status["triacs"][f"ch{ch}"] = bool(triacs_state & (1 << (ch - 1)))
        
        # Read dry contacts
        contacts_state = megabas.getContact(stack)
        for ch in range(1, 5):
            status["contacts"][f"ch{ch}"] = {
                "state": megabas.getContactCh(stack, ch),
                "counter": megabas.getContactCounter(stack, ch),
                "edge_mode": megabas.getContactCountEdge(stack, ch)
            }
        
        # Read sensors
        status["sensors"] = {
            "power_supply_v": megabas.getInVolt(stack),
            "raspberry_v": megabas.getRaspVolt(stack),
            "cpu_temp_c": megabas.getCpuTemp(stack)
        }
        
        # Read RTC
        try:
            rtc_data = megabas.rtcGet(stack)
            status["rtc"] = {
                "year": rtc_data[0],
                "month": rtc_data[1],
                "day": rtc_data[2],
                "hour": rtc_data[3],
                "minute": rtc_data[4],
                "second": rtc_data[5]
            }
        except:
            pass
        
        # Read watchdog
        status["watchdog"] = {
            "period": megabas.wdtGetPeriod(stack),
            "default_period": megabas.wdtGetDefaultPeriod(stack),
            "off_interval": megabas.wdtGetOffInterval(stack),
            "reset_count": megabas.wdtGetResetCount(stack)
        }
        
        return status
        
    except Exception as e:
        return {"error": str(e), "type": "megabas", "stack": stack}


def get_16relay_status(stack):
    """Get status of 16-relay board"""
    try:
        board = expansion_libs['16relay'](stack)
        relays = board.get_all()
        
        status = {
            "type": "16relay",
            "stack": stack,
            "relays": {}
        }
        
        for i in range(16):
            status["relays"][f"ch{i+1}"] = bool(relays & (1 << i))
        
        return status
        
    except Exception as e:
        return {"error": str(e), "type": "16relay", "stack": stack}


def get_8relay_status(stack):
    """Get status of 8-relay board"""
    try:
        relays = expansion_libs['8relay'].get_all(stack)
        
        status = {
            "type": "8relay",
            "stack": stack,
            "relays": {}
        }
        
        for i in range(8):
            status["relays"][f"ch{i+1}"] = bool(relays & (1 << i))
        
        return status
        
    except Exception as e:
        return {"error": str(e), "type": "8relay", "stack": stack}


def get_16univin_status(stack):
    """Get status of 16 universal input board"""
    try:
        board = expansion_libs['16univin'](stack)
        
        status = {
            "type": "16univin",
            "stack": stack,
            "firmware": board.get_version(),
            "inputs": {}
        }
        
        for ch in range(1, 17):
            status["inputs"][f"ch{ch}"] = {
                "voltage": board.get_u_in(ch),
                "r1k": board.get_r1k_in(ch),
                "r10k": board.get_r10k_in(ch),
                "digital": board.get_dig_in(ch),
                "counter": board.get_dig_in_counter(ch),
                "count_enabled": board.get_dig_in_cnt_en(ch)
            }
        
        return status
        
    except Exception as e:
        return {"error": str(e), "type": "16univin", "stack": stack}


def get_16uout_status(stack):
    """Get status of 16 analog output board"""
    try:
        board = expansion_libs['16uout']()
        board.stack = stack
        
        status = {
            "type": "16uout",
            "stack": stack,
            "firmware": board.get_version(),
            "outputs": {},
            "calibration": board.calib_status()
        }
        
        for ch in range(1, 17):
            status["outputs"][f"ch{ch}"] = board.get_u_out(ch)
        
        return status
        
    except Exception as e:
        return {"error": str(e), "type": "16uout", "stack": stack}


def set_megabas_output(stack, output_type, channel, value):
    """Set MegaBAS output"""
    try:
        if output_type == "analog":
            megabas.setUOut(stack, channel, float(value))
        elif output_type == "triac":
            megabas.setTriac(stack, channel, int(value))
        return {"success": True}
    except Exception as e:
        return {"error": str(e)}


def set_relay(board_type, stack, channel, value):
    """Set relay state"""
    try:
        if board_type == "16relay":
            board = expansion_libs['16relay'](stack)
            board.set(channel, int(value))
        elif board_type == "8relay":
            expansion_libs['8relay'].set(stack, channel, int(value))
        return {"success": True}
    except Exception as e:
        return {"error": str(e)}


def set_16uout(stack, channel, value):
    """Set 16 analog output"""
    try:
        board = expansion_libs['16uout']()
        board.stack = stack
        board.set_u_out(channel, float(value))
        return {"success": True}
    except Exception as e:
        return {"error": str(e)}


def scan_all_boards():
    """Scan for all connected boards"""
    boards = []
    
    # Always check MegaBAS at stack 0
    try:
        megabas.getVer(0)
        boards.append({"type": "megabas", "stack": 0})
    except:
        pass
    
    # Scan other stack levels for expansion boards
    for stack in range(1, 8):
        # Try each board type
        for board_type, lib in expansion_libs.items():
            try:
                if board_type == '16relay':
                    board = lib(stack)
                    board.get_all()
                elif board_type == '8relay':
                    lib.get_all(stack)
                elif board_type == '16univin':
                    board = lib(stack)
                    board.get_version()
                elif board_type == '16uout':
                    board = lib()
                    board.stack = stack
                    board.get_version()
                
                boards.append({"type": board_type, "stack": stack})
                break  # Found a board at this stack level
            except:
                continue
    
    return boards


# Command line interface
if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(json.dumps({"error": "No command specified"}))
        sys.exit(1)
    
    command = sys.argv[1]
    
    try:
        if command == "scan":
            result = scan_all_boards()
            print(json.dumps(result))
        
        elif command == "status":
            if len(sys.argv) < 4:
                print(json.dumps({"error": "Missing board type and stack"}))
                sys.exit(1)
            
            board_type = sys.argv[2]
            stack = int(sys.argv[3])
            
            if board_type == "megabas":
                result = get_megabas_status(stack)
            elif board_type == "16relay":
                result = get_16relay_status(stack)
            elif board_type == "8relay":
                result = get_8relay_status(stack)
            elif board_type == "16univin":
                result = get_16univin_status(stack)
            elif board_type == "16uout":
                result = get_16uout_status(stack)
            else:
                result = {"error": f"Unknown board type: {board_type}"}
            
            print(json.dumps(result))
        
        elif command == "set":
            if len(sys.argv) < 6:
                print(json.dumps({"error": "Missing parameters"}))
                sys.exit(1)
            
            board_type = sys.argv[2]
            stack = int(sys.argv[3])
            channel = int(sys.argv[4])
            value = sys.argv[5]
            
            if board_type == "megabas-analog":
                result = set_megabas_output(stack, "analog", channel, value)
            elif board_type == "megabas-triac":
                result = set_megabas_output(stack, "triac", channel, value)
            elif board_type in ["16relay", "8relay"]:
                result = set_relay(board_type, stack, channel, value)
            elif board_type == "16uout":
                result = set_16uout(stack, channel, value)
            else:
                result = {"error": f"Unknown board type: {board_type}"}
            
            print(json.dumps(result))
        
        else:
            print(json.dumps({"error": f"Unknown command: {command}"}))
            
    except Exception as e:
        print(json.dumps({"error": str(e)}))
        sys.exit(1)