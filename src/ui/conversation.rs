// Conversation view UI
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Label, Orientation, ScrolledWindow};
use std::sync::mpsc;

use crate::data;
use crate::models::{Contact, Message};

// Ensure CSS is loaded for message bubbles
fn ensure_css_loaded() {
    use gtk4::CssProvider;
    use gtk4::gdk::Display;

    static CSS_LOADED: std::sync::Once = std::sync::Once::new();

    CSS_LOADED.call_once(|| {
        let provider = CssProvider::new();
        provider.load_from_data(
            ".message-sent {
                background: @accent_bg_color;
                color: @accent_fg_color;
                border-radius: 16px;
                border-top-right-radius: 4px;
            }
            .card {
                border-radius: 16px;
                border-top-left-radius: 4px;
            }",
        );

        gtk4::style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });
}

pub fn create_conversation_view(contact: &Contact, messages: Vec<Message>) -> Box {
    ensure_css_loaded();

    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.set_hexpand(true);
    main_box.set_vexpand(true);

    // Messages area (header is now in the main window titlebar)
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .build();

    let messages_box = Box::new(Orientation::Vertical, 10);
    messages_box.set_margin_start(15);
    messages_box.set_margin_end(15);
    messages_box.set_margin_top(15);
    messages_box.set_margin_bottom(15);
    // messages_box.add_css_class("view");

    for message in messages {
        let message_widget = create_message_bubble(&message);
        messages_box.append(&message_widget);
    }

    scrolled.set_child(Some(&messages_box));

    // Scroll to bottom after the widget is realized
    let scrolled_clone = scrolled.clone();
    scrolled.connect_realize(move |_| {
        // Use idle_add to ensure this happens after layout
        let adj = scrolled_clone.vadjustment();
        glib::idle_add_local_once(move || {
            adj.set_value(adj.upper() - adj.page_size());
        });
    });

    main_box.append(&scrolled);

    // Input area with modern styling
    let input_container = Box::new(Orientation::Vertical, 0);

    let input_separator = gtk4::Separator::new(Orientation::Horizontal);
    input_container.append(&input_separator);

    let input_box = Box::new(Orientation::Horizontal, 8);
    input_box.set_margin_start(15);
    input_box.set_margin_end(15);
    input_box.set_margin_top(12);
    input_box.set_margin_bottom(12);

    // Attach button (emoji or file)
    let attach_button = Button::from_icon_name("list-add-symbolic");
    attach_button.add_css_class("flat");
    attach_button.add_css_class("circular");
    input_box.append(&attach_button);

    // Emoji button
    let emoji_button = Button::from_icon_name("face-smile-symbolic");
    emoji_button.add_css_class("flat");
    emoji_button.add_css_class("circular");
    input_box.append(&emoji_button);

    let input_entry = Entry::builder()
        .placeholder_text("Type a message...")
        .hexpand(true)
        .build();

    let send_button = Button::from_icon_name("mail-send-symbolic");
    send_button.add_css_class("suggested-action");
    send_button.add_css_class("circular");

    // Clone variables for closures
    let remote_jid = contact.remote_jid.clone();
    let messages_box_clone = messages_box.clone();
    let scrolled_clone = scrolled.clone();
    let input_entry_clone = input_entry.clone();

    // Helper function to send message
    let send_message_fn = move |text: String| {
        if text.trim().is_empty() {
            return;
        }

        let remote_jid_for_thread = remote_jid.clone();
        let messages_box = messages_box_clone.clone();
        let scrolled = scrolled_clone.clone();
        let input_entry = input_entry_clone.clone();

        // Create channel for communication between threads
        let (sender, receiver) = mpsc::channel();

        // Send message in a separate thread
        std::thread::spawn(move || {
            let result = data::send_message(&remote_jid_for_thread, &text);
            let _ = sender.send((result, text));
        });

        // Poll for response on main thread using idle_add
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if let Ok((result, text)) = receiver.try_recv() {
                match result {
                    Ok(_) => {
                        // Add message to UI on success
                        let now = chrono::Local::now();
                        let time_str = now.format("%H:%M").to_string();
                        let message = Message::new(text, time_str, true);

                        let message_widget = create_message_bubble(&message);
                        messages_box.append(&message_widget);

                        // Clear input
                        input_entry.set_text("");

                        // Scroll to bottom
                        let adj = scrolled.vadjustment();
                        glib::idle_add_local_once(move || {
                            adj.set_value(adj.upper() - adj.page_size());
                        });
                    }
                    Err(e) => {
                        eprintln!("Failed to send message: {}", e);
                        // TODO: Show error dialog to user
                    }
                }
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    };

    // Send button click handler
    let send_message_fn_clone = send_message_fn.clone();
    let input_entry_for_button = input_entry.clone();
    send_button.connect_clicked(move |_| {
        let text = input_entry_for_button.text().to_string();
        send_message_fn_clone(text);
    });

    // Enter key handler for input entry
    let input_entry_controller = gtk4::EventControllerKey::new();
    let input_entry_for_key = input_entry.clone();
    input_entry_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Return || key == gtk4::gdk::Key::KP_Enter {
            let text = input_entry_for_key.text().to_string();
            send_message_fn(text);
            return gtk4::glib::Propagation::Stop;
        }
        gtk4::glib::Propagation::Proceed
    });
    input_entry.add_controller(input_entry_controller);

    input_box.append(&input_entry);
    input_box.append(&send_button);

    input_container.append(&input_box);
    main_box.append(&input_container);

    main_box
}

fn create_message_bubble(message: &Message) -> Box {
    let container = Box::new(Orientation::Horizontal, 0);
    container.set_margin_start(10);
    container.set_margin_end(10);

    let bubble = Box::new(Orientation::Vertical, 4);
    bubble.set_margin_start(8);
    bubble.set_margin_end(8);
    bubble.set_margin_top(4);
    bubble.set_margin_bottom(4);

    let content = Label::new(Some(&message.content));
    content.set_wrap(true);
    content.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    content.set_xalign(0.0);
    content.set_max_width_chars(50);
    content.set_selectable(true);

    // Add padding around the message text
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_margin_top(8);
    content.set_margin_bottom(4);

    let time = Label::new(Some(&message.time));
    time.add_css_class("caption");
    time.add_css_class("dim-label");
    time.set_xalign(1.0); // Align time to the right
    time.set_margin_start(12);
    time.set_margin_end(12);
    time.set_margin_top(0);
    time.set_margin_bottom(6);

    bubble.append(&content);
    bubble.append(&time);

    if message.is_own {
        // Own messages - align right with accent color background
        bubble.add_css_class("message-sent");
        container.set_halign(gtk4::Align::End);
        container.append(&bubble);
    } else {
        // Other messages - align left with card background
        bubble.add_css_class("card");
        container.set_halign(gtk4::Align::Start);
        container.append(&bubble);
    }

    container
}
