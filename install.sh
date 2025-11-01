#!/usr/bin/env bash

# Installation script for WhatsApp Linux Client

set -e

echo "======================================"
echo "WhatsApp Linux Client Installer"
echo "======================================"
echo ""

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Determine installation location
if [ "$EUID" -eq 0 ]; then
    # Running as root, install system-wide
    INSTALL_DIR="/opt/whatsapp-linux"
    DESKTOP_DIR="/usr/share/applications"
    ICON_DIR="/usr/share/icons/hicolor/256x256/apps"
    echo "Installing system-wide to $INSTALL_DIR"
else
    # Running as user, install locally
    INSTALL_DIR="$HOME/.local/share/whatsapp-linux"
    DESKTOP_DIR="$HOME/.local/share/applications"
    ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
    echo "Installing for current user to $INSTALL_DIR"
fi

# Check for Node.js
echo ""
echo "Checking dependencies..."
if ! command -v node &> /dev/null; then
    echo "ERROR: Node.js is not installed!"
    echo "Please install Node.js (version 16 or higher) and try again."
    echo ""
    echo "On Ubuntu/Debian: sudo apt install nodejs npm"
    echo "On Fedora: sudo dnf install nodejs npm"
    echo "On Arch: sudo pacman -S nodejs npm"
    exit 1
fi

NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 16 ]; then
    echo "WARNING: Node.js version $NODE_VERSION detected. Version 16+ is recommended."
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo "✓ Node.js $(node -v) found"

# Create installation directory
echo ""
echo "Creating installation directory..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$DESKTOP_DIR"
mkdir -p "$ICON_DIR"

# Copy files
echo "Copying files..."
cp -r "$SCRIPT_DIR"/* "$INSTALL_DIR/"

# Install icon
if [ -f "$INSTALL_DIR/whatsapp-icon.png" ]; then
    echo "Installing icon..."
    cp "$INSTALL_DIR/whatsapp-icon.png" "$ICON_DIR/whatsapp-linux.png"
elif [ -f "$INSTALL_DIR/whatsapp-icon.svg" ]; then
    echo "Installing SVG icon..."
    cp "$INSTALL_DIR/whatsapp-icon.svg" "$ICON_DIR/whatsapp-linux.svg"
fi

# Update desktop file with correct paths
echo "Creating desktop entry..."
cat > "$DESKTOP_DIR/whatsapp-linux.desktop" << EOF
[Desktop Entry]
Type=Application
Name=WhatsApp Linux
Comment=WhatsApp client for Linux using Baileys
Exec=$INSTALL_DIR/whatsapp-linux.sh
Icon=whatsapp-linux
Terminal=false
Categories=Network;InstantMessaging;GTK;
Keywords=whatsapp;messenger;chat;messaging;
StartupNotify=true
StartupWMClass=whatsapp-frontend
EOF

chmod +x "$DESKTOP_DIR/whatsapp-linux.desktop"
chmod +x "$INSTALL_DIR/whatsapp-linux.sh"
chmod +x "$INSTALL_DIR/whatsapp-frontend"

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    echo "Updating desktop database..."
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    echo "Updating icon cache..."
    if [ "$EUID" -eq 0 ]; then
        gtk-update-icon-cache /usr/share/icons/hicolor 2>/dev/null || true
    else
        gtk-update-icon-cache "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
    fi
fi

echo ""
echo "======================================"
echo "Installation completed successfully!"
echo "======================================"
echo ""
echo "Installation directory: $INSTALL_DIR"
echo ""
echo "You can now:"
echo "  - Launch from application menu (search for 'WhatsApp Linux')"
echo "  - Run from terminal: $INSTALL_DIR/whatsapp-linux.sh"
echo ""
echo "To uninstall, run:"
echo "  rm -rf $INSTALL_DIR"
echo "  rm $DESKTOP_DIR/whatsapp-linux.desktop"
echo ""
