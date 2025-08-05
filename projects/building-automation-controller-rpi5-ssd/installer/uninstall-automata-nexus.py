#!/usr/bin/env python3
"""
Automata Nexus Automation Control Center - Uninstaller
Copyright (c) 2024 Automata Nexus. All rights reserved.
"""

import os
import sys
import subprocess
import shutil
import tkinter as tk
from tkinter import ttk, messagebox, scrolledtext
import threading

class AutomataNexusUninstaller:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("Automata Nexus - Uninstaller")
        self.root.geometry("700x500")
        self.root.resizable(False, False)
        
        # Center window
        self.root.update_idletasks()
        x = (self.root.winfo_screenwidth() // 2) - (700 // 2)
        y = (self.root.winfo_screenheight() // 2) - (500 // 2)
        self.root.geometry(f"700x500+{x}+{y}")
        
        # Paths and services
        self.install_path = "/opt/automata-nexus"
        self.service_name = "automata-nexus-control-center"
        self.user_data_path = "/var/lib/automata-nexus"
        self.config_path = "/etc/automata-nexus"
        
        # Components to remove
        self.components = [
            ("Stop Services", self.stop_services),
            ("Remove Systemd Service", self.remove_service),
            ("Remove Application Files", self.remove_application),
            ("Remove Configuration", self.remove_config),
            ("Remove User Data", self.remove_user_data),
            ("Remove Python Libraries", self.remove_python_libs),
            ("Remove User & Groups", self.remove_user),
            ("Clean Up", self.cleanup),
        ]
        
        self.current_step = 0
        self.total_steps = len(self.components)
        
        # Setup UI
        self.setup_ui()
        
    def setup_ui(self):
        # Header
        header_frame = tk.Frame(self.root, bg="#dc2626", height=100)
        header_frame.pack(fill=tk.X)
        header_frame.pack_propagate(False)
        
        # Logo
        logo_label = tk.Label(header_frame, text="🏭", font=("Arial", 36), bg="#dc2626", fg="white")
        logo_label.pack(pady=10)
        
        title_label = tk.Label(header_frame, text="Uninstall Automata Nexus Control Center", 
                              font=("Arial", 16, "bold"), bg="#dc2626", fg="white")
        title_label.pack()
        
        # Main content
        self.content_frame = tk.Frame(self.root, bg="white")
        self.content_frame.pack(fill=tk.BOTH, expand=True)
        
        # Show confirmation dialog
        self.show_confirmation()
        
    def show_confirmation(self):
        # Clear content
        for widget in self.content_frame.winfo_children():
            widget.destroy()
            
        # Confirmation frame
        confirm_frame = tk.Frame(self.content_frame, bg="white")
        confirm_frame.pack(fill=tk.BOTH, expand=True, padx=30, pady=30)
        
        # Warning icon and message
        tk.Label(confirm_frame, text="⚠️", font=("Arial", 48), bg="white", fg="#dc2626").pack(pady=10)
        
        tk.Label(confirm_frame, text="Uninstall Confirmation", 
                font=("Arial", 16, "bold"), bg="white").pack(pady=10)
        
        message = tk.Label(confirm_frame, 
                          text="This will completely remove Automata Nexus Automation Control Center\n"
                               "from your system, including:",
                          font=("Arial", 11), bg="white", justify=tk.CENTER)
        message.pack(pady=10)
        
        # List of items to be removed
        items_frame = tk.Frame(confirm_frame, bg="white")
        items_frame.pack(pady=10)
        
        items = [
            "• Application files and binaries",
            "• System service configuration",
            "• Python libraries (megabas, SM16relind, etc.)",
            "• Application user account"
        ]
        
        for item in items:
            tk.Label(items_frame, text=item, font=("Arial", 10), bg="white", 
                    anchor=tk.W).pack(anchor=tk.W, padx=50)
        
        # Data preservation option
        self.preserve_data_var = tk.BooleanVar(value=True)
        tk.Checkbutton(confirm_frame, 
                      text="Preserve user data and configuration files",
                      variable=self.preserve_data_var,
                      bg="white", font=("Arial", 11)).pack(pady=20)
        
        # Buttons
        button_frame = tk.Frame(confirm_frame, bg="white")
        button_frame.pack(pady=20)
        
        tk.Button(button_frame, text="Cancel", command=self.cancel_uninstall,
                 width=15, height=2, bg="#6b7280", fg="white").pack(side=tk.LEFT, padx=10)
        
        tk.Button(button_frame, text="Uninstall", command=self.start_uninstall,
                 width=15, height=2, bg="#dc2626", fg="white").pack(side=tk.LEFT, padx=10)
    
    def start_uninstall(self):
        # Clear content
        for widget in self.content_frame.winfo_children():
            widget.destroy()
            
        # Uninstall frame
        uninstall_frame = tk.Frame(self.content_frame, bg="white")
        uninstall_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)
        
        tk.Label(uninstall_frame, text="Uninstalling Automata Nexus Control Center", 
                font=("Arial", 14, "bold"), bg="white").pack(pady=(0, 20))
        
        # Progress section
        self.status_label = tk.Label(uninstall_frame, text="Preparing to uninstall...", 
                                   font=("Arial", 10), bg="white")
        self.status_label.pack(pady=5)
        
        # Progress bar
        self.progress = ttk.Progressbar(uninstall_frame, length=640, mode='determinate')
        self.progress.pack(pady=10)
        
        # Log window
        tk.Label(uninstall_frame, text="Uninstall Log:", font=("Arial", 10), bg="white").pack(anchor=tk.W)
        self.log_text = scrolledtext.ScrolledText(uninstall_frame, height=12, width=80, wrap=tk.WORD)
        self.log_text.pack(pady=(5, 20))
        
        # Start uninstall in thread
        self.uninstall_thread = threading.Thread(target=self.run_uninstall)
        self.uninstall_thread.start()
        
    def run_uninstall(self):
        try:
            # Check if running as root
            if os.geteuid() != 0:
                self.log("ERROR: This uninstaller must be run as root (sudo)")
                messagebox.showerror("Error", "Please run this uninstaller with sudo")
                return
                
            self.log("Starting Automata Nexus uninstallation...")
            
            # Run each removal step
            for i, (name, func) in enumerate(self.components):
                self.current_step = i + 1
                self.update_progress(name)
                
                try:
                    func()
                    self.log(f"✓ {name} completed")
                except Exception as e:
                    self.log(f"⚠ {name} warning: {str(e)}")
                    # Continue with uninstall even if step fails
                    
            self.log("\n" + "="*50)
            self.log("Uninstallation completed")
            self.log("="*50)
            
            if self.preserve_data_var.get():
                self.log(f"\nUser data preserved in: {self.user_data_path}")
                self.log(f"Configuration preserved in: {self.config_path}")
            
            self.root.after(0, self.uninstall_complete)
            
        except Exception as e:
            self.log(f"\nERROR: Uninstallation failed - {str(e)}")
            self.root.after(0, lambda: messagebox.showerror("Uninstall Failed", str(e)))
            
    def stop_services(self):
        """Stop running services"""
        self.log("Stopping services...")
        try:
            self.run_command(["systemctl", "stop", self.service_name])
            self.run_command(["systemctl", "disable", self.service_name])
        except:
            pass
            
    def remove_service(self):
        """Remove systemd service"""
        self.log("Removing systemd service...")
        service_path = f"/etc/systemd/system/{self.service_name}.service"
        if os.path.exists(service_path):
            os.remove(service_path)
            self.run_command(["systemctl", "daemon-reload"])
            
    def remove_application(self):
        """Remove application files"""
        self.log("Removing application files...")
        if os.path.exists(self.install_path):
            shutil.rmtree(self.install_path)
            self.log(f"Removed: {self.install_path}")
            
    def remove_config(self):
        """Remove configuration files"""
        if not self.preserve_data_var.get():
            self.log("Removing configuration files...")
            if os.path.exists(self.config_path):
                shutil.rmtree(self.config_path)
                self.log(f"Removed: {self.config_path}")
        else:
            self.log("Preserving configuration files")
            
    def remove_user_data(self):
        """Remove user data"""
        if not self.preserve_data_var.get():
            self.log("Removing user data...")
            if os.path.exists(self.user_data_path):
                shutil.rmtree(self.user_data_path)
                self.log(f"Removed: {self.user_data_path}")
        else:
            self.log("Preserving user data")
            
    def remove_python_libs(self):
        """Remove Python libraries"""
        self.log("Removing Python libraries...")
        libs = ["SMmegabas", "SM16relind", "SM16univin", "SM16uout", "SM8relind"]
        for lib in libs:
            try:
                self.run_command(["pip3", "uninstall", "-y", lib])
            except:
                pass
                
    def remove_user(self):
        """Remove automata user"""
        self.log("Removing user account...")
        try:
            self.run_command(["userdel", "automata"])
        except:
            pass
            
    def cleanup(self):
        """Final cleanup"""
        self.log("Performing final cleanup...")
        
        # Remove any remaining directories
        cleanup_paths = [
            "/var/log/automata-nexus",
            "/run/automata-nexus"
        ]
        
        for path in cleanup_paths:
            if os.path.exists(path):
                try:
                    shutil.rmtree(path)
                except:
                    pass
                    
    def update_progress(self, status):
        """Update progress bar and status"""
        progress = (self.current_step / self.total_steps) * 100
        self.root.after(0, lambda: [
            self.status_label.config(text=f"Removing: {status}"),
            self.progress.config(value=progress)
        ])
        
    def log(self, message):
        """Add message to log window"""
        self.root.after(0, lambda: [
            self.log_text.insert(tk.END, message + "\n"),
            self.log_text.see(tk.END),
            self.log_text.update()
        ])
        
    def run_command(self, cmd):
        """Run shell command"""
        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            raise Exception(result.stderr)
        return result.stdout
        
    def uninstall_complete(self):
        """Show uninstall complete dialog"""
        messagebox.showinfo(
            "Uninstall Complete",
            "Automata Nexus Automation Control Center has been removed from your system."
        )
        self.root.quit()
        
    def cancel_uninstall(self):
        """Cancel uninstallation"""
        self.root.quit()
        
    def run(self):
        """Run the uninstaller"""
        self.root.mainloop()

if __name__ == "__main__":
    uninstaller = AutomataNexusUninstaller()
    uninstaller.run()