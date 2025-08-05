#!/usr/bin/env python3
"""
Automata Nexus Control Center Installer - Raspberry Pi 5 SSD Edition
Optimized for RPi5 with NVMe SSD on Bookworm 64-bit
"""

import os
import sys
import subprocess
import time
import shutil
import json
from pathlib import Path
import tkinter as tk
from tkinter import ttk, messagebox, scrolledtext
from datetime import datetime
import threading
from PIL import Image, ImageTk

class AutomataNexusRPi5Installer:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("Automata Nexus RPi5 SSD Installer")
        self.root.geometry("800x600")
        
        # Check if running on RPi5
        self.check_hardware()
        
        # Installation paths - optimized for SSD
        self.ssd_path = "/mnt/ssd"
        self.install_path = f"{self.ssd_path}/automata-nexus"
        self.data_path = f"{self.ssd_path}/automata-nexus/data"
        self.metrics_path = f"{self.ssd_path}/metrics"
        self.cache_path = f"{self.ssd_path}/automata-nexus/cache"
        self.service_name = "automata-nexus"
        
        # Component list with RPi5 optimizations
        self.components = [
            ("System Optimization", self.optimize_rpi5_system),
            ("SSD Setup", self.setup_ssd),
            ("Performance Tuning", self.tune_performance),
            ("System Update", self.update_system),
            ("I2C & GPIO Setup", self.enable_hardware),
            ("Python 3.11+", self.install_python),
            ("Node.js 20+", self.install_nodejs),
            ("Rust 1.75+", self.install_rust),
            ("System Dependencies", self.install_system_deps),
            ("Python Libraries", self.install_python_libs),
            ("Sequent Libraries", self.install_sequent_libs),
            ("Database Setup", self.setup_databases),
            ("Control Center Application", self.install_application),
            ("Service Configuration", self.install_service),
            ("Performance Monitoring", self.setup_monitoring),
            ("Automated Backups", self.setup_backups),
        ]
        
        self.current_step = 0
        self.log_file = None
        self.setup_gui()
        
    def check_hardware(self):
        """Verify running on RPi5 with proper setup"""
        try:
            # Check for RPi5
            with open('/proc/cpuinfo', 'r') as f:
                cpuinfo = f.read()
                if 'BCM2712' not in cpuinfo:
                    messagebox.showwarning(
                        "Hardware Check",
                        "This installer is optimized for Raspberry Pi 5.\n"
                        "Continuing anyway, but some optimizations may not apply."
                    )
            
            # Check for 64-bit OS
            arch = subprocess.check_output(['uname', '-m']).decode().strip()
            if arch != 'aarch64':
                raise Exception("This installer requires 64-bit Raspberry Pi OS")
                
        except Exception as e:
            messagebox.showerror("Hardware Check Failed", str(e))
            sys.exit(1)
    
    def setup_gui(self):
        """Create the installer GUI"""
        # Header with logo
        header_frame = tk.Frame(self.root, bg="#1a1a1a", height=100)
        header_frame.pack(fill=tk.X)
        header_frame.pack_propagate(False)
        
        # Try to load logo
        try:
            logo_path = os.path.join(os.path.dirname(__file__), "..", "public", "automata-nexus-logo.png")
            if os.path.exists(logo_path):
                img = Image.open(logo_path)
                img = img.resize((250, 80), Image.Resampling.LANCZOS)
                self.logo = ImageTk.PhotoImage(img)
                logo_label = tk.Label(header_frame, image=self.logo, bg="#1a1a1a")
                logo_label.pack(pady=10)
        except:
            # Fallback to text
            title_label = tk.Label(header_frame, 
                                 text="Automata Nexus Control Center",
                                 font=("Arial", 20, "bold"),
                                 fg="white", bg="#1a1a1a")
            title_label.pack(pady=25)
        
        subtitle_label = tk.Label(header_frame,
                                text="Raspberry Pi 5 SSD Edition",
                                font=("Arial", 12),
                                fg="#00ff00", bg="#1a1a1a")
        subtitle_label.pack()
        
        # Progress section
        progress_frame = tk.Frame(self.root, pady=20)
        progress_frame.pack(fill=tk.X, padx=20)
        
        self.progress_label = tk.Label(progress_frame, 
                                     text="Ready to install...",
                                     font=("Arial", 12))
        self.progress_label.pack()
        
        self.progress_bar = ttk.Progressbar(progress_frame, 
                                          length=400, 
                                          mode='determinate',
                                          maximum=len(self.components))
        self.progress_bar.pack(pady=10)
        
        # Current component label
        self.current_component_label = tk.Label(progress_frame,
                                              text="",
                                              font=("Arial", 10),
                                              fg="#666666")
        self.current_component_label.pack()
        
        # Log output
        log_frame = tk.Frame(self.root)
        log_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=10)
        
        self.log_text = scrolledtext.ScrolledText(log_frame, 
                                                 wrap=tk.WORD,
                                                 width=80, 
                                                 height=20,
                                                 bg="#1a1a1a",
                                                 fg="#00ff00",
                                                 font=("Courier", 9))
        self.log_text.pack(fill=tk.BOTH, expand=True)
        
        # Control buttons
        button_frame = tk.Frame(self.root)
        button_frame.pack(fill=tk.X, padx=20, pady=20)
        
        self.install_button = tk.Button(button_frame, 
                                      text="Install",
                                      command=self.start_installation,
                                      bg="#00aa00", 
                                      fg="white",
                                      font=("Arial", 12, "bold"),
                                      padx=30, 
                                      pady=10)
        self.install_button.pack(side=tk.LEFT)
        
        self.cancel_button = tk.Button(button_frame, 
                                     text="Cancel",
                                     command=self.cancel_installation,
                                     bg="#aa0000", 
                                     fg="white",
                                     font=("Arial", 12),
                                     padx=30, 
                                     pady=10)
        self.cancel_button.pack(side=tk.RIGHT)
        
        # Performance metrics
        self.metrics_label = tk.Label(self.root,
                                    text="",
                                    font=("Arial", 9),
                                    fg="#666666")
        self.metrics_label.pack(pady=5)
        
    def log(self, message):
        """Log message to GUI and file"""
        timestamp = datetime.now().strftime("%H:%M:%S")
        log_entry = f"[{timestamp}] {message}\n"
        
        # GUI logging
        self.log_text.insert(tk.END, log_entry)
        self.log_text.see(tk.END)
        self.root.update()
        
        # File logging
        if self.log_file:
            self.log_file.write(log_entry)
            self.log_file.flush()
    
    def start_installation(self):
        """Start the installation process"""
        self.install_button.config(state=tk.DISABLED)
        
        # Open log file
        log_path = f"/tmp/automata-nexus-rpi5-install-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
        self.log_file = open(log_path, 'w')
        
        # Start installation in background thread
        install_thread = threading.Thread(target=self.run_installation)
        install_thread.daemon = True
        install_thread.start()
    
    def run_installation(self):
        """Run the installation steps"""
        try:
            self.log("Starting Automata Nexus RPi5 SSD Installation...")
            self.log(f"Installation log: {self.log_file.name}")
            
            # Check if running as root
            if os.geteuid() != 0:
                raise Exception("This installer must be run with sudo")
            
            # Run each component
            for i, (name, func) in enumerate(self.components):
                self.current_step = i
                self.progress_label.config(text=f"Installing {name}...")
                self.current_component_label.config(text=f"Current Component: {name}")
                self.progress_bar['value'] = i
                self.root.update()
                
                try:
                    func()
                    self.log(f"✓ {name} completed successfully")
                except Exception as e:
                    self.log(f"✗ {name} failed: {str(e)}")
                    raise
            
            # Installation complete
            self.progress_bar['value'] = len(self.components)
            self.progress_label.config(text="Installation Complete!")
            self.current_component_label.config(text="")
            self.log("\n✓ Installation completed successfully!")
            
            # Show completion dialog
            self.root.after(100, self.installation_complete)
            
        except Exception as e:
            self.log(f"\n✗ Installation failed: {str(e)}")
            messagebox.showerror("Installation Failed", str(e))
        finally:
            if self.log_file:
                self.log_file.close()
    
    def run_command(self, cmd, cwd=None):
        """Run shell command with optimized settings for RPi5"""
        if isinstance(cmd, list):
            cmd_str = " ".join(cmd)
        else:
            cmd_str = cmd
            
        self.log(f"Running: {cmd_str}")
        
        # For build commands, use optimized settings
        if "cargo build" in cmd_str:
            # Add RPi5 optimizations
            env = os.environ.copy()
            env['CARGO_BUILD_JOBS'] = '4'  # Use all 4 cores
            env['RUSTFLAGS'] = '-C target-cpu=cortex-a76 -C opt-level=3'
            
            process = subprocess.Popen(cmd_str, shell=True, cwd=cwd, 
                                     stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                     text=True, bufsize=1, env=env)
        else:
            process = subprocess.Popen(cmd_str, shell=True, cwd=cwd, 
                                     stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                     text=True, bufsize=1)
        
        output = []
        for line in iter(process.stdout.readline, ''):
            if line:
                line = line.rstrip()
                output.append(line)
                if "Compiling" in line or "Building" in line or "Finished" in line:
                    self.log(f"  {line}")
                elif "warning:" in line:
                    self.log(f"⚠ {line}")
        
        process.wait()
        
        if process.returncode != 0:
            raise Exception(f"Command failed with exit code {process.returncode}")
            
        return '\n'.join(output)
    
    def optimize_rpi5_system(self):
        """RPi5-specific system optimizations"""
        self.log("Applying Raspberry Pi 5 optimizations...")
        
        # Enable PCIe for NVMe
        config_txt = "/boot/firmware/config.txt"
        if os.path.exists(config_txt):
            with open(config_txt, 'r') as f:
                content = f.read()
            
            if 'dtparam=pciex1' not in content:
                self.log("Enabling PCIe for NVMe...")
                with open(config_txt, 'a') as f:
                    f.write("\n# Enable PCIe for NVMe SSD\n")
                    f.write("dtparam=pciex1\n")
                    f.write("dtparam=pciex1_gen=3\n")  # Gen3 speeds
        
        # Set performance governor
        self.log("Setting performance CPU governor...")
        for cpu in range(4):  # RPi5 has 4 cores
            gov_path = f"/sys/devices/system/cpu/cpu{cpu}/cpufreq/scaling_governor"
            if os.path.exists(gov_path):
                with open(gov_path, 'w') as f:
                    f.write("performance")
        
        # Optimize kernel parameters
        sysctl_conf = """
# RPi5 SSD Optimizations
vm.swappiness=10
vm.vfs_cache_pressure=50
vm.dirty_background_ratio=5
vm.dirty_ratio=10
net.core.rmem_max=134217728
net.core.wmem_max=134217728
net.ipv4.tcp_rmem=4096 87380 134217728
net.ipv4.tcp_wmem=4096 65536 134217728
fs.file-max=2097152
"""
        with open("/etc/sysctl.d/99-automata-nexus.conf", "w") as f:
            f.write(sysctl_conf)
        
        self.run_command(["sysctl", "-p", "/etc/sysctl.d/99-automata-nexus.conf"])
    
    def setup_ssd(self):
        """Setup and optimize SSD storage"""
        self.log("Setting up SSD storage...")
        
        # Check if SSD is mounted
        if not os.path.exists(self.ssd_path):
            raise Exception(f"SSD not mounted at {self.ssd_path}")
        
        # Create directory structure
        dirs = [
            self.install_path,
            self.data_path,
            self.metrics_path,
            self.cache_path,
            f"{self.install_path}/logs",
            f"{self.install_path}/backups"
        ]
        
        for dir_path in dirs:
            os.makedirs(dir_path, exist_ok=True)
            self.log(f"Created directory: {dir_path}")
        
        # Set optimal mount options if not already set
        self.log("Checking SSD mount options...")
        mount_output = subprocess.check_output(["mount"]).decode()
        if self.ssd_path in mount_output and "noatime" not in mount_output:
            self.log("⚠ Consider adding 'noatime' mount option for better SSD performance")
    
    def tune_performance(self):
        """Apply performance tuning for RPi5"""
        self.log("Applying performance tuning...")
        
        # Increase file descriptor limits
        limits_conf = """
# Automata Nexus Limits
* soft nofile 65536
* hard nofile 65536
* soft nproc 4096
* hard nproc 4096
"""
        with open("/etc/security/limits.d/99-automata-nexus.conf", "w") as f:
            f.write(limits_conf)
        
        # Enable huge pages for better memory performance
        self.run_command(["sysctl", "-w", "vm.nr_hugepages=64"])
        
        # Set I/O scheduler to mq-deadline for NVMe
        nvme_dev = "/sys/block/nvme0n1/queue/scheduler"
        if os.path.exists(nvme_dev):
            with open(nvme_dev, 'w') as f:
                f.write("mq-deadline")
            self.log("Set NVMe I/O scheduler to mq-deadline")
    
    def update_system(self):
        """Update system packages"""
        self.log("Updating system packages...")
        self.run_command(["apt-get", "update"])
        # Skip upgrade to save time, just update package lists
    
    def enable_hardware(self):
        """Enable I2C and GPIO interfaces"""
        self.log("Enabling hardware interfaces...")
        
        # Enable I2C
        self.run_command(["raspi-config", "nonint", "do_i2c", "0"])
        
        # Add i2c modules
        modules = ["i2c-dev", "i2c-bcm2835"]
        with open("/etc/modules", "a") as f:
            for module in modules:
                if module not in open("/etc/modules").read():
                    f.write(f"{module}\n")
    
    def install_python(self):
        """Install Python 3.11+"""
        self.log("Installing Python 3.11+...")
        packages = [
            "python3.11", "python3.11-dev", "python3.11-venv",
            "python3-pip", "python3-smbus", "python3-serial"
        ]
        self.run_command(["apt-get", "install", "-y"] + packages)
    
    def install_nodejs(self):
        """Install Node.js 20+"""
        self.log("Installing Node.js 20...")
        # Use NodeSource repository for latest version
        self.run_command(["curl", "-fsSL", "https://deb.nodesource.com/setup_20.x", "|", "bash", "-"])
        self.run_command(["apt-get", "install", "-y", "nodejs"])
    
    def install_rust(self):
        """Install Rust with RPi5 optimizations"""
        self.log("Installing Rust toolchain...")
        
        # Install rustup
        rust_installer = "/tmp/rustup.sh"
        self.run_command(["curl", "--proto", "=https", "--tlsv1.2", "-sSf", 
                         "https://sh.rustup.rs", "-o", rust_installer])
        self.run_command(["sh", rust_installer, "-y"])
        
        # Add ARM64 target with specific features
        self.run_command(["/root/.cargo/bin/rustup", "target", "add", "aarch64-unknown-linux-gnu"])
        
        # Install additional tools
        self.run_command(["/root/.cargo/bin/cargo", "install", "cargo-binutils"])
    
    def install_system_deps(self):
        """Install system dependencies"""
        self.log("Installing system dependencies...")
        packages = [
            # Build tools
            "build-essential", "gcc", "g++", "make", "cmake", "pkg-config",
            # Tauri dependencies
            "libwebkit2gtk-4.0-dev", "libssl-dev", "libgtk-3-dev",
            "libayatana-appindicator3-dev", "librsvg2-dev",
            # Hardware tools
            "i2c-tools", "libi2c-dev", "libudev-dev",
            # Performance tools
            "htop", "iotop", "sysstat", "nvme-cli",
            # Other utilities
            "git", "curl", "wget", "jq", "redis-tools",
            # GUI dependencies
            "python3-pil", "python3-pil.imagetk"
        ]
        self.run_command(["apt-get", "install", "-y"] + packages)
    
    def install_python_libs(self):
        """Install Python libraries"""
        self.log("Installing Python libraries...")
        
        # Create virtual environment on SSD for better performance
        venv_path = f"{self.install_path}/venv"
        self.run_command(["/usr/bin/python3.11", "-m", "venv", venv_path])
        
        # Upgrade pip
        pip_path = f"{venv_path}/bin/pip"
        self.run_command([pip_path, "install", "--upgrade", "pip"])
        
        # Install packages with caching on SSD
        packages = [
            "influxdb-client", "aiohttp", "pandas", "numpy",
            "pyserial", "pymodbus", "redis", "asyncio",
            "prometheus-client", "psutil", "schedule"
        ]
        
        cache_dir = f"{self.cache_path}/pip"
        os.makedirs(cache_dir, exist_ok=True)
        
        self.run_command([pip_path, "install", "--cache-dir", cache_dir] + packages)
    
    def install_sequent_libs(self):
        """Install Sequent Microsystems libraries"""
        self.log("Installing Sequent Microsystems libraries...")
        
        pip_path = f"{self.install_path}/venv/bin/pip"
        
        # Install all Sequent libraries
        sequent_libs = [
            "SM16relind", "SMmegabas", "SM16univin", 
            "SM16uout", "SM8relind"
        ]
        
        for lib in sequent_libs:
            try:
                self.run_command([pip_path, "install", lib])
                self.log(f"✓ Installed {lib}")
            except:
                self.log(f"⚠ Could not install {lib}, will need manual installation")
    
    def setup_databases(self):
        """Setup optimized databases for SSD"""
        self.log("Setting up databases...")
        
        # Create optimized SQLite configuration
        sqlite_config = f"""-- SQLite optimizations for NVMe SSD
PRAGMA cache_size = 100000;  -- 100MB cache
PRAGMA mmap_size = 1073741824;  -- 1GB memory map
PRAGMA page_size = 4096;  -- Match SSD page size
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA temp_store = MEMORY;
PRAGMA wal_autocheckpoint = 10000;
"""
        
        config_path = f"{self.data_path}/sqlite-init.sql"
        with open(config_path, 'w') as f:
            f.write(sqlite_config)
        
        self.log("Created optimized SQLite configuration")
        
        # Setup Redis for high-speed caching
        redis_conf = f"""# Redis configuration for RPi5 SSD
daemonize yes
pidfile /var/run/redis/redis-server.pid
port 6379
bind 127.0.0.1
timeout 0
tcp-keepalive 300
loglevel notice
logfile {self.install_path}/logs/redis.log
databases 16
save 900 1
save 300 10
save 60 10000
stop-writes-on-bgsave-error yes
rdbcompression yes
rdbchecksum yes
dbfilename dump.rdb
dir {self.data_path}
maxmemory 1gb
maxmemory-policy allkeys-lru
"""
        
        redis_conf_path = f"{self.install_path}/redis.conf"
        with open(redis_conf_path, 'w') as f:
            f.write(redis_conf)
        
        self.log("Created Redis configuration for caching")
    
    def install_application(self):
        """Install the application with RPi5 optimizations"""
        self.log("Installing Automata Nexus Control Center...")
        
        # Find application source
        app_source = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        
        # Copy to SSD
        self.log(f"Copying application to SSD...")
        app_dest = f"{self.install_path}/app"
        if os.path.exists(app_dest):
            shutil.rmtree(app_dest)
        shutil.copytree(app_source, app_dest, dirs_exist_ok=True)
        
        # Update configuration for SSD paths
        config_updates = {
            "database_path": f"{self.data_path}/metrics.db",
            "cache_path": self.cache_path,
            "log_path": f"{self.install_path}/logs"
        }
        
        # Build the application
        os.chdir(app_dest)
        
        # Install npm dependencies with SSD cache
        npm_cache = f"{self.cache_path}/npm"
        os.makedirs(npm_cache, exist_ok=True)
        self.run_command(["npm", "config", "set", "cache", npm_cache])
        
        self.log("Installing Node.js dependencies...")
        self.run_command(["npm", "install"])
        
        self.log("Building Next.js frontend...")
        self.run_command(["npm", "run", "build"])
        
        # Build Rust backend with RPi5 optimizations
        self.log("Building Rust backend with RPi5 optimizations...")
        rust_dir = os.path.join(app_dest, "src-tauri")
        os.chdir(rust_dir)
        
        # Create optimized Cargo config
        cargo_config = """[build]
target = "aarch64-unknown-linux-gnu"
jobs = 4

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
rustflags = [
    "-C", "target-cpu=cortex-a76",
    "-C", "opt-level=3",
    "-C", "lto=fat",
    "-C", "codegen-units=1"
]

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
"""
        
        os.makedirs(".cargo", exist_ok=True)
        with open(".cargo/config.toml", "w") as f:
            f.write(cargo_config)
        
        # Build with optimizations
        self.run_command([
            "/root/.cargo/bin/cargo", "build", "--release",
            "--target", "aarch64-unknown-linux-gnu"
        ])
        
        # Update performance metrics
        self.update_metrics("Build completed")
    
    def install_service(self):
        """Install optimized systemd service"""
        self.log("Installing systemd service...")
        
        service_content = f"""[Unit]
Description=Automata Nexus Automation Control Center (RPi5 SSD)
After=network.target redis.service

[Service]
Type=simple
User=Automata
Group=Automata
WorkingDirectory={self.install_path}/app
Environment="NODE_ENV=production"
Environment="DATABASE_PATH={self.data_path}/metrics.db"
Environment="CACHE_PATH={self.cache_path}"
Environment="RUST_LOG=info"

# Performance optimizations
CPUSchedulingPolicy=fifo
CPUSchedulingPriority=50
IOSchedulingClass=realtime
IOSchedulingPriority=0
Nice=-10

# Memory settings
MemoryMax=4G
MemoryHigh=3G

# Security
PrivateTmp=true
NoNewPrivileges=true

ExecStartPre=/bin/sleep 5
ExecStart={self.install_path}/app/src-tauri/target/aarch64-unknown-linux-gnu/release/building-automation-controller
Restart=always
RestartSec=10

# Logging
StandardOutput=append:{self.install_path}/logs/automata-nexus.log
StandardError=append:{self.install_path}/logs/automata-nexus-error.log

[Install]
WantedBy=multi-user.target
"""
        
        service_path = f"/etc/systemd/system/{self.service_name}.service"
        with open(service_path, 'w') as f:
            f.write(service_content)
        
        # Enable service
        self.run_command(["systemctl", "daemon-reload"])
        self.run_command(["systemctl", "enable", self.service_name])
        
        self.log(f"✓ Service installed and enabled")
    
    def setup_monitoring(self):
        """Setup performance monitoring"""
        self.log("Setting up performance monitoring...")
        
        # Install monitoring script
        monitor_script = f"""#!/bin/bash
# Automata Nexus Performance Monitor

LOG_DIR="{self.install_path}/logs/performance"
mkdir -p "$LOG_DIR"

while true; do
    DATE=$(date +%Y%m%d-%H%M%S)
    
    # CPU and Memory
    top -bn1 | head -20 > "$LOG_DIR/top-$DATE.log"
    
    # Disk I/O
    iostat -x 1 10 > "$LOG_DIR/iostat-$DATE.log"
    
    # NVMe health
    nvme smart-log /dev/nvme0 > "$LOG_DIR/nvme-$DATE.log" 2>/dev/null
    
    # Application metrics
    curl -s http://localhost:1420/api/metrics > "$LOG_DIR/app-metrics-$DATE.json"
    
    # Clean old logs (keep 7 days)
    find "$LOG_DIR" -name "*.log" -mtime +7 -delete
    find "$LOG_DIR" -name "*.json" -mtime +7 -delete
    
    sleep 3600  # Run hourly
done
"""
        
        monitor_path = f"{self.install_path}/monitor.sh"
        with open(monitor_path, 'w') as f:
            f.write(monitor_script)
        os.chmod(monitor_path, 0o755)
        
        # Create monitoring service
        monitor_service = f"""[Unit]
Description=Automata Nexus Performance Monitor
After=automata-nexus.service

[Service]
Type=simple
ExecStart={monitor_path}
Restart=always
User=Automata

[Install]
WantedBy=multi-user.target
"""
        
        with open("/etc/systemd/system/automata-monitor.service", 'w') as f:
            f.write(monitor_service)
        
        self.run_command(["systemctl", "enable", "automata-monitor"])
    
    def setup_backups(self):
        """Setup automated backups"""
        self.log("Setting up automated backups...")
        
        backup_script = f"""#!/bin/bash
# Automata Nexus Backup Script

BACKUP_DIR="{self.install_path}/backups"
DATA_DIR="{self.data_path}"
RETENTION_DAYS=30

# Create backup
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_FILE="$BACKUP_DIR/backup-$DATE.tar.gz"

# Stop writes during backup
redis-cli BGSAVE

# Create compressed backup
tar -czf "$BACKUP_FILE" \\
    -C "$DATA_DIR" . \\
    --exclude='*.log' \\
    --exclude='cache/*'

# Resume normal operation
redis-cli FLUSHDB

# Upload to cloud (optional)
# rclone copy "$BACKUP_FILE" remote:automata-backups/

# Clean old backups
find "$BACKUP_DIR" -name "backup-*.tar.gz" -mtime +$RETENTION_DAYS -delete

# Log backup size
SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
echo "$(date): Backup completed - $BACKUP_FILE ($SIZE)" >> "{self.install_path}/logs/backup.log"
"""
        
        backup_path = f"{self.install_path}/backup.sh"
        with open(backup_path, 'w') as f:
            f.write(backup_script)
        os.chmod(backup_path, 0o755)
        
        # Add to crontab (daily at 2 AM)
        cron_entry = f"0 2 * * * {backup_path}\n"
        self.run_command([f'echo "{cron_entry}" | crontab -'])
    
    def update_metrics(self, status):
        """Update performance metrics display"""
        try:
            # Get current metrics
            cpu_percent = psutil.cpu_percent(interval=0.1)
            mem = psutil.virtual_memory()
            
            if os.path.exists("/dev/nvme0n1"):
                disk = psutil.disk_usage(self.ssd_path)
                io_counters = psutil.disk_io_counters(perdisk=True).get('nvme0n1')
                if io_counters:
                    read_mb = io_counters.read_bytes / 1024 / 1024
                    write_mb = io_counters.write_bytes / 1024 / 1024
                    metrics_text = (f"CPU: {cpu_percent}% | RAM: {mem.percent}% | "
                                  f"SSD: {disk.percent}% | "
                                  f"Read: {read_mb:.1f}MB | Write: {write_mb:.1f}MB")
                else:
                    metrics_text = f"CPU: {cpu_percent}% | RAM: {mem.percent}%"
            else:
                metrics_text = f"CPU: {cpu_percent}% | RAM: {mem.percent}%"
            
            self.metrics_label.config(text=metrics_text)
        except:
            pass
    
    def installation_complete(self):
        """Show installation complete dialog"""
        messagebox.showinfo(
            "Installation Complete",
            "Automata Nexus Control Center RPi5 SSD Edition has been successfully installed!\n\n"
            "Optimizations applied:\n"
            "• NVMe SSD performance tuning\n"
            "• CPU governor set to performance\n"
            "• Memory and I/O optimizations\n"
            "• Automated monitoring and backups\n\n"
            "The service can be started with:\n"
            f"sudo systemctl start {self.service_name}\n\n"
            "Access the web interface at:\n"
            "http://localhost:1420 or http://<your-pi-ip>:1420\n\n"
            "Monitor performance:\n"
            f"sudo journalctl -u {self.service_name} -f"
        )
        self.root.quit()
    
    def cancel_installation(self):
        """Cancel installation"""
        if messagebox.askyesno("Cancel Installation", 
                             "Are you sure you want to cancel the installation?"):
            self.root.quit()
    
    def run(self):
        """Run the installer"""
        # Import psutil here to avoid import errors if not installed
        try:
            global psutil
            import psutil
        except ImportError:
            pass
        
        self.root.mainloop()

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--help":
        print("Automata Nexus RPi5 SSD Installer")
        print("Usage: sudo python3 install-automata-nexus-rpi5.py")
        print("\nThis installer is optimized for:")
        print("  - Raspberry Pi 5")
        print("  - 64-bit Raspberry Pi OS Bookworm")
        print("  - NVMe SSD via PCIe HAT")
        print("\nEnsure your SSD is mounted at /mnt/ssd before running.")
        sys.exit(0)
    
    installer = AutomataNexusRPi5Installer()
    installer.run()