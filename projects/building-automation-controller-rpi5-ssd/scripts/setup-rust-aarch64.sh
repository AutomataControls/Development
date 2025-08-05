#!/bin/bash

# Setup Rust toolchain for 64-bit ARM (aarch64) cross-compilation

echo "Setting up Rust toolchain for 64-bit ARM (aarch64)..."

# Install Rust if not already installed
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Add aarch64 target
echo "Adding aarch64-unknown-linux-gnu target..."
rustup target add aarch64-unknown-linux-gnu

# Install cross-compilation tools
echo "Installing cross-compilation tools..."
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    sudo apt-get update
    sudo apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    brew install aarch64-elf-gcc
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    # Windows/WSL
    sudo apt-get update
    sudo apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
fi

# Create cargo config for cross-compilation
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"

[env]
PKG_CONFIG_ALLOW_CROSS = "1"
EOF

echo ""
echo "Rust toolchain setup complete!"
echo ""
echo "To build for 64-bit Raspberry Pi OS Bullseye, use:"
echo "  ./scripts/deploy-to-pi.sh <pi-host> release aarch64"
echo ""
echo "To build for 32-bit Raspberry Pi OS, use:"
echo "  ./scripts/deploy-to-pi.sh <pi-host> release armv7"