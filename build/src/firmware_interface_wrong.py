#!/usr/bin/env python3
"""
Comprehensive Firmware Interface for Sequent Microsystems Boards
Provides unified JSON API for all supported hardware
"""

import sys
import json
import time
import os
import subprocess
from typing import Dict, List, Any, Optional

# Add firmware paths to Python path
FIRMWARE_BASE = "/home/Automata/firmware"
sys.path.insert(0, os.path.join(FIRMWARE_BASE, "megabas-rpi/python"))
sys.path.insert(0, os.path.join(FIRMWARE_BASE, "8relind-rpi/python"))
sys.path.insert(0, os.path.join(FIRMWARE_BASE, "16relind-rpi/python"))
sys.path.insert(0, os.path.join(FIRMWARE_BASE, "16univin-rpi/python"))
sys.path.insert(0, os.path.join(FIRMWARE_BASE, "16uout-rpi/python"))

# Import all available board libraries
boards_available = {}

try:
    import megabas
    boards_available['megabas'] = megabas
except ImportError as e:
    print(f"Warning: megabas library not available: {e}", file=sys.stderr)

try:
    import lib8relind
    boards_available['8relind'] = lib8relind
except ImportError as e:
    print(f"Warning: lib8relind library not available: {e}", file=sys.stderr)

try:
    import SM16relind
    boards_available['16relind'] = SM16relind
except ImportError as e:
    print(f"Warning: SM16relind library not available: {e}", file=sys.stderr)

try:
    import lib16univin
    boards_available['16univin'] = lib16univin
except ImportError as e:
    print(f"Warning: lib16univin library not available: {e}", file=sys.stderr)

try:
    from SM16uout import SM16uout_data
    boards_available['16uout'] = SM16uout_data
except ImportError as e:
    print(f"Warning: SM16uout library not available: {e}", file=sys.stderr)


