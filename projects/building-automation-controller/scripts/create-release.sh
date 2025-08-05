#!/bin/bash
# Create pre-compiled release packages for Raspberry Pi 4
# This creates a distributable tarball with all binaries pre-compiled

set -e

echo "=== Automata Nexus Release Builder ==="
echo "Creating pre-compiled release package..."

# Configuration
VERSION="1.0.0"
ARCH="aarch64"
RELEASE_DIR="release-pi4-${VERSION}"
RELEASE_NAME="automata-nexus-pi4-${VERSION}-precompiled.tar.gz"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if we're in the right directory
if [ ! -f "package.json" ] || [ ! -d "src-tauri" ]; then
    echo "Error: Must run from project root directory"
    exit 1
fi

# Clean previous release
rm -rf "${RELEASE_DIR}"
mkdir -p "${RELEASE_DIR}"

echo -e "${YELLOW}Step 1/6: Building frontend...${NC}"
# Build the Next.js frontend
npm install
npm run build

echo -e "${YELLOW}Step 2/6: Building Rust backend...${NC}"
# Build the Rust backend
cd src-tauri
cargo build --release --target aarch64-unknown-linux-gnu
cd ..

echo -e "${YELLOW}Step 3/6: Preparing release directory...${NC}"
# Create directory structure
mkdir -p "${RELEASE_DIR}/app"
mkdir -p "${RELEASE_DIR}/installer"
mkdir -p "${RELEASE_DIR}/scripts"
mkdir -p "${RELEASE_DIR}/drivers"

# Copy application files (excluding development files)
echo "Copying application files..."
rsync -av --progress \
    --exclude 'node_modules' \
    --exclude '.git' \
    --exclude '.next' \
    --exclude 'target' \
    --exclude '*.log' \
    --exclude '__pycache__' \
    --exclude '.env*' \
    . "${RELEASE_DIR}/app/"

# Copy pre-built frontend
echo "Copying pre-built frontend..."
cp -r out "${RELEASE_DIR}/app/"
cp -r .next "${RELEASE_DIR}/app/"

# Copy pre-built Rust binary
echo "Copying pre-built binary..."
mkdir -p "${RELEASE_DIR}/app/src-tauri/target/aarch64-unknown-linux-gnu/release"
cp src-tauri/target/aarch64-unknown-linux-gnu/release/building-automation-controller \
   "${RELEASE_DIR}/app/src-tauri/target/aarch64-unknown-linux-gnu/release/"

# Strip debug symbols to reduce size
aarch64-linux-gnu-strip "${RELEASE_DIR}/app/src-tauri/target/aarch64-unknown-linux-gnu/release/building-automation-controller"

echo -e "${YELLOW}Step 4/6: Creating quick installer...${NC}"
# Create a quick installer that skips compilation
cat > "${RELEASE_DIR}/quick-install.py" << 'EOF'
#!/usr/bin/env python3
"""
Automata Nexus Quick Installer - Pre-compiled Version
Installs in minutes instead of 40+ minutes!
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path

class QuickInstaller:
    def __init__(self):
        if os.geteuid() != 0:
            print("ERROR: This installer must be run as root (sudo)")
            sys.exit(1)
            
        self.install_path = "/opt/automata-nexus"
        self.service_name = "automata-nexus-control-center"
        
    def log(self, message):
        print(f"[INSTALL] {message}")
        
    def run_command(self, cmd):
        if isinstance(cmd, list):
            cmd_str = " ".join(cmd)
        else:
            cmd_str = cmd
        print(f"[RUN] {cmd_str}")
        result = subprocess.run(cmd_str, shell=True, capture_output=True, text=True)
        if result.returncode != 0:
            print(f"[ERROR] {result.stderr}")
            raise Exception(f"Command failed: {cmd_str}")
        return result.stdout
        
    def install(self):
        print("=== Automata Nexus Quick Installer ===")
        print("Using pre-compiled binaries for fast installation!")
        print("")
        
        # System updates and dependencies
        self.log("Updating system packages...")
        self.run_command("apt-get update -y")
        
        self.log("Installing runtime dependencies...")
        deps = [
            "i2c-tools", "python3", "python3-pip", "python3-smbus",
            "libwebkit2gtk-4.0-37", "libgtk-3-0", "libayatana-appindicator3-1"
        ]
        self.run_command(f"apt-get install -y {' '.join(deps)}")
        
        # Python libraries
        self.log("Installing Python libraries...")
        libs = ["SMmegabas", "SM16relind", "SM16univin", "SM16uout", "SM8relind", "requests", "pyserial"]
        for lib in libs:
            self.run_command(f"pip3 install {lib}")
            
        # Enable I2C
        self.log("Enabling I2C interface...")
        self.run_command("raspi-config nonint do_i2c 0")
        
        # Install Sequent drivers
        self.log("Installing hardware drivers...")
        driver_repos = [
            ("megabas-rpi", "https://github.com/SequentMicrosystems/megabas-rpi.git"),
            ("16univin-rpi", "https://github.com/SequentMicrosystems/16univin-rpi.git"),
            ("16relind-rpi", "https://github.com/SequentMicrosystems/16relind-rpi.git"),
            ("8relind-rpi", "https://github.com/SequentMicrosystems/8relind-rpi.git"),
            ("16uout-rpi", "https://github.com/SequentMicrosystems/16uout-rpi.git")
        ]
        
        os.makedirs(f"{self.install_path}/drivers", exist_ok=True)
        for name, url in driver_repos:
            if not os.path.exists(f"{self.install_path}/drivers/{name}"):
                self.run_command(f"git clone {url} {self.install_path}/drivers/{name}")
                self.run_command(f"cd {self.install_path}/drivers/{name} && make install")
                
        # Copy pre-built application
        self.log("Installing pre-built application...")
        if os.path.exists(self.install_path + "/app"):
            shutil.rmtree(self.install_path + "/app")
        shutil.copytree("app", self.install_path + "/app")
        
        # Make binary executable
        binary_path = f"{self.install_path}/app/src-tauri/target/aarch64-unknown-linux-gnu/release/building-automation-controller"
        os.chmod(binary_path, 0o755)
        
        # Create systemd service
        self.log("Creating systemd service...")
        service_content = f"""[Unit]
