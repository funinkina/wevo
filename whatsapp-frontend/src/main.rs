mod models;
mod services;
mod ui;

use adw::prelude::*;
use gtk4::prelude::*;
use libadwaita as adw;
use std::sync::Arc;

use models::Database;
use services::ws_client::WhatsAppEvent;
use services::{ApiClient, WebSocketClient};
use ui::{MainView, QrView};

fn main() {
    let app = adw::Application::builder()
        .application_id("org.aryan.whatsappgtk")
        .build();

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

            // Load contacts from database
            main_view.load_contacts();

            // Setup WebSocket for receiving messages
            let (_ws, rx) = WebSocketClient::new("ws://localhost:8787");
            let main_view_clone = Arc::new(main_view);
            let db_clone = Arc::clone(&db);

            // Handle incoming WebSocket events
            glib::timeout_add_local(std::time::Duration::from_millis(100), {
                let main_view = Arc::clone(&main_view_clone);
                move || {
                    while let Ok(event) = rx.try_recv() {
                        match event {
                            WhatsAppEvent::Message {
                                jid,
                                sender,
                                content,
                                timestamp,
                                is_from_me,
                            } => {
                                println!("📨 [main.rs] Received message event for: {}", jid);
                                main_view
                                    .add_message(&jid, &sender, &content, timestamp, is_from_me);
                            }
                            WhatsAppEvent::ContactsUpdate(contacts) => {
                                println!(
                                    "👥 [main.rs] Received ContactsUpdate with {} contacts",
                                    contacts.len()
                                );
                                for contact in &contacts {
                                    println!(
                                        "  - Saving contact: {} ({})",
                                        contact.name, contact.jid
                                    );
                                    match db_clone.save_contact(&contact) {
                                        Ok(_) => println!("    ✅ Saved successfully"),
                                        Err(e) => eprintln!("    ❌ Failed to save: {}", e),
                                    }
                                }
                                match db_clone.get_contacts() {
                                    Ok(saved_contacts) => {
                                        println!(
                                            "👥 [main.rs] Retrieved {} contacts from DB",
                                            saved_contacts.len()
                                        );
                                        main_view.update_contacts(saved_contacts);
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "❌ [main.rs] Failed to get contacts from DB: {}",
                                            e
                                        );
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
            main_view_clone.setup_send_handler({
                let api = Arc::clone(&api);
                let main_view = Arc::clone(&main_view_clone);
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

            window.set_content(Some(&main_view_clone.widget));
        } else {
            // Show QR view
            let qr_view = Arc::new(QrView::new());
            window.set_content(Some(&qr_view.widget));

            // Request QR code from backend
            let api_clone_qr = Arc::clone(&api);
            std::thread::spawn(move || {
                println!("📱 Requesting QR code from backend...");
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
                            println!("✅ QR Code received from backend");
                            qr_view_clone.show_qr(&qr);
                        }
                        WhatsAppEvent::Connected => {
                            println!("✅ Connected to WhatsApp");
                            qr_view_clone.show_connecting();

                            // Mark as authenticated
                            let _ = db_clone.set_authenticated(true);

                            // Transition to main view
                            let main_view =
                                MainView::new(Arc::clone(&db_clone), Arc::clone(&api_clone));

                            // Try to fetch contacts
                            if let Ok(contacts) = api_clone.get_contacts() {
                                for contact in &contacts {
                                    let _ = db_clone.save_contact(contact);
                                }
                                main_view.update_contacts(contacts);
                            }

                            main_view.load_contacts();
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