class SequentBoardInterface:
    """Unified interface for all Sequent Microsystems boards"""
    
    def __init__(self):
        self.boards = boards_available
        self.board_configs = {
            'megabas': {
                'analog_in_channels': 8,
                'analog_out_channels': 4,
                'triac_channels': 4,
                'contact_channels': 8,
                'has_rtc': True,
                'has_watchdog': True
            },
            '8relind': {
                'relay_channels': 8
            },
            '16relind': {
                'relay_channels': 16
            },
            '16univin': {
                'universal_in_channels': 16,
                'has_rtc': True,
                'has_rs485': True
            },
            '16uout': {
                'analog_out_channels': 16,
                'has_rs485': True
            }
        }
    
    def scan_boards(self) -> List[Dict[str, Any]]:
        """Scan for all connected boards"""
        found_boards = []
        
        # Try each stack level (0-7)
        for stack in range(8):
            # Check MegaBAS
            if 'megabas' in self.boards:
                try:
                    version = self.boards['megabas'].getVer(stack)
                    found_boards.append({
                        'type': 'megabas',
                        'stack': stack,
                        'name': f'MegaBAS Stack {stack}',
                        'version': version,
                        'config': self.board_configs['megabas']
                    })
                except:
                    pass
            
            # Check 8-relay board
            if '8relind' in self.boards:
                try:
                    # Try to read relay state to check if board exists
                    self.boards['8relind'].get(stack)
                    found_boards.append({
                        'type': '8relind',
                        'stack': stack,
                        'name': f'8-Relay Stack {stack}',
                        'version': 'Unknown',
                        'config': self.board_configs['8relind']
                    })
                except:
                    pass
            
            # Check 16-relay board
            if '16relind' in self.boards:
                try:
                    # Try to read relay state to check if board exists
                    relays = self.boards['16relind'].get(stack)
                    found_boards.append({
                        'type': '16relind',
                        'stack': stack,
                        'name': f'16-Relay Stack {stack}',
                        'version': 'Unknown',
                        'config': self.board_configs['16relind']
                    })
                except:
                    pass
            
            # Check 16 universal input board
            if '16univin' in self.boards:
                try:
                    # Try to read input to check if board exists
                    val = self.boards['16univin'].readU0_10In(stack, 1)
                    found_boards.append({
                        'type': '16univin',
                        'stack': stack,
                        'name': f'16-UnivIn Stack {stack}',
                        'version': 'Unknown',
                        'config': self.board_configs['16univin']
                    })
                except:
                    pass
            
            # Check 16 universal output board
            if '16uout' in self.boards:
                try:
                    # Try to read output to check if board exists
                    val = self.boards['16uout'].readU0_10Out(stack, 1)
                    found_boards.append({
                        'type': '16uout',
                        'stack': stack,
                        'name': f'16-UOut Stack {stack}',
                        'version': 'Unknown',
                        'config': self.board_configs['16uout']
                    })
                except:
                    pass
        
        return found_boards
    
    def get_megabas_status(self, stack: int) -> Dict[str, Any]:
        """Get complete status of MegaBAS board"""
        if 'megabas' not in self.boards:
            return {'error': 'MegaBAS library not available'}
        
        mb = self.boards['megabas']
        
        try:
            status = {
                'type': 'megabas',
                'stack': stack,
                'firmware': mb.getVer(stack),
                'analog_inputs': {},
                'analog_outputs': {},
                'triacs': {},
                'contacts': {},
                'sensors': {},
                'rtc': None,
                'watchdog': {}
            }
            
            # Read analog inputs (0-10V)
            for ch in range(1, 9):
                status['analog_inputs'][f'ch{ch}'] = {
                    'voltage': mb.getUIn(stack, ch),
                    'resistance_1k': mb.getRIn1K(stack, ch),
                    'resistance_10k': mb.getRIn10K(stack, ch)
                }
            
            # Read analog outputs
            for ch in range(1, 5):
                status['analog_outputs'][f'ch{ch}'] = mb.getUOut(stack, ch)
            
            # Read triacs
            triacs_state = mb.getTriacs(stack)
            for ch in range(1, 5):
                status['triacs'][f'ch{ch}'] = bool(triacs_state & (1 << (ch - 1)))
            
            # Read dry contacts
            contacts_state = mb.getContact(stack)
            for ch in range(1, 9):
                status['contacts'][f'ch{ch}'] = {
                    'state': mb.getContactCh(stack, ch),
                    'counter': mb.getContactCounter(stack, ch),
                    'edge_mode': mb.getContactCountEdge(stack, ch)
                }
            
            # Read system sensors
            status['sensors'] = {
                'power_supply_v': mb.getInVolt(stack),
                'raspberry_v': mb.getRaspVolt(stack),
                'cpu_temp_c': mb.getCpuTemp(stack)
            }
            
            # Read RTC
            try:
                rtc_data = mb.rtcGet(stack)
                status['rtc'] = {
                    'year': rtc_data[0],
                    'month': rtc_data[1],
                    'day': rtc_data[2],
                    'hour': rtc_data[3],
                    'minute': rtc_data[4],
                    'second': rtc_data[5]
                }
            except:
                pass
            
            # Read watchdog
            try:
                status['watchdog'] = {
                    'period': mb.wdtGetPeriod(stack),
                    'default_period': mb.wdtGetDefaultPeriod(stack),
                    'off_interval': mb.wdtGetOffInterval(stack),
                    'reset_count': mb.wdtGetResetCount(stack)
                }
            except:
                pass
            
            return status
            
        except Exception as e:
            return {'error': str(e)}
    
    def set_megabas_output(self, stack: int, output_type: str, channel: int, value: Any) -> Dict[str, Any]:
        """Set MegaBAS output (analog, triac)"""
        if 'megabas' not in self.boards:
            return {'error': 'MegaBAS library not available'}
        
        mb = self.boards['megabas']
        
        try:
            if output_type == 'analog':
                if channel < 1 or channel > 4:
                    return {'error': f'Invalid analog channel: {channel}'}
                if value < 0 or value > 10:
                    return {'error': f'Invalid voltage: {value}'}
                mb.setUOut(stack, channel, float(value))
                return {'success': True, 'channel': channel, 'value': value}
                
            elif output_type == 'triac':
                if channel < 1 or channel > 4:
                    return {'error': f'Invalid triac channel: {channel}'}
                mb.setTriac(stack, channel, int(value))
                return {'success': True, 'channel': channel, 'value': bool(value)}
                
            else:
                return {'error': f'Unknown output type: {output_type}'}
                
        except Exception as e:
            return {'error': str(e)}
    
    def get_relay_status(self, board_type: str, stack: int) -> Dict[str, Any]:
        """Get relay board status"""
        if board_type not in self.boards:
            return {'error': f'{board_type} library not available'}
        
        try:
            status = {
                'type': board_type,
                'stack': stack,
                'relays': {}
            }
            
            if board_type == '8relind':
                relay_state = self.boards[board_type].get(stack)
                for ch in range(1, 9):
                    status['relays'][f'ch{ch}'] = bool(relay_state & (1 << (ch - 1)))
                    
            elif board_type == '16relind':
                relay_state = self.boards[board_type].get(stack)
                for ch in range(1, 17):
                    status['relays'][f'ch{ch}'] = bool(relay_state & (1 << (ch - 1)))
            
            return status
            
        except Exception as e:
            return {'error': str(e)}
    
    def set_relay(self, board_type: str, stack: int, channel: int, state: bool) -> Dict[str, Any]:
        """Set relay state"""
        if board_type not in self.boards:
            return {'error': f'{board_type} library not available'}
        
        try:
            if board_type == '8relind':
                if channel < 1 or channel > 8:
                    return {'error': f'Invalid relay channel: {channel}'}
                self.boards[board_type].set(stack, channel, int(state))
                
            elif board_type == '16relind':
                if channel < 1 or channel > 16:
                    return {'error': f'Invalid relay channel: {channel}'}
                self.boards[board_type].set(stack, channel, int(state))
            
            return {'success': True, 'channel': channel, 'state': state}
            
        except Exception as e:
            return {'error': str(e)}
    
    def get_universal_input(self, stack: int, channel: int) -> Dict[str, Any]:
        """Read 16-channel universal input"""
        if '16univin' not in self.boards:
            return {'error': '16univin library not available'}
        
        try:
            if channel < 1 or channel > 16:
                return {'error': f'Invalid channel: {channel}'}
            
            value = self.boards['16univin'].readU0_10In(stack, channel)
            return {
                'success': True,
                'channel': channel,
                'voltage': value,
                'type': '0-10V'
            }
            
        except Exception as e:
            return {'error': str(e)}
    
    def set_universal_output(self, stack: int, channel: int, value: float) -> Dict[str, Any]:
        """Set 16-channel universal output"""
        if '16uout' not in self.boards:
            return {'error': '16uout library not available'}
        
        try:
            if channel < 1 or channel > 16:
                return {'error': f'Invalid channel: {channel}'}
            if value < 0 or value > 10:
                return {'error': f'Invalid voltage: {value}'}
            
            self.boards['16uout'].writeU0_10Out(stack, channel, value)
            return {
                'success': True,
                'channel': channel,
                'value': value
            }
            
        except Exception as e:
            return {'error': str(e)}
    
    def emergency_stop(self) -> Dict[str, Any]:
        """Emergency stop - turn off all outputs on all boards"""
        results = []
        
        # Scan for all boards
        boards = self.scan_boards()
        
        for board in boards:
            stack = board['stack']
            board_type = board['type']
            
            try:
                if board_type == 'megabas':
                    # Turn off all triacs
                    for ch in range(1, 5):
                        self.boards['megabas'].setTriac(stack, ch, 0)
                    # Set all analog outputs to 0
                    for ch in range(1, 5):
                        self.boards['megabas'].setUOut(stack, ch, 0)
                    results.append({'board': f'megabas_{stack}', 'status': 'stopped'})
                    
                elif board_type == '8relind':
                    # Turn off all relays
                    self.boards['8relind'].set_all(stack, 0)
                    results.append({'board': f'8relind_{stack}', 'status': 'stopped'})
                    
                elif board_type == '16relind':
                    # Turn off all relays
                    self.boards['16relind'].set_all(stack, 0)
                    results.append({'board': f'16relind_{stack}', 'status': 'stopped'})
                    
                elif board_type == '16uout':
                    # Set all outputs to 0
                    for ch in range(1, 17):
                        self.boards['16uout'].writeU0_10Out(stack, ch, 0)
                    results.append({'board': f'16uout_{stack}', 'status': 'stopped'})
                    
            except Exception as e:
                results.append({'board': f'{board_type}_{stack}', 'error': str(e)})
        
        return {'emergency_stop': True, 'results': results}


