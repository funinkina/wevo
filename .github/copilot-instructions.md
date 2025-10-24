# Wevo - AI Coding Agent Instructions

## Project Overview
Wevo is a native WhatsApp client for Linux built with **Rust + GTK4**, interfacing with [Evolution API](https://github.com/EvolutionAPI/evolution-api) as the WhatsApp backend. This is a work-in-progress GUI application using modern GTK4 patterns.

## Architecture

### Module Structure
- `main.rs` - GTK4 application initialization and main UI scaffolding (header, sidebar, conversation pane)
- `models.rs` - Data structures for API responses (`Chat`, `Message`, `Contact`) and UI state
- `data.rs` - API client layer with fallback to sample data; handles Evolution API communication
- `config.rs` - Persistent config using system keyring (via `keyring` crate) for API credentials
- `ui/` - UI components organized by feature:
    - `contacts.rs` - Sidebar with contact list (ListBox with custom rows)
    - `conversation.rs` - Message thread view with send functionality
    - `widgets.rs` - Reusable components (avatars with image loading, initials fallback)
    - `preferences.rs` - Settings dialog for API URL and key configuration

### Data Flow
1. **Startup**: `main.rs` calls `data::fetch_chats_or_fallback()` → Evolution API or sample data
2. **Contact selection**: Sidebar callback → `data::fetch_messages_or_fallback(remote_jid)` → conversation view
3. **Send message**: User input → background thread (`data::send_message()`) → poll with `glib::timeout_add_local()` → update UI on main thread
4. **Config**: Stored in system keyring via `ConfigManager`, env vars override (`WEVO_URL`, `WEVO_API_KEY`)

## Critical Patterns

### GTK4 Threading Model
**Never call GTK widgets from background threads.** Use channels + `glib::timeout_add_local()` polling:
```rust
let (sender, receiver) = mpsc::channel();
std::thread::spawn(move || {
    let result = blocking_operation();
    sender.send(result).unwrap();
});

glib::timeout_add_local(Duration::from_millis(100), move || {
    if let Ok(data) = receiver.try_recv() {
        // Update GTK widgets here on main thread
        glib::ControlFlow::Break
    } else {
        glib::ControlFlow::Continue
    }
});
```
See `conversation.rs:171-219` (send_message_fn) and `widgets.rs:47-74` (image loading).

### CSS Styling Approach
- Use GNOME Adwaita classes: `"card"`, `"toolbar"`, `"title-3"`, `"caption"`, `"dim-label"`, `"suggested-action"`
- Custom CSS via `CssProvider` for dynamic properties (avatar colors, border-radius)
- Load custom CSS once with `std::sync::Once` (see `conversation.rs:12-37`)

### Avatar System
Avatars support profile pictures OR initials fallback:
- `widgets::create_avatar_with_pic()` checks for URL → downloads async → scales to exact size → circular border-radius
- Initials generated from contact name (first 2 letters), color hash based on name string
- Always set `set_size_request()` to prevent rendering issues

### API Integration
Evolution API uses:
- `POST /chat/findChats/{instance}` - List conversations
- `POST /chat/findMessages/{instance}` - Fetch messages for `remoteJid` (paginated, returns newest first → reverse for display)
- `POST /message/sendText/{instance}` - Send text message
- Auth: `apikey` header
- Models use `serde` with `#[serde(rename = "camelCase")]` for Evolution API's JSON schema

## Development Workflow

### Build & Run
```fish
cargo build
cargo run
```
GTK4 dev dependencies: `libgtk-4-dev` (Ubuntu/Debian) or equivalent.

### Testing API Integration
Set environment variables to override keyring config:
```fish
set -x WEVO_URL "http://localhost:8080"
set -x WEVO_API_KEY "your-api-key"
cargo run
```
Falls back to sample data if API unreachable (see `data::*_or_fallback()` functions).

### Edition Note
Uses `edition = "2024"` in `Cargo.toml` (requires Rust 1.85+).

## Common Tasks

### Adding New UI Components
1. Create widget in appropriate `ui/*.rs` file
2. Use GTK4 builders: `.builder().property(value).build()`
3. Clone widgets before moving into closures (GTK uses reference counting)
4. Set spacing/margins explicitly (GTK4 requires explicit layout unlike GTK3)

### Adding API Endpoints
1. Define response struct in `models.rs` with Serde derives
2. Implement in `data.rs` with `reqwest::blocking` (use background thread if called from UI)
3. Add `*_or_fallback()` wrapper for development without API

### Profile Picture Handling
Load from URL with: `Pixbuf::from_stream()` → `scale_simple()` → `Image::set_from_pixbuf()`. Always use `MemoryInputStream` for in-memory data. Check `widgets.rs:47-74` for async pattern.

## Known Patterns

- **Remote JID format**: `{phone}@s.whatsapp.net` (individual) or `{group_id}@g.us` (group)
- **Message ordering**: API returns newest→oldest, reverse before display (`messages.reverse()`)
- **Time formatting**: Use `chrono` crate for timestamp conversion (Unix → local time)
- **Callback closures**: Wrap in `Rc<>` for multiple uses, clone before `move` into closures
- **Error handling**: `anyhow::Result` throughout, print to stderr + fallback (no modal dialogs yet)

## Project Conventions

- No async runtime (tokio/async-std) - uses `reqwest::blocking` + threads
- UI state managed via callbacks, no global state container
- File organization: One widget/feature per file in `ui/` module
- Prefer GNOME HIG design patterns (header bars, popovers, stack-based navigation)