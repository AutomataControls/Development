#!/usr/bin/env python3
"""
Automata Nexus Automation Control Center - Professional Installer
Copyright (c) 2024 Automata Nexus. All rights reserved.
"""

import os
import sys
import subprocess
import time
import threading
import tkinter as tk
from tkinter import ttk, messagebox, scrolledtext
import requests
import tarfile
import shutil
from pathlib import Path

class AutomataNexusInstaller:
    def __init__(self):
        # Check if running as root first
        if os.geteuid() != 0:
            print("ERROR: This installer must be run as root (sudo)")
            sys.exit(1)
            
        # Install PIL dependencies before showing GUI
        self.install_pil_deps()
        
        self.root = tk.Tk()
        self.root.title("Automata Nexus Automation Control Center - Installer")
        self.root.geometry("800x600")
        self.root.resizable(False, False)
        
        # Center window
        self.root.update_idletasks()
        x = (self.root.winfo_screenwidth() // 2) - (800 // 2)
        y = (self.root.winfo_screenheight() // 2) - (600 // 2)
        self.root.geometry(f"800x600+{x}+{y}")
        
        # Installation paths
        self.install_path = "/opt/automata-nexus"
        self.service_name = "automata-nexus-control-center"
        
        # Components to install
        self.components = [
            ("System Update", self.update_system),
            ("Timezone & Time Sync", self.setup_timezone),
            ("Increase Swap Size", self.increase_swap_size),
            ("I2C Interface", self.enable_i2c),
            ("Python 3 & pip", self.install_python),
            ("Node.js 18+", self.install_nodejs),
            ("Rust Toolchain", self.install_rust),
            ("System Dependencies", self.install_system_deps),
            ("Python Libraries", self.install_python_libs),
            ("Sequent Microsystems Drivers", self.install_sequent_drivers),
            ("Control Center Application", self.install_application),
            ("Systemd Service", self.install_service),
            ("Permissions & Groups", self.setup_permissions),
        ]
        
        self.current_step = 0
        self.total_steps = len(self.components)
        
        # Setup UI
        self.setup_ui()
        
    def install_pil_deps(self):
        """Install PIL dependencies before showing GUI"""
        print("Installing GUI dependencies...")
        try:
            # Update package list
            subprocess.run(["apt-get", "update", "-y"], check=True, capture_output=True)
            
            # Install PIL dependencies
            subprocess.run([
                "apt-get", "install", "-y", 
                "python3-pil", "python3-pil.imagetk"
            ], check=True, capture_output=True)
            
            print("✓ GUI dependencies installed successfully")
        except subprocess.CalledProcessError as e:
            print(f"⚠ Warning: Could not install GUI dependencies: {e}")
            print("Continuing with text logo...")
        
    def setup_ui(self):
        # Header with logo - using exact shadcn/ui colors
        # Background: hsl(210 40% 98%) converted to hex = #f8fafc
        header_frame = tk.Frame(self.root, bg="#f8fafc", height=150)
        header_frame.pack(fill=tk.X)
        header_frame.pack_propagate(False)
        
        # Try to load logo immediately, fallback to text logo
        self.logo_label = tk.Label(header_frame, text="🏭", font=("Arial", 48), bg="#f8fafc", fg="#3b82f6")
        self.logo_label.pack(pady=15)
        
        # Try to load the actual logo immediately
        self.load_logo_immediately()
        
        # Foreground: hsl(222.2 84% 4.9%) = #0f172a
        title_label = tk.Label(header_frame, text="Automata Nexus Automation Control Center", 
                              font=("Arial", 18, "bold"), bg="#f8fafc", fg="#0f172a")
        title_label.pack()
        
        # Main content
        self.content_frame = tk.Frame(self.root, bg="white")
        self.content_frame.pack(fill=tk.BOTH, expand=True)
        
        # Show license agreement first
        self.show_license_agreement()
    
    def load_logo_immediately(self):
        """Try to load logo immediately when GUI starts"""
        try:
            from PIL import Image, ImageTk
            
            # Get the current working directory and installer directory
            current_dir = os.getcwd()
            installer_dir = os.path.dirname(os.path.abspath(__file__))
            project_root = os.path.dirname(installer_dir)
            
            # Try multiple possible logo locations
            logo_paths = [
                os.path.join(current_dir, "public/automata-nexus-logo.png"),
                os.path.join(current_dir, "public/images/automata-nexus-logo.png"),
                os.path.join(project_root, "public/automata-nexus-logo.png"),
                os.path.join(project_root, "public/images/automata-nexus-logo.png"),
                "/home/Automata/Development/projects/building-automation-controller/public/automata-nexus-logo.png",
                "/home/Automata/Development/projects/building-automation-controller/public/images/automata-nexus-logo.png"
            ]
            
            for logo_path in logo_paths:
                if os.path.exists(logo_path):
                    try:
                        logo_img = Image.open(logo_path)
                        # Maintain aspect ratio and make it larger
                        logo_img.thumbnail((120, 120), Image.LANCZOS)
                        logo_photo = ImageTk.PhotoImage(logo_img)
                        
                        # Update the existing logo label
                        self.logo_label.configure(image=logo_photo, text="")
                        self.logo_label.image = logo_photo  # Keep a reference
                        print("✓ Logo loaded successfully")
                        return
                    except Exception as img_e:
                        print(f"Error loading image {logo_path}: {str(img_e)}")
                        continue
                    
        except Exception as e:
            print(f"Could not load logo: {str(e)}")
            # Keep the text logo
    
    def update_logo(self):
        """Update the logo after PIL dependencies are installed"""
        try:
            from PIL import Image, ImageTk
            
            # Get the current working directory and installer directory
            current_dir = os.getcwd()
            installer_dir = os.path.dirname(os.path.abspath(__file__))
            project_root = os.path.dirname(installer_dir)
            
            # Try multiple possible logo locations
            logo_paths = [
                os.path.join(current_dir, "public/automata-nexus-logo.png"),
                os.path.join(current_dir, "public/images/automata-nexus-logo.png"),
                os.path.join(project_root, "public/automata-nexus-logo.png"),
                os.path.join(project_root, "public/images/automata-nexus-logo.png"),
                "/home/Automata/Development/projects/building-automation-controller/public/automata-nexus-logo.png",
                "/home/Automata/Development/projects/building-automation-controller/public/images/automata-nexus-logo.png"
            ]
            
            self.log("Checking logo paths:")
            for logo_path in logo_paths:
                exists = os.path.exists(logo_path)
                self.log(f"  {logo_path} -> exists: {exists}")
                if exists:
                    try:
                        self.log(f"Loading logo from: {logo_path}")
                        logo_img = Image.open(logo_path)
                        # Maintain aspect ratio and make it larger
                        logo_img.thumbnail((120, 120), Image.LANCZOS)
                        logo_photo = ImageTk.PhotoImage(logo_img)
                        
                        # Update the existing logo label
                        self.logo_label.configure(image=logo_photo, text="")
                        self.logo_label.image = logo_photo  # Keep a reference
                        self.root.update()  # Force GUI update
                        self.log("✓ Logo updated successfully")
                        return
                    except Exception as img_e:
                        self.log(f"Error loading image {logo_path}: {str(img_e)}")
                        continue
                    
        except Exception as e:
            self.log(f"Could not update logo: {str(e)}")
            # Keep the text logo
        
    def show_license_agreement(self):
        # Clear content
        for widget in self.content_frame.winfo_children():
            widget.destroy()
            
        # License frame
        license_frame = tk.Frame(self.content_frame, bg="white")
        license_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)
        
        tk.Label(license_frame, text="Commercial License Agreement", 
                font=("Arial", 16, "bold"), bg="white").pack(pady=(0, 10))
        
        # License text
        license_text = scrolledtext.ScrolledText(license_frame, wrap=tk.WORD, height=15, width=80)
        license_text.pack(fill=tk.BOTH, expand=True)
        license_text.insert(tk.END, """AUTOMATA NEXUS AUTOMATION CONTROL CENTER
COMMERCIAL LICENSE AGREEMENT

IMPORTANT: READ THIS LICENSE AGREEMENT CAREFULLY BEFORE INSTALLING OR USING THE SOFTWARE.

This Commercial License Agreement ("Agreement") is entered into between Automata Nexus ("Licensor") and the entity or individual installing this software ("Licensee").

1. GRANT OF LICENSE
Subject to the terms of this Agreement, Licensor grants Licensee a non-exclusive, non-transferable license to install and use the Automata Nexus Automation Control Center software ("Software") on a single Raspberry Pi device for commercial purposes.

2. RESTRICTIONS
Licensee shall not:
- Copy, modify, or distribute the Software without prior written consent
- Reverse engineer, decompile, or disassemble the Software
- Remove or alter any proprietary notices or labels
- Use the Software for any unlawful purpose
- Sublicense, rent, lease, or lend the Software

3. OWNERSHIP
The Software is licensed, not sold. Licensor retains all right, title, and interest in and to the Software, including all intellectual property rights.

4. SUPPORT AND UPDATES
- Technical support is provided for licensed installations
- Software updates are included for the duration of the license
- Priority support available with premium licenses

5. WARRANTY DISCLAIMER
THE SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND. LICENSOR DISCLAIMS ALL WARRANTIES, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.

6. LIMITATION OF LIABILITY
IN NO EVENT SHALL LICENSOR BE LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL, OR CONSEQUENTIAL DAMAGES ARISING OUT OF OR IN CONNECTION WITH THE USE OR INABILITY TO USE THE SOFTWARE.

7. TERM AND TERMINATION
This license is effective until terminated. Licensee may terminate by destroying all copies of the Software. Licensor may terminate upon breach of any term of this Agreement.

8. COMPLIANCE WITH LAWS
Licensee agrees to comply with all applicable laws and regulations in connection with the use of the Software.

9. ENTIRE AGREEMENT
This Agreement constitutes the entire agreement between the parties and supersedes all prior agreements and understandings.

10. GOVERNING LAW
This Agreement shall be governed by the laws of the jurisdiction in which Licensor is located.

BY CLICKING "I AGREE" OR INSTALLING THE SOFTWARE, YOU ACKNOWLEDGE THAT YOU HAVE READ THIS AGREEMENT, UNDERSTAND IT, AND AGREE TO BE BOUND BY ITS TERMS.

For licensing inquiries, contact: licensing@automatanexus.com
""")
        license_text.config(state=tk.DISABLED)
        
        # Agreement checkbox
        self.agree_var = tk.BooleanVar()
        agree_cb = tk.Checkbutton(license_frame, text="I have read and agree to the license terms", 
                                 variable=self.agree_var, bg="white", font=("Arial", 11))
        agree_cb.pack(pady=10)
        
        # Buttons
        button_frame = tk.Frame(license_frame, bg="white")
        button_frame.pack(pady=10)
        
        tk.Button(button_frame, text="Cancel", command=self.cancel_installation,
                 width=15, height=2).pack(side=tk.LEFT, padx=5)
        
        self.continue_btn = tk.Button(button_frame, text="Continue", command=self.start_installation,
                                     width=15, height=2, state=tk.DISABLED)
        self.continue_btn.pack(side=tk.LEFT, padx=5)
        
        # Enable continue button when agreement is checked
        self.agree_var.trace("w", lambda *args: self.continue_btn.config(
            state=tk.NORMAL if self.agree_var.get() else tk.DISABLED))
    
    def start_installation(self):
        # Clear content
        for widget in self.content_frame.winfo_children():
            widget.destroy()
            
        # Installation frame
        install_frame = tk.Frame(self.content_frame, bg="white")
        install_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)
        
        tk.Label(install_frame, text="Installing Automata Nexus Automation Control Center", 
                font=("Arial", 16, "bold"), bg="white").pack(pady=(0, 20))
        
        # Progress section
        self.status_label = tk.Label(install_frame, text="Preparing installation...", 
                                   font=("Arial", 11), bg="white")
        self.status_label.pack(pady=5)
        
        # Overall progress
        tk.Label(install_frame, text="Overall Progress:", font=("Arial", 10), bg="white").pack(anchor=tk.W, padx=20)
        self.overall_progress = ttk.Progressbar(install_frame, length=740, mode='determinate')
        self.overall_progress.pack(pady=(5, 10), padx=20)
        
        # Component progress
        tk.Label(install_frame, text="Current Component:", font=("Arial", 10), bg="white").pack(anchor=tk.W, padx=20)
        self.component_progress = ttk.Progressbar(install_frame, length=740, mode='indeterminate')
        self.component_progress.pack(pady=(5, 20), padx=20)
        
        # Log window
        tk.Label(install_frame, text="Installation Log:", font=("Arial", 10), bg="white").pack(anchor=tk.W, padx=20)
        self.log_text = scrolledtext.ScrolledText(install_frame, height=12, width=90, wrap=tk.WORD)
        self.log_text.pack(padx=20, pady=(5, 20))
        
        # Make log text selectable and copyable
        self.log_text.config(state=tk.NORMAL)
        self.log_text.bind("<Control-a>", lambda e: self.log_text.tag_add("sel", "1.0", "end"))
        self.log_text.bind("<Control-c>", lambda e: self.copy_selection())
        
        # Add right-click context menu
        self.context_menu = tk.Menu(self.root, tearoff=0)
        self.context_menu.add_command(label="Copy", command=self.copy_selection)
        self.context_menu.add_command(label="Select All", command=lambda: self.log_text.tag_add("sel", "1.0", "end"))
        self.log_text.bind("<Button-3>", self.show_context_menu)
        
        # Start installation in thread
        self.install_thread = threading.Thread(target=self.run_installation)
        self.install_thread.start()
        
    def run_installation(self):
        try:
            self.log("Starting Automata Nexus installation...")
            self.log(f"Target directory: {self.install_path}")
            
            # Create installation directory
            os.makedirs(self.install_path, exist_ok=True)
            
            # Run each component installation
            for i, (name, func) in enumerate(self.components):
                self.current_step = i + 1
                self.update_progress(name)
                self.component_progress.start(10)
                
                try:
                    func()
                    self.log(f"✓ {name} completed successfully")
                except Exception as e:
                    self.log(f"✗ {name} failed: {str(e)}")
                    raise
                finally:
                    self.component_progress.stop()
                    
            self.log("\n" + "="*50)
            self.log("Installation completed successfully!")
            self.log("="*50)
            self.log("\nThe Automata Nexus Automation Control Center is now installed.")
            self.log("Access the web interface at: http://localhost:1420")
            self.log("\nKey Features Installed:")
            self.log("  • Universal I/O Control with 0-10V scaling")
            self.log("  • BMS Integration with InfluxDB command queries")
            self.log("  • SQLite metrics database with 7-day retention")
            self.log("  • Real-time trend analysis and visualization")
            self.log("  • Maintenance mode with time-limited manual control")
            self.log("  • JavaScript logic engine with fallback capability")
            self.log("\nTo start the service:")
            self.log(f"  sudo systemctl start {self.service_name}")
            self.log("\nTo enable auto-start on boot:")
            self.log(f"  sudo systemctl enable {self.service_name}")
            
            self.root.after(0, self.installation_complete)
            
        except Exception as e:
            error_msg = str(e)
            self.log(f"\nERROR: Installation failed - {error_msg}")
            self.root.after(0, lambda msg=error_msg: messagebox.showerror("Installation Failed", msg))
            
    def update_system(self):
        """Update system packages"""
        self.log("Updating system packages...")
        self.run_command(["apt-get", "update", "-y"])
    
    def setup_timezone(self):
        """Setup Eastern Standard Time and enable NTP time sync"""
        self.log("Setting up Eastern Standard Time...")
        
        try:
            # Set timezone to Eastern
            self.log("Setting timezone to America/New_York (Eastern Time)...")
            self.run_command(["timedatectl", "set-timezone", "America/New_York"])
            
            # Enable NTP time synchronization
            self.log("Enabling NTP time synchronization...")
            self.run_command(["timedatectl", "set-ntp", "true"])
            
            # Install and configure chrony for better time sync
            self.log("Installing chrony for improved time synchronization...")
            self.run_command(["apt-get", "install", "-y", "chrony"])
            
            # Enable and start chrony
            self.run_command(["systemctl", "enable", "chrony"])
            self.run_command(["systemctl", "start", "chrony"])
            
            # Show current time settings
            time_status = self.run_command(["timedatectl", "status"])
            self.log(f"Time configuration:\n{time_status}")
            
            self.log("✓ Timezone set to Eastern Standard Time with NTP sync enabled")
            
        except Exception as e:
            self.log(f"⚠ Warning: Could not configure timezone: {str(e)}")
            self.log("You may need to manually set timezone with: sudo timedatectl set-timezone America/New_York")
    
    def increase_swap_size(self):
        """Increase swap size to 2GB for Rust compilation"""
        self.log("Increasing swap size to 2GB for compilation...")
        
        try:
            # Check current swap size
            result = self.run_command(["free", "-h"])
            self.log(f"Current memory status:\n{result}")
            
            # Edit the swap configuration
            self.log("Configuring swap size to 2048MB...")
            self.run_command(["sed", "-i", "s/^CONF_SWAPSIZE=.*/CONF_SWAPSIZE=2048/", "/etc/dphys-swapfile"])
            
            # Restart swap service
            self.log("Restarting swap service...")
            self.run_command(["dphys-swapfile", "swapoff"])
            self.run_command(["dphys-swapfile", "setup"])
            self.run_command(["dphys-swapfile", "swapon"])
            
            # Verify the change
            result = self.run_command(["free", "-h"])
            self.log(f"Updated memory status:\n{result}")
            self.log("✓ Swap size increased successfully")
            
        except Exception as e:
            self.log(f"⚠ Warning: Could not increase swap size: {str(e)}")
            self.log("Continuing installation - may cause build failures on low memory systems")
        
    def enable_i2c(self):
        """Enable I2C interface"""
        self.log("Enabling I2C interface...")
        self.run_command(["raspi-config", "nonint", "do_i2c", "0"])
        
        # Add i2c modules to boot
        modules = ["i2c-dev", "i2c-bcm2708"]
        with open("/etc/modules", "a") as f:
            for module in modules:
                if module not in open("/etc/modules").read():
                    f.write(f"{module}\n")
                    
    def install_python(self):
        """Install Python and pip"""
        self.log("Installing Python 3 and pip...")
        packages = ["python3", "python3-pip", "python3-dev", "python3-venv", "python3-smbus"]
        self.run_command(["apt-get", "install", "-y"] + packages)
        
    def install_nodejs(self):
        """Install Node.js 18+"""
        self.log("Installing Node.js 18...")
        # Add NodeSource repository
        self.run_command(["curl", "-fsSL", "https://deb.nodesource.com/setup_18.x", "|", "bash", "-"])
        self.run_command(["apt-get", "install", "-y", "nodejs"])
        
    def install_rust(self):
        """Install Rust toolchain"""
        self.log("Installing Rust toolchain...")
        # Download and run rustup
        rust_installer = "/tmp/rustup.sh"
        self.run_command(["curl", "--proto", "=https", "--tlsv1.2", "-sSf", 
                         "https://sh.rustup.rs", "-o", rust_installer])
        self.run_command(["sh", rust_installer, "-y"])
        
        # Add ARM64 target
        self.run_command(["/root/.cargo/bin/rustup", "target", "add", "aarch64-unknown-linux-gnu"])
        
    def install_system_deps(self):
        """Install system dependencies"""
        self.log("Installing system dependencies...")
        packages = [
            "build-essential", "gcc", "g++", "make", "cmake",
            "libwebkit2gtk-4.0-dev", "libssl-dev", "libgtk-3-dev",
            "libayatana-appindicator3-dev", "librsvg2-dev",
            "i2c-tools", "git", "curl", "wget", "expect",
            "gcc-aarch64-linux-gnu", "g++-aarch64-linux-gnu",
            "python3-pil", "python3-pil.imagetk"  # For logo display in installer
        ]
        self.run_command(["apt-get", "install", "-y"] + packages)
        
    def install_python_libs(self):
        """Install Python libraries"""
        self.log("Installing Python libraries...")
        libs = ["SMmegabas", "SM16relind", "SM16univin", "SM16uout", "SM8relind", "requests"]
        for lib in libs:
            try:
                self.log(f"Installing {lib}...")
                self.run_command(["pip3", "install", lib])
                self.log(f"✓ {lib} installed successfully")
            except Exception as e:
                self.log(f"⚠ Warning: {lib} installation failed: {str(e)}")
                # Try with sudo for system packages
                try:
                    self.run_command(["pip3", "install", lib, "-U"])
                    self.log(f"✓ {lib} installed with update flag")
                except:
                    self.log(f"✗ {lib} installation failed completely")
            
    def install_sequent_drivers(self):
        """Install Sequent Microsystems drivers"""
        self.log("Installing Sequent Microsystems drivers...")
        
        repos = [
            ("megabas-rpi", "https://github.com/SequentMicrosystems/megabas-rpi.git"),
            ("16univin-rpi", "https://github.com/SequentMicrosystems/16univin-rpi.git"),
            ("16relind-rpi", "https://github.com/SequentMicrosystems/16relind-rpi.git"),
            ("8relind-rpi", "https://github.com/SequentMicrosystems/8relind-rpi.git"),
            ("16uout-rpi", "https://github.com/SequentMicrosystems/16uout-rpi.git")
        ]
        
        driver_dir = f"{self.install_path}/drivers"
        os.makedirs(driver_dir, exist_ok=True)
        
        for name, url in repos:
            self.log(f"Installing {name} driver...")
            repo_path = f"{driver_dir}/{name}"
            if os.path.exists(repo_path):
                shutil.rmtree(repo_path)
            self.run_command(["git", "clone", url, repo_path])
            self.run_command(["make", "install"], cwd=repo_path)
            
    def install_application(self):
        """Install the main application"""
        self.log("Installing Automata Nexus Control Center...")
        
        # Find the application source - should be in current directory or cloned repo
        possible_sources = [
            # Current directory (if running from repo)
            os.getcwd(),
            # Parent directories
            os.path.dirname(os.getcwd()),
            # Standard clone locations
            "/home/Automata/Development/projects/building-automation-controller",
            "/home/pi/Development/projects/building-automation-controller",
            # Current installer directory parent
            os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        ]
        
        app_source = None
        for source in possible_sources:
            if os.path.exists(os.path.join(source, "package.json")) and os.path.exists(os.path.join(source, "src-tauri")):
                app_source = source
                self.log(f"Found application source at: {source}")
                break
        
        if not app_source:
            raise Exception("Could not find application source directory with package.json and src-tauri")
        
        # Copy application files
        self.log("Copying application files...")
        if os.path.exists(f"{self.install_path}/app"):
            shutil.rmtree(f"{self.install_path}/app")
        shutil.copytree(app_source, f"{self.install_path}/app", dirs_exist_ok=True)
        
        # Build the application
        self.log("Building application...")
        build_dir = f"{self.install_path}/app"
        os.chdir(build_dir)
        
        try:
            # Install npm dependencies
            self.log("Installing Node.js dependencies...")
            self.run_command(["npm", "install"])
            
            # Build Next.js frontend
            self.log("Building Next.js frontend...")
            self.run_command(["npm", "run", "build"])
            
            # Build Rust backend for ARM64
            self.log("Building Rust backend for ARM64...")
            rust_dir = os.path.join(build_dir, "src-tauri")
            os.chdir(rust_dir)
            
            # Add ARM64 target if not present
            self.run_command(["/root/.cargo/bin/rustup", "target", "add", "aarch64-unknown-linux-gnu"])
            
            # Build for ARM64
            self.run_command(["/root/.cargo/bin/cargo", "build", "--release", 
                            "--target", "aarch64-unknown-linux-gnu"])
            
            # Verify binary was created
            binary_path = os.path.join(rust_dir, "target/aarch64-unknown-linux-gnu/release/building-automation-controller")
            if not os.path.exists(binary_path):
                raise Exception(f"Binary not created at {binary_path}")
            
            # Make binary executable
            os.chmod(binary_path, 0o755)
            self.log(f"✓ Binary created successfully at {binary_path}")
            
        except Exception as e:
            self.log(f"Build failed: {str(e)}")
            raise
            
    def install_service(self):
        """Install systemd service"""
        self.log("Installing systemd service...")
        
        service_content = f"""[Unit]
Description=Automata Nexus Automation Control Center
After=network.target

[Service]
Type=simple
User=automata
Group=automata
WorkingDirectory={self.install_path}/app
ExecStart={self.install_path}/app/src-tauri/target/aarch64-unknown-linux-gnu/release/building-automation-controller
Restart=always
RestartSec=10
Environment="RUST_LOG=info"
Environment="WEBKIT_DISABLE_COMPOSITING_MODE=1"

# I2C device access
SupplementaryGroups=i2c
PrivateDevices=no

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
"""
        
        service_path = f"/etc/systemd/system/{self.service_name}.service"
        with open(service_path, "w") as f:
            f.write(service_content)
            
        self.run_command(["systemctl", "daemon-reload"])
        
    def setup_permissions(self):
        """Setup permissions and user groups"""
        self.log("Setting up permissions...")
        
        # Create automata user if it doesn't exist
        try:
            self.run_command(["useradd", "-r", "-s", "/bin/false", "automata"])
        except:
            pass  # User might already exist
            
        # Add automata user to required groups
        self.run_command(["usermod", "-a", "-G", "i2c,gpio", "automata"])
        
        # Set ownership
        self.run_command(["chown", "-R", "automata:automata", self.install_path])
        
        # Set executable permissions
        binary_path = f"{self.install_path}/app/src-tauri/target/aarch64-unknown-linux-gnu/release/building-automation-controller"
        if os.path.exists(binary_path):
            self.run_command(["chmod", "+x", binary_path])
            
    def update_progress(self, status):
        """Update progress bars and status"""
        progress = (self.current_step / self.total_steps) * 100
        self.root.after(0, lambda: [
            self.status_label.config(text=f"Installing: {status} ({self.current_step}/{self.total_steps})"),
            self.overall_progress.config(value=progress)
        ])
        
    def log(self, message):
        """Add message to log window"""
        timestamp = time.strftime("%H:%M:%S")
        log_message = f"[{timestamp}] {message}\n"
        self.root.after(0, lambda: [
            self.log_text.config(state=tk.NORMAL),
            self.log_text.insert(tk.END, log_message),
            self.log_text.see(tk.END),
            self.log_text.update()
        ])
        
    def run_command(self, cmd, cwd=None):
        """Run shell command and log output"""
        if isinstance(cmd, list):
            cmd_str = " ".join(cmd)
        else:
            cmd_str = cmd
            
        self.log(f"Running: {cmd_str}")
        
        result = subprocess.run(cmd_str, shell=True, cwd=cwd, 
                              capture_output=True, text=True)
        
        if result.returncode != 0:
            raise Exception(f"Command failed: {result.stderr}")
            
        return result.stdout
        
    def installation_complete(self):
        """Show installation complete dialog"""
        messagebox.showinfo(
            "Installation Complete",
            "Automata Nexus Automation Control Center has been successfully installed!\n\n"
            "The service can be started with:\n"
            f"sudo systemctl start {self.service_name}\n\n"
            "Access the web interface at:\n"
            "http://localhost:1420"
        )
        self.root.quit()
        
    def cancel_installation(self):
        """Cancel installation"""
        if messagebox.askyesno("Cancel Installation", 
                             "Are you sure you want to cancel the installation?"):
            self.root.quit()
    
    def copy_selection(self):
        """Copy selected text to clipboard"""
        try:
            text = self.log_text.get("sel.first", "sel.last")
            self.root.clipboard_clear()
            self.root.clipboard_append(text)
        except tk.TclError:
            # No selection
            pass
    
    def show_context_menu(self, event):
        """Show right-click context menu"""
        try:
            self.context_menu.tk_popup(event.x_root, event.y_root)
        finally:
            self.context_menu.grab_release()
            
    def run(self):
        """Run the installer"""
        self.root.mainloop()

if __name__ == "__main__":
    installer = AutomataNexusInstaller()
    installer.run()