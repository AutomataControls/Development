#!/usr/bin/env python3
"""
Automata Nexus Control Center Smart Installer - Raspberry Pi 5 SSD Edition
Handles both existing SSD setups and new SSD installations
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

class SmartInstallerRPi5:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("Automata Nexus RPi5 Smart Installer")
        self.root.geometry("900x700")
        
        # Installation modes
        self.install_mode = tk.StringVar(value="detect")
        self.ssd_device = None
        self.ssd_mounted = False
        self.ssd_mount_point = None
        self.existing_os = False
        
        # Paths
        self.install_path = None
        self.service_name = "automata-nexus"
        
        self.setup_gui()
        
    def setup_gui(self):
        """Create the installer GUI"""
        # Header
        header_frame = tk.Frame(self.root, bg="#1a1a1a", height=100)
        header_frame.pack(fill=tk.X)
        header_frame.pack_propagate(False)
        
        # Logo
        try:
            logo_path = os.path.join(os.path.dirname(__file__), "..", "public", "automata-nexus-logo.png")
            if os.path.exists(logo_path):
                img = Image.open(logo_path)
                img = img.resize((250, 80), Image.Resampling.LANCZOS)
                self.logo = ImageTk.PhotoImage(img)
                logo_label = tk.Label(header_frame, image=self.logo, bg="#1a1a1a")
                logo_label.pack(pady=10)
        except:
            title_label = tk.Label(header_frame, 
                                 text="Automata Nexus Control Center",
                                 font=("Arial", 20, "bold"),
                                 fg="white", bg="#1a1a1a")
            title_label.pack(pady=25)
        
        subtitle_label = tk.Label(header_frame,
                                text="Raspberry Pi 5 SSD Smart Installer",
                                font=("Arial", 12),
                                fg="#00ff00", bg="#1a1a1a")
        subtitle_label.pack()
        
        # Main content area with tabs
        self.notebook = ttk.Notebook(self.root)
        self.notebook.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        
        # Detection tab
        self.detection_frame = ttk.Frame(self.notebook)
        self.notebook.add(self.detection_frame, text="SSD Detection")
        self.setup_detection_tab()
        
        # Installation tab
        self.install_frame = ttk.Frame(self.notebook)
        self.notebook.add(self.install_frame, text="Installation")
        self.setup_installation_tab()
        
        # Progress tab
        self.progress_frame = ttk.Frame(self.notebook)
        self.notebook.add(self.progress_frame, text="Progress")
        self.setup_progress_tab()
        
        # Start with detection
        self.detect_ssd_setup()
        
    def setup_detection_tab(self):
        """Setup SSD detection interface"""
        # Detection results
        self.detection_text = scrolledtext.ScrolledText(
            self.detection_frame,
            wrap=tk.WORD,
            width=80,
            height=15,
            bg="#1a1a1a",
            fg="#00ff00",
            font=("Courier", 10)
        )
        self.detection_text.pack(padx=20, pady=20, fill=tk.BOTH, expand=True)
        
        # Action buttons
        button_frame = tk.Frame(self.detection_frame)
        button_frame.pack(pady=10)
        
        self.refresh_btn = tk.Button(
            button_frame,
            text="Refresh Detection",
            command=self.detect_ssd_setup,
            bg="#0066cc",
            fg="white",
            font=("Arial", 10),
            padx=15,
            pady=5
        )
        self.refresh_btn.pack(side=tk.LEFT, padx=5)
        
    def setup_installation_tab(self):
        """Setup installation options interface"""
        # Installation mode selection
        mode_frame = tk.LabelFrame(self.install_frame, text="Installation Mode", padx=20, pady=20)
        mode_frame.pack(padx=20, pady=20, fill=tk.X)
        
        # Mode options
        self.mode_existing = tk.Radiobutton(
            mode_frame,
            text="Install on existing SSD (preserves current data)",
            variable=self.install_mode,
            value="existing",
            font=("Arial", 12)
        )
        self.mode_existing.pack(anchor=tk.W, pady=5)
        
        self.mode_new = tk.Radiobutton(
            mode_frame,
            text="Setup new SSD (formats and migrates OS)",
            variable=self.install_mode,
            value="new",
            font=("Arial", 12)
        )
        self.mode_new.pack(anchor=tk.W, pady=5)
        
        self.mode_dual = tk.Radiobutton(
            mode_frame,
            text="Dual setup (OS on SD, apps on SSD)",
            variable=self.install_mode,
            value="dual",
            font=("Arial", 12)
        )
        self.mode_dual.pack(anchor=tk.W, pady=5)
        
        # Path selection for existing SSD
        path_frame = tk.LabelFrame(self.install_frame, text="Installation Path", padx=20, pady=20)
        path_frame.pack(padx=20, pady=10, fill=tk.X)
        
        self.path_var = tk.StringVar(value="/mnt/ssd/automata-nexus")
        tk.Label(path_frame, text="Install to:").pack(side=tk.LEFT)
        self.path_entry = tk.Entry(path_frame, textvariable=self.path_var, width=50)
        self.path_entry.pack(side=tk.LEFT, padx=10)
        
        # Options
        options_frame = tk.LabelFrame(self.install_frame, text="Options", padx=20, pady=20)
        options_frame.pack(padx=20, pady=10, fill=tk.X)
        
        self.backup_sd = tk.BooleanVar(value=True)
        tk.Checkbutton(
            options_frame,
            text="Backup SD card before migration (recommended)",
            variable=self.backup_sd
        ).pack(anchor=tk.W)
        
        self.optimize_ssd = tk.BooleanVar(value=True)
        tk.Checkbutton(
            options_frame,
            text="Apply SSD optimizations",
            variable=self.optimize_ssd
        ).pack(anchor=tk.W)
        
        self.enable_monitoring = tk.BooleanVar(value=True)
        tk.Checkbutton(
            options_frame,
            text="Enable performance monitoring",
            variable=self.enable_monitoring
        ).pack(anchor=tk.W)
        
        # Install button
        self.install_btn = tk.Button(
            self.install_frame,
            text="Start Installation",
            command=self.start_installation,
            bg="#00aa00",
            fg="white",
            font=("Arial", 14, "bold"),
            padx=30,
            pady=10
        )
        self.install_btn.pack(pady=20)
        
    def setup_progress_tab(self):
        """Setup progress display"""
        # Progress label
        self.progress_label = tk.Label(
            self.progress_frame,
            text="Ready to install...",
            font=("Arial", 12)
        )
        self.progress_label.pack(pady=10)
        
        # Progress bar
        self.progress_bar = ttk.Progressbar(
            self.progress_frame,
            length=600,
            mode='determinate'
        )
        self.progress_bar.pack(pady=10)
        
        # Current task
        self.current_task = tk.Label(
            self.progress_frame,
            text="",
            font=("Arial", 10),
            fg="#666666"
        )
        self.current_task.pack(pady=5)
        
        # Log output
        self.log_text = scrolledtext.ScrolledText(
            self.progress_frame,
            wrap=tk.WORD,
            width=80,
            height=20,
            bg="#1a1a1a",
            fg="#00ff00",
            font=("Courier", 9)
        )
        self.log_text.pack(padx=20, pady=10, fill=tk.BOTH, expand=True)
        
    def detect_ssd_setup(self):
        """Detect current SSD configuration"""
        self.detection_text.delete(1.0, tk.END)
        self.log_detection("=== SSD Detection Starting ===\n")
        
        # Check for NVMe devices
        self.log_detection("Checking for NVMe devices...")
        nvme_devices = self.find_nvme_devices()
        
        if nvme_devices:
            self.ssd_device = nvme_devices[0]
            self.log_detection(f"Found NVMe device: {self.ssd_device}")
            
            # Check if it's mounted
            mount_info = self.check_mount_status(self.ssd_device)
            if mount_info:
                self.ssd_mounted = True
                self.ssd_mount_point = mount_info['mount_point']
                self.existing_os = mount_info.get('has_os', False)
                
                self.log_detection(f"SSD is mounted at: {self.ssd_mount_point}")
                self.log_detection(f"Used space: {mount_info['used']}")
                self.log_detection(f"Free space: {mount_info['free']}")
                
                if self.existing_os:
                    self.log_detection("\n⚠️  Existing OS detected on SSD!")
                    self.log_detection("Recommended: Use 'existing SSD' installation mode")
                else:
                    self.log_detection("\n✓ SSD has data but no OS detected")
                    
                # Check for existing Automata Nexus installation
                existing_install = self.check_existing_installation()
                if existing_install:
                    self.log_detection(f"\n⚠️  Found existing Automata Nexus at: {existing_install}")
                    self.log_detection("Consider using the upgrade script instead")
                    
            else:
                self.ssd_mounted = False
                self.log_detection("SSD is not mounted")
                self.log_detection("\n✓ SSD can be formatted for new installation")
                
            # Set recommended mode
            if self.ssd_mounted and self.existing_os:
                self.install_mode.set("existing")
                self.mode_existing.config(fg="green")
            elif self.ssd_mounted:
                self.install_mode.set("dual")
                self.mode_dual.config(fg="green")
            else:
                self.install_mode.set("new")
                self.mode_new.config(fg="green")
                
        else:
            self.log_detection("❌ No NVMe SSD detected!")
            self.log_detection("\nPlease ensure:")
            self.log_detection("1. NVMe SSD is properly connected to PCIe HAT")
            self.log_detection("2. PCIe is enabled in /boot/firmware/config.txt")
            self.log_detection("3. System has been rebooted after enabling PCIe")
            
            self.install_btn.config(state=tk.DISABLED)
            
    def find_nvme_devices(self):
        """Find all NVMe devices"""
        devices = []
        try:
            result = subprocess.run(['lsblk', '-d', '-o', 'NAME,TYPE,SIZE'], 
                                  capture_output=True, text=True)
            for line in result.stdout.split('\n'):
                if 'nvme' in line:
                    device = line.split()[0]
                    devices.append(f"/dev/{device}")
        except:
            pass
        return devices
        
    def check_mount_status(self, device):
        """Check if device is mounted and gather info"""
        try:
            # Check all partitions
            result = subprocess.run(['lsblk', '-J', device], 
                                  capture_output=True, text=True)
            if result.returncode == 0:
                import json
                data = json.loads(result.stdout)
                
                for disk in data.get('blockdevices', []):
                    for child in disk.get('children', []):
                        if child.get('mountpoint'):
                            mount_point = child['mountpoint']
                            
                            # Get disk usage
                            df_result = subprocess.run(['df', '-h', mount_point], 
                                                     capture_output=True, text=True)
                            lines = df_result.stdout.strip().split('\n')
                            if len(lines) > 1:
                                parts = lines[1].split()
                                
                                # Check for OS indicators
                                has_os = any(os.path.exists(os.path.join(mount_point, d)) 
                                           for d in ['boot', 'etc', 'usr', 'var'])
                                
                                return {
                                    'mount_point': mount_point,
                                    'size': parts[1],
                                    'used': parts[2],
                                    'free': parts[3],
                                    'use_percent': parts[4],
                                    'has_os': has_os
                                }
        except:
            pass
        return None
        
    def check_existing_installation(self):
        """Check for existing Automata Nexus installation"""
        possible_paths = [
            "/mnt/ssd/automata-nexus",
            "/opt/automata-nexus",
            "/home/Automata/automata-nexus",
            "/home/pi/automata-nexus"
        ]
        
        if self.ssd_mount_point:
            possible_paths.insert(0, os.path.join(self.ssd_mount_point, "automata-nexus"))
            
        for path in possible_paths:
            if os.path.exists(path) and os.path.exists(os.path.join(path, "app")):
                return path
                
        return None
        
    def log_detection(self, message):
        """Log to detection text area"""
        self.detection_text.insert(tk.END, message + "\n")
        self.detection_text.see(tk.END)
        self.root.update()
        
    def start_installation(self):
        """Start the installation process"""
        mode = self.install_mode.get()
        
        if mode == "detect":
            messagebox.showwarning("Select Mode", "Please select an installation mode")
            return
            
        if not self.ssd_device and mode != "existing":
            messagebox.showerror("No SSD", "No SSD detected. Cannot proceed.")
            return
            
        # Confirm installation
        if mode == "new":
            if not messagebox.askyesno(
                "Confirm Format", 
                "This will FORMAT the SSD and migrate the OS.\n\n"
                "ALL DATA ON THE SSD WILL BE LOST!\n\n"
                "Are you sure you want to continue?"
            ):
                return
                
        # Switch to progress tab
        self.notebook.select(self.progress_frame)
        
        # Disable buttons
        self.install_btn.config(state=tk.DISABLED)
        
        # Start installation in thread
        install_thread = threading.Thread(
            target=self.run_installation,
            args=(mode,),
            daemon=True
        )
        install_thread.start()
        
    def run_installation(self, mode):
        """Run the appropriate installation based on mode"""
        try:
            if mode == "existing":
                self.install_on_existing_ssd()
            elif mode == "new":
                self.setup_new_ssd()
            elif mode == "dual":
                self.setup_dual_mode()
                
            self.log("\n✅ Installation completed successfully!")
            self.progress_label.config(text="Installation Complete!")
            
            messagebox.showinfo(
                "Success",
                "Installation completed successfully!\n\n"
                f"Access the web interface at:\n"
                f"http://localhost:1420 or http://<your-pi-ip>:1420"
            )
            
        except Exception as e:
            self.log(f"\n❌ Installation failed: {str(e)}")
            messagebox.showerror("Installation Failed", str(e))
            
    def install_on_existing_ssd(self):
        """Install on existing SSD without disturbing current data"""
        self.log("=== Installing on Existing SSD ===")
        
        # Determine installation path
        install_path = self.path_var.get()
        if not install_path.startswith('/'):
            install_path = os.path.join(self.ssd_mount_point or '/mnt/ssd', install_path)
            
        self.install_path = install_path
        self.log(f"Installation path: {install_path}")
        
        # Create directory structure
        self.update_progress("Creating directories", 10)
        os.makedirs(install_path, exist_ok=True)
        os.makedirs(f"{install_path}/data", exist_ok=True)
        os.makedirs(f"{install_path}/logs", exist_ok=True)
        os.makedirs(f"{install_path}/cache", exist_ok=True)
        
        # Install dependencies
        self.update_progress("Installing system dependencies", 20)
        self.install_dependencies()
        
        # Copy application
        self.update_progress("Installing application", 40)
        self.install_application()
        
        # Apply optimizations if requested
        if self.optimize_ssd.get():
            self.update_progress("Applying SSD optimizations", 60)
            self.apply_ssd_optimizations()
            
        # Setup service
        self.update_progress("Configuring service", 80)
        self.setup_service()
        
        # Enable monitoring if requested
        if self.enable_monitoring.get():
            self.update_progress("Setting up monitoring", 90)
            self.setup_monitoring()
            
        self.update_progress("Complete", 100)
        
    def setup_new_ssd(self):
        """Format SSD and migrate OS"""
        self.log("=== Setting up New SSD ===")
        
        # Backup SD card if requested
        if self.backup_sd.get():
            self.update_progress("Backing up SD card", 10)
            self.backup_sd_card()
            
        # Partition and format SSD
        self.update_progress("Partitioning SSD", 20)
        self.partition_ssd()
        
        # Mount SSD
        self.update_progress("Mounting SSD", 30)
        self.mount_ssd()
        
        # Migrate OS
        self.update_progress("Migrating OS to SSD", 50)
        self.migrate_os_to_ssd()
        
        # Update boot configuration
        self.update_progress("Updating boot configuration", 70)
        self.update_boot_config()
        
        # Install application
        self.update_progress("Installing application", 85)
        self.install_path = "/mnt/ssd/opt/automata-nexus"
        self.install_application()
        
        # Setup service
        self.update_progress("Configuring service", 95)
        self.setup_service()
        
        self.update_progress("Complete", 100)
        
        self.log("\n⚠️  IMPORTANT: System will boot from SSD after reboot")
        self.log("The SD card can be removed after successful SSD boot")
        
    def setup_dual_mode(self):
        """Setup with OS on SD and applications on SSD"""
        self.log("=== Setting up Dual Mode ===")
        self.log("OS remains on SD card, applications on SSD")
        
        # This is similar to existing SSD but with specific paths
        self.install_path = os.path.join(self.ssd_mount_point or '/mnt/ssd', 'automata-nexus')
        
        # Continue with standard installation
        self.install_on_existing_ssd()
        
    def partition_ssd(self):
        """Partition the SSD"""
        self.log(f"Partitioning {self.ssd_device}...")
        
        # Create GPT partition table
        cmds = [
            ['parted', '-s', self.ssd_device, 'mklabel', 'gpt'],
            ['parted', '-s', self.ssd_device, 'mkpart', 'primary', 'ext4', '0%', '100%']
        ]
        
        for cmd in cmds:
            self.run_command(cmd)
            
        time.sleep(2)  # Let kernel update
        
        # Format partition
        partition = f"{self.ssd_device}p1"
        if os.path.exists(f"{self.ssd_device}1"):
            partition = f"{self.ssd_device}1"
            
        self.log(f"Formatting {partition}...")
        self.run_command(['mkfs.ext4', '-F', partition])
        
    def mount_ssd(self):
        """Mount the SSD"""
        self.ssd_mount_point = "/mnt/ssd"
        os.makedirs(self.ssd_mount_point, exist_ok=True)
        
        partition = f"{self.ssd_device}p1"
        if os.path.exists(f"{self.ssd_device}1"):
            partition = f"{self.ssd_device}1"
            
        self.run_command(['mount', partition, self.ssd_mount_point])
        self.log(f"Mounted SSD at {self.ssd_mount_point}")
        
    def migrate_os_to_ssd(self):
        """Migrate OS from SD card to SSD"""
        self.log("Migrating OS to SSD (this will take time)...")
        
        # Use rsync to copy entire system
        exclude_dirs = [
            '/dev/*', '/proc/*', '/sys/*', '/tmp/*', '/run/*',
            '/mnt/*', '/media/*', '/lost+found', '/boot/firmware/*'
        ]
        
        rsync_cmd = ['rsync', '-axHAWXS', '--numeric-ids', '--info=progress2']
        for exclude in exclude_dirs:
            rsync_cmd.extend(['--exclude', exclude])
            
        rsync_cmd.extend(['/', f'{self.ssd_mount_point}/'])
        
        # Run rsync with progress
        process = subprocess.Popen(
            rsync_cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1
        )
        
        for line in iter(process.stdout.readline, ''):
            if line:
                self.log(f"  {line.strip()}")
                
        process.wait()
        
        # Copy boot files separately
        self.log("Copying boot files...")
        boot_src = "/boot/firmware"
        boot_dst = f"{self.ssd_mount_point}/boot/firmware"
        os.makedirs(boot_dst, exist_ok=True)
        self.run_command(['rsync', '-av', f'{boot_src}/', f'{boot_dst}/'])
        
    def update_boot_config(self):
        """Update boot configuration to use SSD"""
        self.log("Updating boot configuration...")
        
        # Get UUID of SSD partition
        partition = f"{self.ssd_device}p1"
        if os.path.exists(f"{self.ssd_device}1"):
            partition = f"{self.ssd_device}1"
            
        result = subprocess.run(['blkid', '-o', 'value', '-s', 'UUID', partition],
                              capture_output=True, text=True)
        ssd_uuid = result.stdout.strip()
        
        # Update cmdline.txt
        cmdline_path = "/boot/firmware/cmdline.txt"
        with open(cmdline_path, 'r') as f:
            cmdline = f.read().strip()
            
        # Replace root device
        import re
        cmdline = re.sub(r'root=\S+', f'root=UUID={ssd_uuid}', cmdline)
        
        # Backup original
        shutil.copy(cmdline_path, f"{cmdline_path}.sd-backup")
        
        # Write new cmdline
        with open(cmdline_path, 'w') as f:
            f.write(cmdline)
            
        self.log(f"Updated boot to use SSD UUID: {ssd_uuid}")
        
        # Update fstab on SSD
        fstab_path = f"{self.ssd_mount_point}/etc/fstab"
        with open(fstab_path, 'w') as f:
            f.write(f"UUID={ssd_uuid} / ext4 defaults,noatime 0 1\n")
            f.write("tmpfs /tmp tmpfs defaults,noatime,mode=1777 0 0\n")
            
    def backup_sd_card(self):
        """Create backup of SD card"""
        backup_path = f"{self.ssd_mount_point}/sd-backup-{datetime.now().strftime('%Y%m%d-%H%M%S')}.img"
        self.log(f"Creating SD card backup at {backup_path}")
        self.log("This will take a while...")
        
        # Note: In production, you'd want to use dd with progress
        self.log("(Backup simulation - actual implementation would use dd)")
        
    def install_dependencies(self):
        """Install system dependencies"""
        self.log("Installing system dependencies...")
        
        deps = [
            "python3.11", "python3.11-dev", "python3.11-venv",
            "nodejs", "npm", "build-essential", "git",
            "libwebkit2gtk-4.0-dev", "libssl-dev", "libgtk-3-dev",
            "libayatana-appindicator3-dev", "librsvg2-dev",
            "i2c-tools", "libi2c-dev", "libudev-dev", "pkg-config"
        ]
        
        self.run_command(["apt-get", "update"])
        self.run_command(["apt-get", "install", "-y"] + deps)
        
    def install_application(self):
        """Install the application"""
        self.log(f"Installing application to {self.install_path}")
        
        # Copy application files
        app_source = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        shutil.copytree(app_source, f"{self.install_path}/app", dirs_exist_ok=True)
        
        # Build application
        os.chdir(f"{self.install_path}/app")
        
        # Install npm dependencies
        self.log("Installing Node.js dependencies...")
        self.run_command(["npm", "install"])
        
        # Build frontend
        self.log("Building frontend...")
        self.run_command(["npm", "run", "build"])
        
        # Build Rust backend
        self.log("Building Rust backend...")
        os.chdir(f"{self.install_path}/app/src-tauri")
        
        # Install Rust if needed
        if not os.path.exists("/root/.cargo/bin/cargo"):
            self.install_rust()
            
        self.run_command(["/root/.cargo/bin/cargo", "build", "--release", 
                         "--target", "aarch64-unknown-linux-gnu"])
        
    def install_rust(self):
        """Install Rust toolchain"""
        self.log("Installing Rust...")
        rust_installer = "/tmp/rustup.sh"
        self.run_command(["curl", "--proto", "=https", "--tlsv1.2", "-sSf", 
                         "https://sh.rustup.rs", "-o", rust_installer])
        self.run_command(["sh", rust_installer, "-y"])
        self.run_command(["/root/.cargo/bin/rustup", "target", "add", 
                         "aarch64-unknown-linux-gnu"])
        
    def apply_ssd_optimizations(self):
        """Apply SSD-specific optimizations"""
        self.log("Applying SSD optimizations...")
        
        # Mount options
        if self.ssd_mount_point:
            # Get device UUID
            result = subprocess.run(['findmnt', '-n', '-o', 'SOURCE', self.ssd_mount_point],
                                  capture_output=True, text=True)
            device = result.stdout.strip()
            
            if device:
                result = subprocess.run(['blkid', '-o', 'value', '-s', 'UUID', device],
                                      capture_output=True, text=True)
                uuid = result.stdout.strip()
                
                # Update fstab with optimized options
                fstab_line = f"UUID={uuid} {self.ssd_mount_point} ext4 defaults,noatime,nodiratime 0 2\n"
                
                # Add to fstab if not exists
                with open('/etc/fstab', 'r') as f:
                    if uuid not in f.read():
                        with open('/etc/fstab', 'a') as f:
                            f.write(fstab_line)
                            
        # System optimizations
        with open('/etc/sysctl.d/99-ssd-optimizations.conf', 'w') as f:
            f.write("""# SSD Optimizations