def main():
    """Main entry point for command-line interface"""
    if len(sys.argv) < 2:
        print(json.dumps({'error': 'No command specified'}))
        sys.exit(1)
    
    interface = SequentBoardInterface()
    command = sys.argv[1]
    
    try:
        if command == 'scan':
            result = interface.scan_boards()
            
        elif command == 'status':
            if len(sys.argv) < 4:
                result = {'error': 'Usage: status <board_type> <stack>'}
            else:
                board_type = sys.argv[2]
                stack = int(sys.argv[3])
                
                if board_type == 'megabas':
                    result = interface.get_megabas_status(stack)
                elif board_type in ['8relind', '16relind']:
                    result = interface.get_relay_status(board_type, stack)
                else:
                    result = {'error': f'Unknown board type: {board_type}'}
        
        elif command == 'set_output':
            if len(sys.argv) < 6:
                result = {'error': 'Usage: set_output <stack> <type> <channel> <value>'}
            else:
                stack = int(sys.argv[2])
                output_type = sys.argv[3]
                channel = int(sys.argv[4])
                value = float(sys.argv[5]) if output_type == 'analog' else int(sys.argv[5])
                result = interface.set_megabas_output(stack, output_type, channel, value)
        
        elif command == 'set_relay':
            if len(sys.argv) < 6:
                result = {'error': 'Usage: set_relay <board_type> <stack> <channel> <state>'}
            else:
                board_type = sys.argv[2]
                stack = int(sys.argv[3])
                channel = int(sys.argv[4])
                state = bool(int(sys.argv[5]))
                result = interface.set_relay(board_type, stack, channel, state)
        
        elif command == 'read_univin':
            if len(sys.argv) < 4:
                result = {'error': 'Usage: read_univin <stack> <channel>'}
            else:
                stack = int(sys.argv[2])
                channel = int(sys.argv[3])
                result = interface.get_universal_input(stack, channel)
        
        elif command == 'set_univout':
            if len(sys.argv) < 5:
                result = {'error': 'Usage: set_univout <stack> <channel> <value>'}
            else:
                stack = int(sys.argv[2])
                channel = int(sys.argv[3])
                value = float(sys.argv[4])
                result = interface.set_universal_output(stack, channel, value)
        
        elif command == 'emergency_stop':
            result = interface.emergency_stop()
        
        else:
            result = {'error': f'Unknown command: {command}'}
        
        print(json.dumps(result, indent=2))
        
    except Exception as e:
        print(json.dumps({'error': str(e)}))
        sys.exit(1)


if __name__ == '__main__':
    main()