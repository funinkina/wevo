mod config;
mod data;
mod models;
mod ui;

use gtk4::gio::{Menu, SimpleAction};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Orientation, Paned};

fn main() {
    let app = Application::builder()
        .application_id("com.funinkina.wevo")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let win = ApplicationWindow::builder()
        .application(app)
        .title("Wevo Chat")
        .default_width(1200)
        .default_height(800)
        .build();

    // Add preferences action
    let preferences_action = SimpleAction::new("preferences", None);
    let win_clone = win.clone();
    preferences_action.connect_activate(move |_, _| {
        ui::preferences::show_preferences_dialog(&win_clone);
    });
    win.add_action(&preferences_action);

    // Add menu
    let menu = Menu::new();
    menu.append(Some("Preferences"), Some("win.preferences"));

    // You can add the menu to the app's menubar or as a popover
    // For now, we'll add it as an application menu
    app.set_menubar(Some(&menu));

    // Create main horizontal paned (split view)
    let paned = Paned::builder()
        .orientation(Orientation::Horizontal)
        .wide_handle(true)
        .position(350)
        .build();

    // Get contacts from API (or sample data as fallback)
    let contacts = data::fetch_chats_or_fallback();

    // Left side - Contacts list with callback to update conversation view
    let paned_for_callback = paned.clone();
    let contacts_box = ui::contacts::create_contacts_list(contacts.clone(), &win, move |contact| {
        // Fetch messages for the selected contact
        let messages = data::fetch_messages_or_fallback(&contact.remote_jid);

        // Create new conversation view
        let conversation_box = ui::conversation::create_conversation_view(contact, messages);

        // Update the right side of the paned
        paned_for_callback.set_end_child(Some(&conversation_box));
    });
    paned.set_start_child(Some(&contacts_box));

    // Right side - Empty initially or with first contact
    if !contacts.is_empty() {
        let messages = data::fetch_messages_or_fallback(&contacts[0].remote_jid);
        let conversation_box = ui::conversation::create_conversation_view(&contacts[0], messages);
        paned.set_end_child(Some(&conversation_box));
    }

    win.set_child(Some(&paned));
    win.show();
}
