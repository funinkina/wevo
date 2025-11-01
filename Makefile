.PHONY: build run clean install package dev test

# Build the release version
build:
	@echo "Building frontend..."
	cd whatsapp-frontend && cargo build --release
	@echo "Installing backend dependencies..."
	cd baileys-backend && npm install

# Run in development mode (using existing start.sh logic but with proper cleanup)
dev:
	@echo "Starting development mode..."
	cd baileys-backend && npm install
	@./start.sh

# Run the packaged version (starts backend automatically)
run: build
	@echo "Running application..."
	cd whatsapp-frontend && cargo run --release

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	rm -rf whatsapp-frontend/target
	rm -rf baileys-backend/node_modules
	rm -rf build
	@echo "Clean complete"

# Create distributable package
package:
	@./build-package.sh

# Install the application (requires package to be built first)
install: package
	@cd build/whatsapp-linux && ./install.sh

# Test backend
test-backend:
	cd baileys-backend && npm test

# Test frontend
test-frontend:
	cd whatsapp-frontend && cargo test

# Help
help:
	@echo "Available targets:"
	@echo "  make build    - Build the application"
	@echo "  make run      - Build and run the application"
	@echo "  make dev      - Run in development mode"
	@echo "  make package  - Create distributable package"
	@echo "  make install  - Build, package, and install"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make test-*   - Run tests"
	@echo "  make help     - Show this help"