Description=Automata Nexus Automation Control Center
After=network.target

[Service]
Type=simple
User=Automata
Group=Automata
WorkingDirectory={self.install_path}/app
ExecStart={binary_path}
Restart=always
RestartSec=10
Environment="RUST_LOG=info"
Environment="WEBKIT_DISABLE_COMPOSITING_MODE=1"
SupplementaryGroups=i2c dialout
PrivateDevices=no

[Install]
WantedBy=multi-user.target
"""
        
        with open(f"/etc/systemd/system/{self.service_name}.service", "w") as f:
            f.write(service_content)
            
        self.run_command("systemctl daemon-reload")
        
        # Setup user and permissions
        self.log("Setting up permissions...")
        try:
            self.run_command("useradd -r -s /bin/false Automata")
        except:
            pass
        self.run_command("usermod -a -G i2c,gpio,dialout Automata")
        self.run_command(f"chown -R Automata:Automata {self.install_path}")
        
        print("\n" + "="*50)
        print("Installation completed successfully!")
        print("="*50)
        print(f"\nStart the service: sudo systemctl start {self.service_name}")
        print(f"Enable auto-start: sudo systemctl enable {self.service_name}")
        print("\nAccess the web interface at: http://localhost:1420")
        print("\nTotal installation time: ~5 minutes!")
        
if __name__ == "__main__":
    installer = QuickInstaller()
    installer.install()
EOF

chmod +x "${RELEASE_DIR}/quick-install.py"

echo -e "${YELLOW}Step 5/6: Creating release info...${NC}"
# Create release info
cat > "${RELEASE_DIR}/README.md" << EOF
# Automata Nexus Control Center - Pre-compiled Release

Version: ${VERSION}
Architecture: Raspberry Pi 4 (${ARCH})
Build Date: $(date)

## Quick Installation (5 minutes!)

This is a pre-compiled release that installs in minutes instead of 40+ minutes.

\`\`\`bash
sudo python3 quick-install.py
\`\`\`

## What's Included

- Pre-built Next.js frontend
- Pre-compiled Rust binary for ARM64
- All necessary configuration files
- Quick installer script

## System Requirements

- Raspberry Pi 4 (2GB+ RAM recommended)
- Raspberry Pi OS Bullseye 64-bit
- Internet connection (for downloading drivers)

## Manual Installation

If you prefer to compile from source (40+ minutes):
\`\`\`bash
cd installer
sudo python3 install-automata-nexus.py
\`\`\`

## Features

- Universal I/O Control (0-10V, 4-20mA, digital)
- BMS Integration with InfluxDB
- SQLite metrics database with 7-day retention
- Real-time trend analysis
- Maintenance mode
- JavaScript logic engine
- Vibration monitoring (RS485/Modbus)
- HVAC refrigerant diagnostics
- P499 pressure transducer support

## Support

For issues or questions, visit: https://github.com/AutomataControls/Development
EOF

# Create version file
echo "${VERSION}" > "${RELEASE_DIR}/VERSION"

echo -e "${YELLOW}Step 6/6: Creating release archive...${NC}"
# Create the release archive
tar -czf "${RELEASE_NAME}" "${RELEASE_DIR}"

# Calculate sizes
ORIGINAL_SIZE=$(du -sh . | cut -f1)
RELEASE_SIZE=$(du -sh "${RELEASE_NAME}" | cut -f1)

echo -e "${GREEN}=== Release Created Successfully ===${NC}"
echo "Release package: ${RELEASE_NAME}"
echo "Size: ${RELEASE_SIZE} (original source: ${ORIGINAL_SIZE})"
echo ""
echo "To distribute:"
echo "1. Upload ${RELEASE_NAME} to GitHub Releases"
echo "2. Users download and extract: tar -xzf ${RELEASE_NAME}"
echo "3. Users run: sudo python3 ${RELEASE_DIR}/quick-install.py"
echo ""
echo "Installation time reduced from 40+ minutes to ~5 minutes!"

# Cleanup
rm -rf "${RELEASE_DIR}"