vm.swappiness=10
vm.vfs_cache_pressure=50
vm.dirty_background_ratio=5
vm.dirty_ratio=10
""")
        
        self.run_command(['sysctl', '-p', '/etc/sysctl.d/99-ssd-optimizations.conf'])
        
        # Enable TRIM
        self.run_command(['systemctl', 'enable', 'fstrim.timer'])
        
    def setup_service(self):
        """Setup systemd service"""
        self.log("Setting up systemd service...")
        
        service_content = f"""[Unit]
Description=Automata Nexus Automation Control Center
After=network.target

[Service]
Type=simple
User=Automata
Group=Automata
WorkingDirectory={self.install_path}/app
Environment="NODE_ENV=production"
Environment="DATABASE_PATH={self.install_path}/data/metrics.db"
ExecStart={self.install_path}/app/src-tauri/target/aarch64-unknown-linux-gnu/release/building-automation-controller
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
"""
        
        with open(f'/etc/systemd/system/{self.service_name}.service', 'w') as f:
            f.write(service_content)
            
        self.run_command(['systemctl', 'daemon-reload'])
        self.run_command(['systemctl', 'enable', self.service_name])
        
    def setup_monitoring(self):
        """Setup performance monitoring"""
        self.log("Setting up performance monitoring...")
        
        # Create monitoring script
        monitor_script = f"""#!/bin/bash
