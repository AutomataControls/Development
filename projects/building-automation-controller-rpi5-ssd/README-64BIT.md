# Automata Nexus Automation Control Center - 64-bit Raspberry Pi OS Support

This application is fully compatible with 64-bit Raspberry Pi OS Bullseye.

## System Requirements

- Raspberry Pi 3B+, 4, or newer (64-bit capable)
- Raspberry Pi OS Bullseye 64-bit (Debian 11)
- 2GB RAM minimum, 4GB+ recommended
- Sequent Microsystems MegaBAS HAT and expansion boards

## 64-bit Compatibility

The application has been optimized for 64-bit ARM architecture (aarch64):

1. **Native 64-bit Binary**: Compiled specifically for aarch64-unknown-linux-gnu target
2. **Optimized Performance**: Takes advantage of 64-bit registers and instructions
3. **Better Memory Management**: Can address more than 4GB of RAM
4. **WebKit Compatibility**: Full support for modern web standards in Tauri

## Building for 64-bit

### Prerequisites

1. Install Rust toolchain for aarch64:
   ```bash
   ./scripts/setup-rust-aarch64.sh
   ```

2. Install Node.js dependencies:
   ```bash
   npm install
   ```

### Build and Deploy

Deploy to 64-bit Raspberry Pi:
```bash
./scripts/deploy-to-pi.sh pi@raspberrypi.local release aarch64
```

## Python Library Compatibility

All Sequent Microsystems Python libraries work on 64-bit Bullseye:

- `megabas` - Building Automation HAT
- `SM16relind` - 16-channel relay board
- `SM16univin` - 16-channel universal input
- `SM16uout` - 16-channel analog output
- `SM8relind` - 8-channel relay board

These libraries use I2C communication which is architecture-independent.

## Performance Benefits on 64-bit

1. **Faster JavaScript Execution**: V8 engine runs more efficiently on 64-bit
2. **Better Memory Usage**: Can utilize full system RAM for data processing
3. **Improved I/O Performance**: 64-bit syscalls are more efficient
4. **Future-Proof**: ARM is moving towards 64-bit only support

## Troubleshooting 64-bit Issues

### Check Architecture
```bash
uname -m
# Should output: aarch64
```

### Verify 64-bit OS
```bash
getconf LONG_BIT
# Should output: 64
```

### I2C Permissions
```bash
# Add user to i2c group
sudo usermod -a -G i2c $USER
# Logout and login again
```

### WebKit Rendering
If you experience rendering issues:
```bash
export WEBKIT_DISABLE_COMPOSITING_MODE=1
```

## Tested Configurations

✅ Raspberry Pi 4B (8GB) + Bullseye 64-bit + MegaBAS HAT
✅ Raspberry Pi 4B (4GB) + Bullseye 64-bit + Multiple expansion boards
✅ Raspberry Pi 3B+ + Bullseye 64-bit + MegaBAS HAT
✅ Raspberry Pi CM4 + Bullseye 64-bit + Custom carrier board

## Migration from 32-bit

If migrating from 32-bit Bullseye:

1. Backup your configuration files
2. Flash 64-bit Bullseye to SD card
3. Run setup script: `./setup-pi.sh`
4. Deploy the application
5. Restore configuration files

The application data format is identical between 32-bit and 64-bit versions.