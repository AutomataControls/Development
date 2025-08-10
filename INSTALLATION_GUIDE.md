# Nexus Controller Installation Guide

## Complete Native Rust Implementation for Raspberry Pi 5

### Table of Contents
1. [System Requirements](#system-requirements)
2. [Pre-Installation Checklist](#pre-installation-checklist)
3. [Installation Methods](#installation-methods)
4. [Step-by-Step Installation](#step-by-step-installation)
5. [Post-Installation Testing](#post-installation-testing)
6. [Troubleshooting](#troubleshooting)
7. [Performance Optimization](#performance-optimization)
8. [Security Configuration](#security-configuration)

---

## System Requirements

### Hardware Requirements
- **Raspberry Pi 5** (4GB or 8GB RAM recommended)
- **NVMe SSD** (recommended) or high-quality SD card
- **Power Supply**: Official RPi5 27W USB-C power adapter
- **Cooling**: Active cooling recommended for production use
- **Network**: Ethernet connection recommended

### Software Requirements
- **Operating System**: Raspberry Pi OS 64-bit (Bookworm)
- **Rust**: 1.75.0 or newer (installer will handle this)
- **Python**: 3.9 or newer
- **Storage**: Minimum 10GB free space for build

### Optional Hardware
- Sequent Microsystems boards (MegaBAS, MegaIND, etc.)
- WTVB01-485 vibration sensors
- P499 pressure transducers
- RS485 USB adapter

---

## Pre-Installation Checklist

Before starting installation, ensure:

- [ ] Raspberry Pi OS 64-bit is installed and updated
- [ ] Internet connection is active
- [ ] SSH is enabled (if installing remotely)
- [ ] At least 10GB free disk space
- [ ] System date/time is correct
- [ ] You have sudo/root access

---

## Installation Methods

### Method 1: Automated Installation (Recommended)

The easiest way to install the Nexus Controller:

```bash
# Download the installer package
git clone https://github.com/your-repo/nexus-controller.git
cd nexus-controller/installer

# Run pre-flight check
sudo bash preflight_check.sh

# If all checks pass, run the installer
sudo bash install_nexus_complete.sh
```

### Method 2: Manual Installation

For advanced users who want more control:

```bash
# Use the Makefile
cd nexus-controller

# Check dependencies
make check

# Build the application
make build

# Install system-wide
sudo make install
```

### Method 3: Quick Development Setup

For development and testing:

```bash
# Build and run locally
cd nexus-controller
cargo build --release
./target/release/nexus-controller
```

---

## Step-by-Step Installation

### Step 1: Prepare Your Raspberry Pi 5

```bash
# Update system packages
sudo apt-get update
sudo apt-get upgrade -y

# Enable hardware interfaces
sudo raspi-config nonint do_i2c 0
sudo raspi-config nonint do_spi 0

# Reboot to apply changes
sudo reboot
```

### Step 2: Download the Nexus Controller

```bash
# Clone the repository
git clone https://github.com/your-repo/nexus-controller.git
cd nexus-controller

# Or download the release package
wget https://github.com/your-repo/nexus-controller/releases/latest/nexus-controller.tar.gz
tar -xzf nexus-controller.tar.gz
cd nexus-controller
```

### Step 3: Run Pre-Installation Check

```bash
cd installer
sudo bash preflight_check.sh
```

This will verify:
- System architecture (ARM64)
- Available memory and disk space
- Required build tools
- Network connectivity
- Hardware interfaces

**If any ERRORS are shown**, fix them before proceeding:

```bash
# Quick fix for common issues
sudo bash quick_fix.sh
```

### Step 4: Run the Installer

```bash
# Full installation (15-30 minutes)
sudo bash install_nexus_complete.sh
```

The installer will:
1. Install all system dependencies
2. Install and configure Rust toolchain
3. Install Python libraries
4. Install hardware libraries
5. Build the Nexus Controller
6. Configure the database
7. Set up systemd services
8. Configure automatic backups
9. Optimize system for RPi5

### Step 5: Verify Installation

```bash
# Run the test suite
sudo bash test_installation.sh
```

All critical tests should PASS. Optional features may show as SKIP.

### Step 6: Start the Service

```bash
# Enable and start the service
sudo systemctl enable nexus-controller
sudo systemctl start nexus-controller

# Check status
sudo systemctl status nexus-controller

# View logs
sudo journalctl -u nexus-controller -f
```

### Step 7: Access the Controller

Open a web browser and navigate to:
- Local: `http://localhost:8080`
- Network: `http://<raspberry-pi-ip>:8080`

Default credentials:
- PIN: `1234` (change immediately after first login)

---

## Post-Installation Testing

### Test 1: Service Health

```bash
# Check if service is running
sudo systemctl is-active nexus-controller

# Check resource usage
htop
```

### Test 2: Database Connection

```bash
# Test database
sqlite3 /opt/automata-nexus/data/nexus.db "SELECT * FROM settings;"
```

### Test 3: Hardware Access

```bash
# Test I2C
sudo i2cdetect -y 1

# Test GPIO (if applicable)
gpio readall
```

### Test 4: Network Connectivity

```bash
# Test API endpoint
curl -I http://localhost:8080/api/health
```

---

## Troubleshooting

### Common Issues and Solutions

#### Issue: Build fails with "out of memory"

**Solution**: Add swap space
```bash
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

#### Issue: "Permission denied" errors

**Solution**: Fix permissions
```bash
sudo chown -R pi:pi /opt/automata-nexus
sudo chmod +x /opt/automata-nexus/*.sh
```

#### Issue: Service won't start

**Solution**: Check logs and configuration
```bash
# View detailed logs
sudo journalctl -u nexus-controller -n 100 --no-pager

# Check configuration
cat /opt/automata-nexus/config/nexus.toml

# Validate binary
/opt/automata-nexus/nexus-controller --version
```

#### Issue: Cannot access web interface

**Solution**: Check firewall and port
```bash
# Check if port is listening
sudo netstat -tuln | grep 8080

# Allow port in firewall (if using ufw)
sudo ufw allow 8080
```

#### Issue: Hardware not detected

**Solution**: Enable interfaces
```bash
# Enable I2C and SPI
sudo raspi-config nonint do_i2c 0
sudo raspi-config nonint do_spi 0
sudo reboot
```

### Getting Help

If issues persist:

1. Check the installation log:
   ```bash
   cat /tmp/nexus_install_*.log
   ```

2. Run diagnostics:
   ```bash
   sudo bash test_installation.sh > diagnostic_report.txt 2>&1
   ```

3. Report issues with diagnostic information

---

## Performance Optimization

### CPU Optimization

```bash
# Set performance governor
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Make permanent
echo 'GOVERNOR="performance"' | sudo tee /etc/default/cpufrequtils
```

### NVMe Optimization

```bash
# Enable PCIe Gen 3 (add to /boot/firmware/config.txt)
dtparam=pciex1
dtparam=pciex1_gen=3

# Optimize mount options (add to /etc/fstab)
/dev/nvme0n1p2 / ext4 defaults,noatime,commit=600,errors=remount-ro 0 1
```

### Memory Optimization

```bash
# Optimize kernel parameters
sudo tee /etc/sysctl.d/99-nexus.conf << EOF
vm.swappiness = 10
vm.vfs_cache_pressure = 50
vm.dirty_background_ratio = 5
vm.dirty_ratio = 10
EOF

sudo sysctl -p /etc/sysctl.d/99-nexus.conf
```

### Database Optimization

```bash
# Optimize SQLite for NVMe
sqlite3 /opt/automata-nexus/data/nexus.db << EOF
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;
PRAGMA temp_store = MEMORY;
VACUUM;
EOF
```

---

## Security Configuration

### Change Default PIN

1. Access the Admin Panel
2. Navigate to Security Settings
3. Change the default PIN (1234) immediately

### Enable HTTPS (Optional)

```bash
# Install nginx as reverse proxy
sudo apt-get install nginx certbot python3-certbot-nginx

# Configure nginx
sudo tee /etc/nginx/sites-available/nexus << EOF
server {
    listen 80;
    server_name your-domain.com;
    
    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
    }
}
EOF

sudo ln -s /etc/nginx/sites-available/nexus /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx

# Get SSL certificate
sudo certbot --nginx -d your-domain.com
```

### Firewall Configuration

```bash
# Install and configure ufw
sudo apt-get install ufw

# Allow SSH (if using)
sudo ufw allow 22

# Allow Nexus Controller
sudo ufw allow 8080

# Allow HTTPS (if configured)
sudo ufw allow 443

# Enable firewall
sudo ufw enable
```

### Regular Updates

```bash
# Update system packages weekly
sudo apt-get update && sudo apt-get upgrade

# Update Nexus Controller
cd /opt/automata-nexus/src
git pull
cargo build --release
sudo systemctl restart nexus-controller
```

---

## Maintenance

### Backup Strategy

Automatic backups are configured during installation:
- Daily backups at 2 AM
- 7-day retention
- Stored in `/opt/automata-nexus/backups/`

### Manual Backup

```bash
sudo /opt/automata-nexus/backup.sh
```

### Restore from Backup

```bash
# Stop service
sudo systemctl stop nexus-controller

# Restore database
cd /opt/automata-nexus/data
sudo tar -xzf ../backups/backup_TIMESTAMP.tar.gz

# Start service
sudo systemctl start nexus-controller
```

### Log Management

Logs are automatically rotated. View logs:

```bash
# Service logs
sudo journalctl -u nexus-controller

# Application logs
tail -f /opt/automata-nexus/logs/nexus.log

# Monitor logs
tail -f /opt/automata-nexus/logs/monitor.log
```

---

## Uninstallation

To completely remove Nexus Controller:

```bash
# Stop and disable services
sudo systemctl stop nexus-controller
sudo systemctl disable nexus-controller
sudo systemctl stop nexus-monitor
sudo systemctl disable nexus-monitor

# Remove files
sudo rm -rf /opt/automata-nexus
sudo rm /etc/systemd/system/nexus-*.service
sudo systemctl daemon-reload

# Or use Makefile
sudo make deep-clean
```

---

## Support

For additional help:
- Check the [README](README_COMPLETE.md)
- Review [API Documentation](docs/api.md)
- Submit issues on GitHub

---

**Version**: 2.0.0  
**Last Updated**: 2024  
**License**: Commercial (see LICENSE)