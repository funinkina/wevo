// Preferences dialog for configuring URL and API key
use crate::config::ConfigManager;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box, Button, Dialog, Entry, Label, Orientation, ResponseType};

pub fn show_preferences_dialog(parent: &ApplicationWindow) {
    let dialog = Dialog::builder()
        .title("Preferences")
        .modal(true)
        .transient_for(parent)
        .default_width(500)
        .default_height(250)
        .build();

    // Add buttons
    dialog.add_button("Cancel", ResponseType::Cancel);
    dialog.add_button("Save", ResponseType::Accept);

    // Create content area
    let content_area = dialog.content_area();
    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    // API URL section
    let url_label = Label::new(Some("API URL:"));
    url_label.set_halign(gtk4::Align::Start);
    main_box.append(&url_label);

    let url_entry = Entry::new();
    url_entry.set_placeholder_text(Some("http://localhost:8080"));
    url_entry.set_text(&ConfigManager::get_url());
    main_box.append(&url_entry);

    // Add some spacing
    let spacer = Box::new(Orientation::Vertical, 0);
    spacer.set_height_request(8);
    main_box.append(&spacer);

    // API Key section
    let key_label = Label::new(Some("API Key:"));
    key_label.set_halign(gtk4::Align::Start);
    main_box.append(&key_label);

    let key_entry = Entry::new();
    key_entry.set_placeholder_text(Some("Enter your API key"));
    key_entry.set_text(&ConfigManager::get_api_key());
    key_entry.set_visibility(false); // Hide the API key text
    main_box.append(&key_entry);

    // Show/Hide API key toggle
    let show_key_box = Box::new(Orientation::Horizontal, 6);
    show_key_box.set_margin_top(4);

    let show_key_button = Button::with_label("Show API Key");
    show_key_button.set_halign(gtk4::Align::Start);

    let key_entry_clone = key_entry.clone();
    let show_key_button_clone = show_key_button.clone();

    // Use a RefCell to track visibility state
    use std::cell::RefCell;
    use std::rc::Rc;
    let is_visible = Rc::new(RefCell::new(false));
    let is_visible_clone = is_visible.clone();

    show_key_button.connect_clicked(move |_| {
        let mut visible = is_visible_clone.borrow_mut();
        *visible = !*visible;
        key_entry_clone.set_visibility(*visible);
        if *visible {
            show_key_button_clone.set_label("Hide API Key");
        } else {
            show_key_button_clone.set_label("Show API Key");
        }
    });

    show_key_box.append(&show_key_button);
    main_box.append(&show_key_box);

    content_area.append(&main_box);

    // Handle dialog response
    let url_entry_clone = url_entry.clone();
    let key_entry_clone = key_entry.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let url = url_entry_clone.text().to_string();
            let api_key = key_entry_clone.text().to_string();

            // Save to keyring
            if let Err(e) = ConfigManager::set_url(&url) {
                eprintln!("Failed to save URL: {}", e);
            }

            if let Err(e) = ConfigManager::set_api_key(&api_key) {
                eprintln!("Failed to save API key: {}", e);
            }

            println!("Configuration saved successfully!");
        }
        dialog.close();
    });

    dialog.show();
}
