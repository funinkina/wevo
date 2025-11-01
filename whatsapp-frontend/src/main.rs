mod models;
mod services;
mod ui;

use adw::prelude::*;
use gtk4::prelude::*;
use libadwaita as adw;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Arc;
use std::sync::Mutex;

use models::Database;
use services::ws_client::WhatsAppEvent;
use services::{ApiClient, WebSocketClient};
use ui::{MainView, QrView};

// Global backend process handle
static BACKEND_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

fn get_backend_path() -> PathBuf {
    // Get the executable path
    let exe_path = std::env::current_exe().expect("Failed to get executable path");
    let exe_dir = exe_path
        .parent()
        .expect("Failed to get executable directory");

    // Check if we're in development (target/debug or target/release)
    if exe_dir.ends_with("debug") || exe_dir.ends_with("release") {
        // Development mode - go up from whatsapp-frontend/target/debug to project root
        exe_dir
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .expect("Failed to find project root")
            .join("baileys-backend")
    } else {
        // Production mode - backend should be next to the binary
        exe_dir.join("baileys-backend")
    }
}

fn start_backend() -> Result<Child, std::io::Error> {
    let backend_path = get_backend_path();
    println!("Starting backend from: {}", backend_path.display());

    // Check if node_modules exists, if not install dependencies
    let node_modules = backend_path.join("node_modules");
    if !node_modules.exists() {
        println!("Installing backend dependencies...");
        let install_status = Command::new("npm")
            .arg("install")
            .current_dir(&backend_path)
            .status()?;

        if !install_status.success() {
            eprintln!("Failed to install backend dependencies");
        }
    }

    // Start the Node.js server
    let child = Command::new("node")
        .arg("server.js")
        .current_dir(&backend_path)
        .spawn()?;

    println!("Backend started with PID: {}", child.id());

    // Wait a moment for the server to start
    std::thread::sleep(std::time::Duration::from_secs(2));

    Ok(child)
}

fn stop_backend() {
    let mut process = BACKEND_PROCESS.lock().unwrap();
    if let Some(mut child) = process.take() {
        println!("Stopping backend process...");
        let _ = child.kill();
        let _ = child.wait();
        println!("Backend stopped");
    }
}