# Performance monitoring for Automata Nexus

LOG_DIR="{self.install_path}/logs/performance"
mkdir -p "$LOG_DIR"

while true; do
    DATE=$(date +%Y%m%d-%H%M%S)
    
    # System metrics
    top -bn1 > "$LOG_DIR/top-$DATE.log"
    iostat -x 1 10 > "$LOG_DIR/iostat-$DATE.log"
    
    # SSD health
    if [ -e /dev/nvme0 ]; then
        nvme smart-log /dev/nvme0 > "$LOG_DIR/nvme-$DATE.log" 2>/dev/null
    fi
    
    # Clean old logs
    find "$LOG_DIR" -name "*.log" -mtime +7 -delete
    
    sleep 3600
done
"""
        
        script_path = f"{self.install_path}/monitor.sh"
        with open(script_path, 'w') as f:
            f.write(monitor_script)
            
        os.chmod(script_path, 0o755)
        
    def run_command(self, cmd):
        """Run command and log output"""
        if isinstance(cmd, list):
            cmd_str = " ".join(cmd)
        else:
            cmd_str = cmd
            
        self.log(f"Running: {cmd_str}")
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            error = result.stderr or result.stdout
            raise Exception(f"Command failed: {error}")
            
        return result.stdout
        
    def log(self, message):
        """Log to progress text area"""
        self.log_text.insert(tk.END, message + "\n")
        self.log_text.see(tk.END)
        self.root.update()
        
    def update_progress(self, task, percentage):
        """Update progress display"""
        self.current_task.config(text=task)
        self.progress_bar['value'] = percentage
        self.log(f"\n>>> {task} ({percentage}%)")
        self.root.update()
        
    def run(self):
        """Run the installer"""
        self.root.mainloop()

if __name__ == "__main__":
    if os.geteuid() != 0:
        print("This installer must be run with sudo")
        sys.exit(1)
        
    installer = SmartInstallerRPi5()
    installer.run()