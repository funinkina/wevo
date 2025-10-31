use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Entry, ListBox, Orientation, ScrolledWindow};
use libadwaita as adw;
use std::sync::{Arc, Mutex};

use super::{ContactRow, MessageRow};
use crate::models::{Contact, Database};
use crate::services::ApiClient;

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
            .message-received {
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

pub struct MainView {
    pub widget: adw::OverlaySplitView,
    contacts_list: ListBox,
    messages_box: GtkBox,
    messages_scrolled: ScrolledWindow,
    message_entry: Entry,
    send_button: Button,
    chat_title: adw::WindowTitle,
    current_contact: Arc<Mutex<Option<String>>>,
    db: Arc<Database>,
    #[allow(dead_code)]
    api: Arc<ApiClient>,
}

impl MainView {
    pub fn new(db: Arc<Database>, api: Arc<ApiClient>) -> Self {
        ensure_css_loaded();

        // Sidebar - Contacts list
        let sidebar = GtkBox::new(Orientation::Vertical, 0);

        // Header bar at the top - FIXED position (not scrollable)
        let header = adw::HeaderBar::new();
        header.add_css_class("flat");
        let title = adw::WindowTitle::new("Chats", "");
        header.set_title_widget(Some(&title));

        // Add search button to the header
        let search_button = Button::builder()
            .icon_name("system-search-symbolic")
            .tooltip_text("Search contacts")
            .build();
        search_button.add_css_class("flat");
        header.pack_end(&search_button);

        sidebar.append(&header);

        // Contacts list in scrolled window
        let contacts_list = ListBox::new();
        contacts_list.set_css_classes(&["navigation-sidebar"]);
        contacts_list.set_selection_mode(gtk4::SelectionMode::None);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .child(&contacts_list)
            .build();

        sidebar.append(&scrolled);

        // Content - Chat view
        let content = GtkBox::new(Orientation::Vertical, 0);

        let chat_header = adw::HeaderBar::new();
        chat_header.add_css_class("flat");
        let chat_title = adw::WindowTitle::new("Select a chat", "");
        chat_header.set_title_widget(Some(&chat_title));
        content.append(&chat_header);

        // Messages area with Box instead of ListBox
        let messages_scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .build();

        let messages_box = GtkBox::new(Orientation::Vertical, 10);
        messages_box.set_margin_start(15);
        messages_box.set_margin_end(15);
        messages_box.set_margin_top(15);
        messages_box.set_margin_bottom(15);

        messages_scrolled.set_child(Some(&messages_box));
        content.append(&messages_scrolled);

        // Input area with modern styling
        let input_container = GtkBox::new(Orientation::Vertical, 0);

        let input_separator = gtk4::Separator::new(Orientation::Horizontal);
        input_container.append(&input_separator);

        let input_box = GtkBox::new(Orientation::Horizontal, 8);
        input_box.set_margin_start(15);
        input_box.set_margin_end(15);
        input_box.set_margin_top(12);
        input_box.set_margin_bottom(12);

        // Attach button
        let attach_button = Button::from_icon_name("list-add-symbolic");
        attach_button.add_css_class("flat");
        attach_button.add_css_class("circular");
        input_box.append(&attach_button);

        // Emoji button
        let emoji_button = Button::from_icon_name("face-smile-symbolic");
        emoji_button.add_css_class("flat");
        emoji_button.add_css_class("circular");
        input_box.append(&emoji_button);

        let message_entry = Entry::builder()
            .placeholder_text("Type a message...")
            .hexpand(true)
            .build();

        let send_button = Button::builder()
            .icon_name("mail-send-symbolic")
            .css_classes(vec!["suggested-action", "circular"])
            .sensitive(false)
            .build();

        input_box.append(&message_entry);
        input_box.append(&send_button);

        input_container.append(&input_box);
        content.append(&input_container);

        // Create OverlaySplitView
        let split_view = adw::OverlaySplitView::new();
        split_view.set_sidebar(Some(&sidebar));
        split_view.set_content(Some(&content));
        split_view.set_show_sidebar(true);
        split_view.set_collapsed(false);
        split_view.set_sidebar_width_fraction(0.3);
        split_view.set_min_sidebar_width(280.0);
        split_view.set_max_sidebar_width(400.0);

        let main_view = Self {
            widget: split_view,
            contacts_list: contacts_list.clone(),
            messages_box: messages_box.clone(),
            messages_scrolled: messages_scrolled.clone(),
            message_entry: message_entry.clone(),
            send_button: send_button.clone(),
            chat_title: chat_title.clone(),
            current_contact: Arc::new(Mutex::new(None)),
            db: Arc::clone(&db),
            api,
        };

        // Connect signals - clone everything we need before moving
        let current_contact_clone = Arc::clone(&main_view.current_contact);
        let send_button_clone = send_button.clone();
        message_entry.connect_changed(move |entry| {
            let has_text = !entry.text().is_empty();
            let has_contact = current_contact_clone.lock().unwrap().is_some();
            send_button_clone.set_sensitive(has_text && has_contact);
        });

        // Connect row activation handler once
        let messages_box_clone = messages_box.clone();
        let messages_scrolled_clone = messages_scrolled.clone();
        let db_clone = Arc::clone(&db);
        let current_contact_clone2 = Arc::clone(&main_view.current_contact);
        let message_entry_clone = main_view.message_entry.clone();
        let send_button_clone2 = main_view.send_button.clone();
        let chat_title_clone = main_view.chat_title.clone();

        contacts_list.connect_row_activated(move |_, row| {
            // Get the JID from the row's widget name
            let jid = row.widget_name();
            println!("[MainView] Row activated, JID: {}", jid);

            if !jid.is_empty() {
                // Update the chat title to show the contact name
                let display_name = if jid.contains("@g.us") {
                    // It's a group - extract name from JID
                    jid.split('@').next().unwrap_or(&jid).to_string()
                } else {
                    // It's an individual contact - extract phone number
                    jid.split('@').next().unwrap_or(&jid).to_string()
                };
                chat_title_clone.set_title(&display_name);

                *current_contact_clone2.lock().unwrap() = Some(jid.to_string());
                Self::load_messages_static(
                    &messages_box_clone,
                    &messages_scrolled_clone,
                    &db_clone,
                    &jid,
                );
                send_button_clone2.set_sensitive(!message_entry_clone.text().is_empty());
            } else {
                println!("[MainView] Warning: Row has no JID set!");
            }
        });

        main_view
    }

    pub fn load_contacts(&self) {
        if let Ok(contacts) = self.db.get_contacts() {
            self.update_contacts(contacts);
        }
    }

    pub fn update_contacts(&self, contacts: Vec<Contact>) {
        println!(
            "[MainView] update_contacts called with {} contacts",
            contacts.len()
        );

        // Clear existing
        while let Some(child) = self.contacts_list.first_child() {
            self.contacts_list.remove(&child);
        }
        println!("[MainView] Cleared existing contacts from list");

        for contact in contacts {
            let contact_row = ContactRow::new(&contact);

            // Use ListBoxRow directly - no button wrapper
            let row = gtk4::ListBoxRow::new();
            row.set_child(Some(&contact_row.widget));
            row.set_activatable(true);

            // Store the JID in the row's name so we can retrieve it later
            row.set_widget_name(&contact.jid);

            self.contacts_list.append(&row);
        }

        println!("[MainView] Finished updating contacts list");
    }
    fn load_messages_static(
        messages_box: &GtkBox,
        messages_scrolled: &ScrolledWindow,
        db: &Database,
        jid: &str,
    ) {
        println!("[MainView] load_messages_static called for JID: {}", jid);

        // Clear existing messages
        while let Some(child) = messages_box.first_child() {
            messages_box.remove(&child);
        }
        println!("[MainView] Cleared existing messages");

        // Load and display messages
        match db.get_messages(jid) {
            Ok(messages) => {
                println!("[MainView] Loaded {} messages for {}", messages.len(), jid);
                for msg in messages {
                    let row = MessageRow::new(&msg.content, msg.is_from_me, msg.timestamp);
                    messages_box.append(&row.widget);
                }
            }
            Err(e) => {
                println!("[MainView] Error loading messages for {}: {}", jid, e);
            }
        }

        // Scroll to bottom after messages are loaded
        let scrolled_clone = messages_scrolled.clone();
        glib::idle_add_local_once(move || {
            let adj = scrolled_clone.vadjustment();
            adj.set_value(adj.upper() - adj.page_size());
        });
    }

    pub fn add_message(
        &self,
        jid: &str,
        sender: &str,
        content: &str,
        timestamp: i64,
        is_from_me: bool,
    ) {
        // Save to DB
        let message = crate::models::Message {
            id: None,
            jid: jid.to_string(),
            sender: sender.to_string(),
            content: content.to_string(),
            timestamp,
            is_from_me,
        };
        let _ = self.db.save_message(&message);

        // Update UI if this is the current chat
        if let Some(current) = self.current_contact.lock().unwrap().as_ref() {
            if current == jid {
                let row = MessageRow::new(content, is_from_me, timestamp);
                self.messages_box.append(&row.widget);

                // Scroll to bottom
                let scrolled = self.messages_scrolled.clone();
                glib::idle_add_local_once(move || {
                    let adj = scrolled.vadjustment();
                    adj.set_value(adj.upper() - adj.page_size());
                });
            }
        }
    }

    pub fn setup_send_handler<F>(&self, callback: F)
    where
        F: Fn(String, String) + 'static,
    {
        let message_entry = self.message_entry.clone();
        let current_contact = Arc::clone(&self.current_contact);

        self.send_button.connect_clicked(move |_| {
            let text = message_entry.text().to_string();
            if let Some(jid) = current_contact.lock().unwrap().as_ref() {
                if !text.is_empty() {
                    callback(jid.clone(), text.clone());
                    message_entry.set_text("");
                }
            }
        });
    }
}