fn main() {
    // Start the backend server
    match start_backend() {
        Ok(child) => {
            *BACKEND_PROCESS.lock().unwrap() = Some(child);
        }
        Err(e) => {
            eprintln!("Failed to start backend: {}", e);
            std::process::exit(1);
        }
    }

    let app = adw::Application::builder()
        .application_id("org.aryan.whatsappgtk")
        .build();

    // Setup cleanup on shutdown
    app.connect_shutdown(|_| {
        println!("Application shutting down...");
        stop_backend();
    });

    app.connect_activate(|app| {
        // Load CSS
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(include_str!("../style.css"));
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Initialize database
        let db = Arc::new(Database::new("../db/client.db").expect("Failed to open database"));

        // Initialize API client
        let api = Arc::new(ApiClient::new("http://localhost:3000"));

        // Create main window
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("WhatsApp")
            .default_width(1000)
            .default_height(700)
            .build();

        // Check if already authenticated
        let is_authenticated = db.is_authenticated();

        if is_authenticated {
            // Show main view
            let main_view = MainView::new(Arc::clone(&db), Arc::clone(&api));

            // Load contacts from database first
            main_view.load_contacts();

            let main_view_arc = Arc::new(main_view);

            // Setup WebSocket for receiving messages and contacts
            let (_ws, rx) = WebSocketClient::new("ws://localhost:8787");
            let main_view_clone = Arc::clone(&main_view_arc);
            let db_clone = Arc::clone(&db);

            // Handle incoming WebSocket events
            glib::timeout_add_local(std::time::Duration::from_millis(100), {
                let main_view = Arc::clone(&main_view_clone);
                move || {
                    while let Ok(event) = rx.try_recv() {
                        match event {
                            WhatsAppEvent::Message(msg) => {
                                println!("[main.rs] Received message event for: {}", msg.key.jid);

                                // Extract message content and type
                                let (content, message_type, quoted_id, media_url, caption) =
                                    if let Some(ref msg_data) = msg.message {
                                        let mut content = String::new();
                                        let mut msg_type = "unknown".to_string();
                                        let mut quoted_id = None;
                                        let mut media_url = None;
                                        let mut caption = None;

                                        // Handle text messages
                                        if let Some(text) =
                                            msg_data.get("conversation").and_then(|v| v.as_str())
                                        {
                                            content = text.to_string();
                                            msg_type = "text".to_string();
                                        }
                                        // Handle extended text (with formatting, links, etc.)
                                        else if let Some(ext_text) =
                                            msg_data.get("extendedTextMessage")
                                        {
                                            if let Some(text) =
                                                ext_text.get("text").and_then(|v| v.as_str())
                                            {
                                                content = text.to_string();
                                                msg_type = "text".to_string();
                                            }
                                            // Check for quoted message
                                            if let Some(context) = ext_text.get("contextInfo") {
                                                if let Some(stanza_id) =
                                                    context.get("stanzaId").and_then(|v| v.as_str())
                                                {
                                                    quoted_id = Some(stanza_id.to_string());
                                                }
                                            }
                                        }
                                        // Handle reactions
                                        else if let Some(reaction) =
                                            msg_data.get("reactionMessage")
                                        {
                                            if let Some(text) =
                                                reaction.get("text").and_then(|v| v.as_str())
                                            {
                                                content = format!("Reacted with {}", text);
                                                msg_type = "reaction".to_string();
                                            }
                                            if let Some(key) = reaction.get("key") {
                                                if let Some(msg_id) =
                                                    key.get("id").and_then(|v| v.as_str())
                                                {
                                                    quoted_id = Some(msg_id.to_string());
                                                }
                                            }
                                        }
                                        // Handle image messages
                                        else if let Some(image) = msg_data.get("imageMessage") {
                                            msg_type = "image".to_string();
                                            content = "[Image]".to_string();
                                            if let Some(url) =
                                                image.get("url").and_then(|v| v.as_str())
                                            {
                                                media_url = Some(url.to_string());
                                            }
                                            if let Some(cap) =
                                                image.get("caption").and_then(|v| v.as_str())
                                            {
                                                caption = Some(cap.to_string());
                                                content = format!("[Image] {}", cap);
                                            }
                                        }
                                        // Handle video messages
                                        else if let Some(video) = msg_data.get("videoMessage") {
                                            msg_type = "video".to_string();
                                            content = "[Video]".to_string();
                                            if let Some(url) =
                                                video.get("url").and_then(|v| v.as_str())
                                            {
                                                media_url = Some(url.to_string());
                                            }
                                            if let Some(cap) =
                                                video.get("caption").and_then(|v| v.as_str())
                                            {
                                                caption = Some(cap.to_string());
                                                content = format!("[Video] {}", cap);
                                            }
                                        }
                                        // Handle audio messages
                                        else if msg_data.get("audioMessage").is_some() {
                                            msg_type = "audio".to_string();
                                            content = "[Audio]".to_string();
                                        }
                                        // Handle document messages
                                        else if let Some(doc) = msg_data.get("documentMessage") {
                                            msg_type = "document".to_string();
                                            if let Some(filename) =
                                                doc.get("fileName").and_then(|v| v.as_str())
                                            {
                                                content = format!("[Document: {}]", filename);
                                            } else {
                                                content = "[Document]".to_string();
                                            }
                                        }
                                        // Handle stickers
                                        else if msg_data.get("stickerMessage").is_some() {
                                            msg_type = "sticker".to_string();
                                            content = "[Sticker]".to_string();
                                        }

                                        (content, msg_type, quoted_id, media_url, caption)
                                    } else {
                                        (
                                            "[Empty message]".to_string(),
                                            "unknown".to_string(),
                                            None,
                                            None,
                                            None,
                                        )
                                    };

                                // Determine sender
                                let sender = if msg.key.from_me {
                                    "me".to_string()
                                } else {
                                    msg.key
                                        .participant
                                        .as_ref()
                                        .unwrap_or(&msg.key.jid)
                                        .split('@')
                                        .next()
                                        .unwrap_or("Unknown")
                                        .to_string()
                                };

                                // Convert to Message model
                                let message = models::Message {
                                    id: None,
                                    message_id: msg.key.id.clone(),
                                    jid: msg.key.jid.clone(),
                                    sender,
                                    content,
                                    timestamp: msg.timestamp,
                                    is_from_me: msg.key.from_me,
                                    message_type,
                                    raw_data: Some(
                                        serde_json::to_string(&msg.message).unwrap_or_default(),
                                    ),
                                    quoted_message_id: quoted_id,
                                    media_url,
                                    caption,
                                };

                                // Save to database
                                if let Err(e) = db_clone.save_message(&message) {
                                    eprintln!(
                                        "Failed to save message {}: {}",
                                        message.message_id, e
                                    );
                                } else {
                                    println!(
                                        "✅ Saved message: {} in chat {}",
                                        message.message_id, message.jid
                                    );
                                }
                            }
                            WhatsAppEvent::Contact(wa_contact) => {
                                println!(
                                    "[main.rs] Received Contact: {} ({})",
                                    wa_contact.name.as_ref().unwrap_or(&wa_contact.id),
                                    wa_contact.id
                                );

                                // Convert WAContact to Contact model
                                let contact = models::Contact {
                                    jid: wa_contact.id.clone(),
                                    name: wa_contact.name.or(wa_contact.notify).unwrap_or_else(
                                        || {
                                            wa_contact
                                                .id
                                                .split('@')
                                                .next()
                                                .unwrap_or(&wa_contact.id)
                                                .to_string()
                                        },
                                    ),
                                    last_message: None,
                                    last_message_time: None,
                                    unread_count: 0,
                                    conversation_timestamp: 0,
                                    is_group: wa_contact.id.contains("@g.us"),
                                    archived: false,
                                    pinned: 0,
                                    mute_end_time: 0,
                                    profile_picture_url: None,
                                };

                                // Save to database
                                if let Err(e) = db_clone.save_contact(&contact) {
                                    eprintln!("Failed to save contact {}: {}", contact.jid, e);
                                } else {
                                    println!("Saved contact: {}", contact.name);
                                }
                            }
                            WhatsAppEvent::Chat(wa_chat) => {
                                println!(
                                    "[main.rs] Received Chat: {} ({})",
                                    wa_chat.name.as_ref().unwrap_or(&wa_chat.id),
                                    wa_chat.id
                                );

                                // Convert WAChat to Contact model
                                let contact = models::Contact {
                                    jid: wa_chat.id.clone(),
                                    name: wa_chat.name.unwrap_or_else(|| {
                                        wa_chat
                                            .id
                                            .split('@')
                                            .next()
                                            .unwrap_or(&wa_chat.id)
                                            .to_string()
                                    }),
                                    last_message: None,
                                    last_message_time: None,
                                    unread_count: wa_chat.unread_count.unwrap_or(0),
                                    conversation_timestamp: wa_chat
                                        .conversation_timestamp
                                        .unwrap_or(0)
                                        as i64,
                                    is_group: wa_chat.id.contains("@g.us"),
                                    archived: wa_chat.archived.unwrap_or(false),
                                    pinned: wa_chat.pinned.unwrap_or(0),
                                    mute_end_time: wa_chat.mute_end_time.unwrap_or(0),
                                    profile_picture_url: None,
                                };

                                // Save to database
                                if let Err(e) = db_clone.save_contact(&contact) {
                                    eprintln!("Failed to save chat {}: {}", contact.jid, e);
                                } else {
                                    println!("Saved chat: {}", contact.name);

                                    // Update UI with the new/updated contact
                                    if let Ok(contacts) = db_clone.get_contacts() {
                                        main_view.update_contacts(contacts);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    glib::Continue(true)
                }
            });

            // Setup send message handler
            main_view_arc.setup_send_handler({
                let api = Arc::clone(&api);
                let main_view = Arc::clone(&main_view_arc);
                move |jid, text| {
                    if let Err(e) = api.send_message(&jid, &text) {
                        eprintln!("Failed to send message: {}", e);
                    } else {
                        // Add message to UI
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64;
                        main_view.add_message(&jid, "me", &text, timestamp, true);
                    }
                }
            });

            window.set_content(Some(&main_view_arc.widget));
        } else {
            // Show QR view
            let qr_view = Arc::new(QrView::new());
            window.set_content(Some(&qr_view.widget));

            // Request QR code from backend
            let api_clone_qr = Arc::clone(&api);
            std::thread::spawn(move || {
                println!("Requesting QR code from backend...");
                if let Err(e) = api_clone_qr.request_qr() {
                    eprintln!("Failed to request QR code: {}", e);
                }
            });

            // Setup WebSocket for QR code
            let (_ws, rx) = WebSocketClient::new("ws://localhost:8787");
            let qr_view_clone = Arc::clone(&qr_view);
            let db_clone = Arc::clone(&db);
            let api_clone = Arc::clone(&api);
            let window_clone = window.clone();

            glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                while let Ok(event) = rx.try_recv() {
                    match event {
                        WhatsAppEvent::QrCode(qr) => {
                            println!("QR Code received from backend");
                            qr_view_clone.show_qr(&qr);
                        }
                        WhatsAppEvent::Connected => {
                            println!("Connected to WhatsApp!");
                            qr_view_clone.show_connecting();

                            // Mark as authenticated
                            let _ = db_clone.set_authenticated(true);

                            // Transition to main view - events will populate contacts via WebSocket
                            let main_view = Arc::new(MainView::new(
                                Arc::clone(&db_clone),
                                Arc::clone(&api_clone),
                            ));

                            main_view.load_contacts();

                            // Create a NEW WebSocket connection for the main view
                            let (_ws_main, rx_main) = WebSocketClient::new("ws://localhost:8787");
                            let main_view_clone = Arc::clone(&main_view);
                            let db_clone_ws = Arc::clone(&db_clone);

                            glib::timeout_add_local(
                                std::time::Duration::from_millis(100),
                                move || {
                                    while let Ok(event) = rx_main.try_recv() {
                                        match event {
                                            WhatsAppEvent::Chat(wa_chat) => {
                                                println!(
                                                    "Received chat: {} ({})",
                                                    wa_chat.name.as_ref().unwrap_or(&wa_chat.id),
                                                    wa_chat.id
                                                );

                                                let contact = models::Contact {
                                                    jid: wa_chat.id.clone(),
                                                    name: wa_chat.name.unwrap_or_else(|| {
                                                        wa_chat
                                                            .id
                                                            .split('@')
                                                            .next()
                                                            .unwrap_or(&wa_chat.id)
                                                            .to_string()
                                                    }),
                                                    last_message: None,
                                                    last_message_time: None,
                                                    unread_count: wa_chat.unread_count.unwrap_or(0),
                                                    conversation_timestamp: wa_chat
                                                        .conversation_timestamp
                                                        .unwrap_or(0)
                                                        as i64,
                                                    is_group: wa_chat.id.contains("@g.us"),
                                                    archived: wa_chat.archived.unwrap_or(false),
                                                    pinned: wa_chat.pinned.unwrap_or(0),
                                                    mute_end_time: wa_chat
                                                        .mute_end_time
                                                        .unwrap_or(0),
                                                    profile_picture_url: None,
                                                };

                                                if let Err(e) = db_clone_ws.save_contact(&contact) {
                                                    eprintln!(
                                                        "Failed to save chat {}: {}",
                                                        contact.jid, e
                                                    );
                                                } else {
                                                    println!("Saved chat: {}", contact.name);
                                                    if let Ok(contacts) = db_clone_ws.get_contacts()
                                                    {
                                                        main_view_clone.update_contacts(contacts);
                                                    }
                                                }
                                            }
                                            WhatsAppEvent::Contact(wa_contact) => {
                                                println!(
                                                    "Received contact: {} ({})",
                                                    wa_contact
                                                        .name
                                                        .as_ref()
                                                        .unwrap_or(&wa_contact.id),
                                                    wa_contact.id
                                                );

                                                let contact = models::Contact {
                                                    jid: wa_contact.id.clone(),
                                                    name: wa_contact
                                                        .name
                                                        .or(wa_contact.notify)
                                                        .unwrap_or_else(|| {
                                                            wa_contact
                                                                .id
                                                                .split('@')
                                                                .next()
                                                                .unwrap_or(&wa_contact.id)
                                                                .to_string()
                                                        }),
                                                    last_message: None,
                                                    last_message_time: None,
                                                    unread_count: 0,
                                                    conversation_timestamp: 0,
                                                    is_group: wa_contact.id.contains("@g.us"),
                                                    archived: false,
                                                    pinned: 0,
                                                    mute_end_time: 0,
                                                    profile_picture_url: None,
                                                };

                                                if let Err(e) = db_clone_ws.save_contact(&contact) {
                                                    eprintln!("Failed to save contact: {}", e);
                                                } else {
                                                    println!("Saved contact: {}", contact.name);
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    glib::Continue(true)
                                },
                            );

                            window_clone.set_content(Some(&main_view.widget));
                        }
                        _ => {}
                    }
                }
                glib::Continue(true)
            });
        }

        window.present();
    });

    app.run();
}
