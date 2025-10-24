// Preferences dialog for configuring URL and API key
use crate::config::ConfigManager;
use adwaita::prelude::*;
use gtk4::Button;

pub fn show_preferences_dialog(parent: &adwaita::ApplicationWindow) {
    let dialog = adwaita::PreferencesWindow::builder()
        .title("Preferences")
        .modal(true)
        .transient_for(parent)
        .build();

    // Create a preferences page
    let page = adwaita::PreferencesPage::builder()
        .title("Connection")
        .icon_name("network-wireless-symbolic")
        .build();

    // API Configuration Group
    let api_group = adwaita::PreferencesGroup::builder()
        .title("API Configuration")
        .description("Configure Evolution API connection settings")
        .build();

    // API URL row
    let url_row = adwaita::EntryRow::builder()
        .title("API URL")
        .text(&ConfigManager::get_url())
        .build();
    url_row.add_suffix(&gtk4::Image::from_icon_name("network-server-symbolic"));

    api_group.add(&url_row);

    // API Key row with visibility toggle
    let key_row = adwaita::PasswordEntryRow::builder()
        .title("API Key")
        .text(&ConfigManager::get_api_key())
        .build();
    key_row.add_suffix(&gtk4::Image::from_icon_name("dialog-password-symbolic"));

    api_group.add(&key_row);

    // Add action row for saving (optional, can also use dialog response)
    let save_row = adwaita::ActionRow::builder()
        .title("Save Settings")
        .subtitle("Changes are saved automatically")
        .build();

    let save_button = Button::builder()
        .label("Save")
        .valign(gtk4::Align::Center)
        .build();
    save_button.add_css_class("suggested-action");

    save_row.add_suffix(&save_button);
    api_group.add(&save_row);

    page.add(&api_group);
    dialog.add(&page);

    // Handle save button click
    let url_row_clone = url_row.clone();
    let key_row_clone = key_row.clone();
    let dialog_clone = dialog.clone();

    save_button.connect_clicked(move |_| {
        let url = url_row_clone.text().to_string();
        let api_key = key_row_clone.text().to_string();

        // Save to keyring
        if let Err(e) = ConfigManager::set_url(&url) {
            eprintln!("Failed to save URL: {}", e);
        }

        if let Err(e) = ConfigManager::set_api_key(&api_key) {
            eprintln!("Failed to save API key: {}", e);
        }

        println!("Configuration saved successfully!");
        dialog_clone.close();
    });

    dialog.present();
}
