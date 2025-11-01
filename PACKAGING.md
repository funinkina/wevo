# WhatsApp Linux Client - Packaging Guide

This project is now packaged as a complete Linux application that automatically manages the Node.js backend when you run the GTK frontend.

## Architecture

The application consists of two components:
1. **Rust GTK Frontend** (`whatsapp-frontend/`) - The user interface
2. **Node.js Backend** (`baileys-backend/`) - WhatsApp API connection using Baileys

The frontend automatically starts and stops the backend server, so you only need to launch one application.

## Development

### Running in Development Mode

```bash
# Using the Makefile
make dev

# Or directly
./start.sh

# Or using the new integrated approach
cd whatsapp-frontend
cargo run --release
```

The Rust application will now:
1. Automatically start the Node.js backend on launch
2. Wait for it to initialize
3. Connect to it via HTTP (port 3000) and WebSocket (port 8787)
4. Automatically stop the backend when you close the app

### Building

```bash
make build
# Or
cd whatsapp-frontend && cargo build --release
```

## Packaging for Distribution

### Create Package

```bash
make package
# Or
./build-package.sh
```

This creates:
- `build/whatsapp-linux/` - The complete application directory
- `build/whatsapp-linux-x86_64.tar.gz` - Distributable tarball

### Package Contents

```
whatsapp-linux/
├── whatsapp-frontend          # Main executable (manages backend)
├── whatsapp-linux.sh          # Launcher script
├── install.sh                 # Installation script
├── whatsapp-linux.desktop     # Desktop entry file
├── style.css                  # GTK styles
├── baileys-backend/           # Node.js backend (bundled)
│   ├── server.js
│   ├── package.json
│   ├── node_modules/         # Pre-installed dependencies
│   └── ...
├── db/                        # Database directory (created at runtime)
└── README.md                  # User documentation
```

## Installation

### For End Users

1. **Extract the package:**
   ```bash
   tar -xzf whatsapp-linux-x86_64.tar.gz
   cd whatsapp-linux
   ```

2. **Run the installer:**
   ```bash
   # For system-wide installation (requires sudo)
   sudo ./install.sh
   
   # For user-only installation
   ./install.sh
   ```

3. **Launch the app:**
   - From application menu: Search for "WhatsApp Linux"
   - From terminal: Run `whatsapp-linux.sh` or the installed executable

### Manual Installation

```bash
# Extract
tar -xzf whatsapp-linux-x86_64.tar.gz

# Move to desired location
sudo mv whatsapp-linux /opt/
# Or for local install
mkdir -p ~/.local/share
mv whatsapp-linux ~/.local/share/

# Run
/opt/whatsapp-linux/whatsapp-linux.sh
# Or
~/.local/share/whatsapp-linux/whatsapp-linux.sh
```

## How It Works

### Backend Management

The Rust application (`main.rs`) now includes:

1. **Auto-start**: On launch, it spawns the Node.js backend as a child process
2. **Path detection**: Automatically finds the backend directory (works in both dev and production)
3. **Dependency check**: Installs npm packages if needed
4. **Lifecycle management**: Tracks the backend process and ensures cleanup
5. **Auto-stop**: When the GTK app closes, it kills the backend process

### Signal Handling

The application uses GTK's `connect_shutdown` signal to ensure the backend is properly terminated when:
- User closes the window
- Application is terminated
- System shutdown occurs

## Requirements

### For Development
- Rust (latest stable)
- Node.js (v16+)
- GTK4 and libadwaita development libraries
- npm

### For End Users (Packaged Version)
- Node.js (v16+)
- GTK4 and libadwaita runtime libraries

## Configuration

### Backend Server Ports
- HTTP API: `http://localhost:3000`
- WebSocket: `ws://localhost:8787`

These are hardcoded but can be made configurable if needed.

### Data Storage
- **Auth data**: `baileys-backend/auth/` (WhatsApp session)
- **Database**: `db/client.db` (messages, contacts)
- **Logs**: Terminal output

## Troubleshooting

### Backend doesn't start
- Check Node.js is installed: `node --version`
- Check backend path is correct
- Look for error messages in terminal
- Try running backend manually: `cd baileys-backend && node server.js`

### Port conflicts
If ports 3000 or 8787 are in use:
- Stop other services using those ports
- Or modify the port numbers in both frontend and backend code

### Desktop integration not working
```bash
# Update desktop database
update-desktop-database ~/.local/share/applications/
# Or for system-wide
sudo update-desktop-database /usr/share/applications/
```

## Uninstallation

### If installed with install.sh
```bash
# System-wide
sudo rm -rf /opt/whatsapp-linux
sudo rm /usr/share/applications/whatsapp-linux.desktop

# User installation
rm -rf ~/.local/share/whatsapp-linux
rm ~/.local/share/applications/whatsapp-linux.desktop
```

## Building from Source

```bash
# Clone the repository
git clone <your-repo-url>
cd whatsapp-linux

# Build and package
make package

# Install
cd build/whatsapp-linux
./install.sh
```

## Future Improvements

- [ ] Add application icon
- [ ] Make ports configurable
- [ ] Add systemd service for backend (optional)
- [ ] Create .deb and .rpm packages
- [ ] Add AppImage support
- [ ] Implement proper logging
- [ ] Add configuration file support
- [ ] Improve error handling and user feedback
