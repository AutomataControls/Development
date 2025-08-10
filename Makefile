# Automata Nexus Controller - Makefile
# Complete build system for Raspberry Pi 5

.PHONY: all check install build clean test run help

# Configuration
INSTALL_DIR = /opt/automata-nexus
BUILD_TARGET = aarch64-unknown-linux-gnu
BINARY_NAME = nexus-controller

# Detect if we're on RPi5
ARCH := $(shell uname -m)
ifeq ($(ARCH),aarch64)
    ON_RPI = 1
else
    ON_RPI = 0
endif

# Default target
all: check build

# Help target
help:
	@echo "Automata Nexus Controller - Build System"
	@echo "========================================"
	@echo ""
	@echo "Available targets:"
	@echo "  make check      - Run pre-flight dependency check"
	@echo "  make fix        - Auto-fix common issues (requires sudo)"
	@echo "  make build      - Build the application (release mode)"
	@echo "  make install    - Full installation (requires sudo)"
	@echo "  make test       - Run tests"
	@echo "  make verify     - Verify installation"
	@echo "  make run        - Run the application"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make dev        - Build in development mode"
	@echo "  make update     - Update dependencies"
	@echo ""
	@echo "Quick start:"
	@echo "  make check      # Verify dependencies"
	@echo "  make fix        # Fix any issues (if needed)"
	@echo "  make            # Build everything"
	@echo "  sudo make install # Install system-wide"
	@echo "  make verify     # Test installation"

# Pre-flight check
check:
	@echo "Running pre-flight check..."
	@bash installer/preflight_check.sh

# Auto-fix common issues
fix:
	@if [ "$$(id -u)" != "0" ]; then \
		echo "Error: Fix requires root privileges"; \
		echo "Please run: sudo make fix"; \
		exit 1; \
	fi
	@echo "Running automatic fix tool..."
	@bash installer/quick_fix.sh

# Verify installation
verify:
	@echo "Verifying installation..."
	@bash installer/test_installation.sh

# Install all dependencies and build
install: check
	@if [ "$$(id -u)" != "0" ]; then \
		echo "Error: Installation requires root privileges"; \
		echo "Please run: sudo make install"; \
		exit 1; \
	fi
	@echo "Starting complete installation..."
	@bash installer/install_nexus_complete.sh

# Build the application
build:
	@echo "Building Nexus Controller..."
	@if [ -f "Cargo_complete.toml" ] && [ ! -f "Cargo.toml" ]; then \
		echo "Using Cargo_complete.toml..."; \
		cp Cargo_complete.toml Cargo.toml; \
	fi
	@echo "Updating dependencies..."
	@cargo update
	@echo "Building release binary for $(BUILD_TARGET)..."
	@if [ $(ON_RPI) -eq 1 ]; then \
		RUSTFLAGS="-C target-cpu=cortex-a76 -C opt-level=3" cargo build --release --target $(BUILD_TARGET); \
	else \
		cargo build --release --target $(BUILD_TARGET); \
	fi
	@echo "Build complete!"
	@echo "Binary location: target/$(BUILD_TARGET)/release/$(BINARY_NAME)"

# Development build
dev:
	@echo "Building in development mode..."
	@cargo build
	@echo "Development build complete!"

# Run tests
test:
	@echo "Running tests..."
	@cargo test --all-features -- --nocapture
	@echo "Tests complete!"

# Run the application
run: build
	@echo "Running Nexus Controller..."
	@if [ -f "target/$(BUILD_TARGET)/release/$(BINARY_NAME)" ]; then \
		./target/$(BUILD_TARGET)/release/$(BINARY_NAME); \
	else \
		echo "Error: Binary not found. Run 'make build' first."; \
		exit 1; \
	fi

# Update dependencies
update:
	@echo "Updating Rust toolchain..."
	@rustup update
	@echo "Updating cargo dependencies..."
	@cargo update
	@echo "Update complete!"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf target/
	@echo "Clean complete!"

# Deep clean (including installed files)
deep-clean: clean
	@echo "Performing deep clean..."
	@if [ "$$(id -u)" = "0" ]; then \
		systemctl stop nexus-controller 2>/dev/null || true; \
		systemctl disable nexus-controller 2>/dev/null || true; \
		rm -rf $(INSTALL_DIR); \
		rm -f /etc/systemd/system/nexus-controller.service; \
		systemctl daemon-reload; \
		echo "Deep clean complete!"; \
	else \
		echo "Note: Run 'sudo make deep-clean' to remove installed files"; \
	fi

# Quick install for development
quick-install: build
	@echo "Quick install for testing..."
	@mkdir -p ~/.local/bin
	@cp target/$(BUILD_TARGET)/release/$(BINARY_NAME) ~/.local/bin/
	@echo "Installed to ~/.local/bin/$(BINARY_NAME)"
	@echo "Make sure ~/.local/bin is in your PATH"

# Generate documentation
docs:
	@echo "Generating documentation..."
	@cargo doc --no-deps --open
	@echo "Documentation generated!"

# Check code quality
lint:
	@echo "Running linters..."
	@cargo clippy -- -D warnings
	@cargo fmt -- --check
	@echo "Lint check complete!"

# Format code
format:
	@echo "Formatting code..."
	@cargo fmt
	@echo "Code formatted!"

# Benchmark
bench:
	@echo "Running benchmarks..."
	@cargo bench
	@echo "Benchmarks complete!"

# Create distribution package
dist: build
	@echo "Creating distribution package..."
	@mkdir -p dist
	@tar -czf dist/nexus-controller-$(shell date +%Y%m%d).tar.gz \
		-C target/$(BUILD_TARGET)/release $(BINARY_NAME) \
		-C ../../../ README_COMPLETE.md \
		-C ../../../installer install_nexus_complete.sh preflight_check.sh
	@echo "Distribution package created in dist/"

# Install systemd service only
install-service:
	@if [ "$$(id -u)" != "0" ]; then \
		echo "Error: Service installation requires root privileges"; \
		exit 1; \
	fi
	@echo "Installing systemd service..."
	@mkdir -p $(INSTALL_DIR)
	@cp target/$(BUILD_TARGET)/release/$(BINARY_NAME) $(INSTALL_DIR)/
	@bash -c 'cat > /etc/systemd/system/nexus-controller.service << EOF
[Unit]
Description=Automata Nexus Controller
After=network.target

[Service]
Type=simple
ExecStart=$(INSTALL_DIR)/$(BINARY_NAME)
Restart=always
User=pi
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
EOF'
	@systemctl daemon-reload
	@systemctl enable nexus-controller
	@echo "Service installed! Start with: sudo systemctl start nexus-controller"

# Show system info
info:
	@echo "System Information"
	@echo "=================="
	@echo "Architecture: $(ARCH)"
	@echo "On RPi5: $(ON_RPI)"
	@echo "Rust version: $$(rustc --version)"
	@echo "Cargo version: $$(cargo --version)"
	@echo "Target: $(BUILD_TARGET)"
	@echo ""
	@echo "Memory: $$(free -h | grep Mem | awk '{print $$2}')"
	@echo "Disk space: $$(df -h / | tail -1 | awk '{print $$4}') available"
	@if [ -e /dev/nvme0n1 ]; then \
		echo "NVMe SSD: Detected"; \
	else \
		echo "NVMe SSD: Not detected"; \
	fi

# Development environment setup
setup-dev:
	@echo "Setting up development environment..."
	@rustup component add rustfmt clippy
	@cargo install cargo-watch cargo-edit
	@echo "Development environment ready!"

.DEFAULT_GOAL := help