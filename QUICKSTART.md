# WhatsApp Linux - Quick Start Guide

## 🚀 Quick Start

### For Developers

```bash
# Run in development mode
make dev

# Or build and run with backend auto-management
cd whatsapp-frontend
cargo run --release
```

### For Distribution

```bash
# Build the package
make package

# This creates: build/whatsapp-linux-x86_64.tar.gz
```

### For End Users

```bash
# Extract the package
tar -xzf whatsapp-linux-x86_64.tar.gz
cd whatsapp-linux

# Install (system-wide with sudo, or user-only without)
./install.sh

# Or run without installing
./whatsapp-linux.sh
```

## 📋 What's New

The application now works as a **single unified package**:

✅ **Frontend auto-starts backend** - No need to manually run two separate processes  
✅ **Auto-cleanup** - Backend stops when you close the app  
✅ **Portable** - Everything bundled in one directory  
✅ **Desktop integration** - Appears in application menu  
✅ **Smart path detection** - Works in both dev and production environments  

## 🔧 How It Works

1. **You launch**: `whatsapp-frontend` (the GTK app)
2. **It starts**: Node.js backend (`baileys-backend/server.js`)
3. **It connects**: HTTP API (3000) + WebSocket (8787)
4. **You use**: Full WhatsApp functionality
5. **You close**: Backend automatically stops

## 📦 Package Structure

```
whatsapp-linux/
├── whatsapp-frontend       ← Main executable (manages everything)
├── whatsapp-linux.sh       ← Launcher script
├── install.sh              ← Installation script
├── baileys-backend/        ← Node.js backend (auto-managed)
│   └── node_modules/       ← Pre-installed
└── db/                     ← Your data
```

## 🎯 Commands

| Command | Description |
|---------|-------------|
| `make dev` | Run in development mode |
| `make build` | Build both frontend and backend |
| `make package` | Create distributable package |
| `make install` | Build, package, and install |
| `make clean` | Remove all build artifacts |
| `make help` | Show all available commands |

## 🐛 Troubleshooting

**Backend doesn't start?**
- Ensure Node.js v16+ is installed: `node --version`
- Check terminal output for errors

**Port already in use?**
- Stop other services on ports 3000 and 8787
- Or modify port numbers in code

**Can't find the app in menu?**
```bash
update-desktop-database ~/.local/share/applications/
```

## 📚 Documentation

- `PACKAGING.md` - Full packaging documentation
- `README.md` (in package) - User documentation
- `baileys-doc.md` - Baileys API documentation

## 🎨 TODO

- [ ] Add application icon (whatsapp-icon.png)
- [ ] Create .deb/.rpm packages
- [ ] Add AppImage support
- [ ] Make ports configurable

---

**Need more details?** Check `PACKAGING.md` for complete documentation.
