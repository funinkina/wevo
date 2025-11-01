#!/usr/bin/env bash

# Build script for WhatsApp Linux Client
# This packages the Rust GTK app and Node.js backend together

set -e

echo "======================================"
echo "Building WhatsApp Linux Client Package"
echo "======================================"

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$PROJECT_ROOT/build"
PACKAGE_DIR="$BUILD_DIR/whatsapp-linux"

# Clean previous builds
echo "Cleaning previous builds..."
rm -rf "$BUILD_DIR"
mkdir -p "$PACKAGE_DIR"

# Build the Rust frontend
echo ""
echo "Building Rust frontend..."
cd "$PROJECT_ROOT/whatsapp-frontend"
cargo build --release

# Copy the frontend binary
echo "Copying frontend binary..."
cp target/release/whatsapp-frontend "$PACKAGE_DIR/"

# Copy the backend
echo "Copying Node.js backend..."
cp -r "$PROJECT_ROOT/baileys-backend" "$PACKAGE_DIR/"

# Install backend dependencies
echo ""
echo "Installing backend dependencies..."
cd "$PACKAGE_DIR/baileys-backend"
npm install --production

# Copy style.css
echo "Copying resources..."
cp "$PROJECT_ROOT/whatsapp-frontend/style.css" "$PACKAGE_DIR/"

# Copy icon
if [ -f "$PROJECT_ROOT/whatsapp-icon.svg" ]; then
    cp "$PROJECT_ROOT/whatsapp-icon.svg" "$PACKAGE_DIR/"
    # Try to convert to PNG if ImageMagick or Inkscape is available
    if command -v convert &> /dev/null; then
        echo "Converting icon to PNG with ImageMagick..."
        convert -background none "$PROJECT_ROOT/whatsapp-icon.svg" -resize 256x256 "$PACKAGE_DIR/whatsapp-icon.png"
    elif command -v inkscape &> /dev/null; then
        echo "Converting icon to PNG with Inkscape..."
        inkscape "$PROJECT_ROOT/whatsapp-icon.svg" -w 256 -h 256 -o "$PACKAGE_DIR/whatsapp-icon.png"
    else
        echo "Note: Install ImageMagick (convert) or Inkscape to generate PNG icon"
        cp "$PROJECT_ROOT/whatsapp-icon.svg" "$PACKAGE_DIR/whatsapp-icon.png"
    fi
else
    echo "Note: whatsapp-icon.svg not found, using placeholder"
fi

# Create directories for runtime data
mkdir -p "$PACKAGE_DIR/db"

# Copy install script
echo "Copying install script..."
cp "$PROJECT_ROOT/install.sh" "$PACKAGE_DIR/"
chmod +x "$PACKAGE_DIR/install.sh"

# Create a launcher script
echo ""
echo "Creating launcher script..."
cat > "$PACKAGE_DIR/whatsapp-linux.sh" << 'EOF'
#!/usr/bin/env bash

# WhatsApp Linux Launcher
# This script launches the WhatsApp GTK application

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Change to the script directory
cd "$SCRIPT_DIR"

# Run the application
./whatsapp-frontend
EOF

chmod +x "$PACKAGE_DIR/whatsapp-linux.sh"

# Create .desktop file
echo "Creating .desktop file..."
cat > "$PACKAGE_DIR/whatsapp-linux.desktop" << EOF
[Desktop Entry]
Type=Application
Name=WhatsApp Linux
Comment=WhatsApp client for Linux using Baileys
Exec=$PACKAGE_DIR/whatsapp-linux.sh
Icon=whatsapp-linux
Terminal=false
Categories=Network;InstantMessaging;GTK;
Keywords=whatsapp;messenger;chat;messaging;
StartupNotify=true
StartupWMClass=whatsapp-frontend
EOF

# Create README
echo "Creating README..."
cat > "$PACKAGE_DIR/README.md" << 'EOF'
# WhatsApp Linux Client

## Installation

1. Copy this folder to a location of your choice (e.g., `/opt/whatsapp-linux` or `~/.local/share/whatsapp-linux`)
2. Make sure you have Node.js installed (version 16 or higher)
3. Run the application using one of these methods:
   - Double-click `whatsapp-linux.sh`
   - Run `./whatsapp-linux.sh` from terminal
   - Run `./whatsapp-frontend` directly

## Desktop Integration

To add WhatsApp to your application menu:

```bash
# Copy the .desktop file to your local applications folder
cp whatsapp-linux.desktop ~/.local/share/applications/

# Update the Exec and Icon paths in the .desktop file to match your installation location
sed -i "s|Exec=.*|Exec=$(pwd)/whatsapp-linux.sh|" ~/.local/share/applications/whatsapp-linux.desktop
sed -i "s|Icon=.*|Icon=$(pwd)/whatsapp-icon.png|" ~/.local/share/applications/whatsapp-linux.desktop

# Update desktop database
update-desktop-database ~/.local/share/applications/
```

## Requirements

- Node.js (version 16+)
- GTK4 and libadwaita libraries

## Data Storage

- Authentication data: `baileys-backend/auth/`
- Chat database: `db/client.db`
- Application logs: Terminal output

## Uninstallation

Simply delete the folder and the .desktop file if you installed it.

EOF

# Create a simple icon placeholder
echo "Note: Add a whatsapp-icon.png to $PACKAGE_DIR for the application icon"

# Create tarball
echo ""
echo "Creating tarball..."
cd "$BUILD_DIR"
tar -czf whatsapp-linux-x86_64.tar.gz whatsapp-linux/

echo ""
echo "======================================"
echo "Build completed successfully!"
echo "======================================"
echo ""
echo "Package location: $BUILD_DIR/whatsapp-linux-x86_64.tar.gz"
echo "Extracted package: $PACKAGE_DIR"
echo ""
echo "To install:"
echo "  1. Extract: tar -xzf whatsapp-linux-x86_64.tar.gz"
echo "  2. Move to desired location: sudo mv whatsapp-linux /opt/"
echo "  3. Run: /opt/whatsapp-linux/whatsapp-linux.sh"
echo ""
echo "For desktop integration, see README.md in the package"
echo ""
