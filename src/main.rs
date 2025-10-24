mod config;
mod data;
mod models;
mod ui;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, HeaderBar, MenuButton, Orientation};

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
        .default_width(1200)
        .default_height(800)
        .build();

    // Get contacts from API (or sample data as fallback)
    let contacts = data::fetch_chats_or_fallback();

    // Get user profile info (first contact's info or default)
    let user_profile_pic = contacts
        .first()
        .and_then(|c| c.profile_pic_url.clone())
        .or_else(|| Some("https://via.placeholder.com/40".to_string()));

    // Create header bar with profile photo and preferences menu
    let header = HeaderBar::new();
    header.add_css_class("flat");

    // Left side - Profile photo/avatar
    let profile_avatar =
        ui::widgets::create_avatar_with_pic(user_profile_pic.as_deref(), "ME", "#5E81AC", 40);
    header.pack_start(&profile_avatar);

    // Right side - Preferences menu button (hamburger)
    let menu_button = MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .build();

    // Create popover menu
    let menu = gtk4::gio::Menu::new();
    menu.append(Some("Preferences"), Some("win.preferences"));

    let popover = gtk4::PopoverMenu::builder().menu_model(&menu).build();
    menu_button.set_popover(Some(&popover));

    header.pack_end(&menu_button);

    // Add preferences action
    let preferences_action = gtk4::gio::SimpleAction::new("preferences", None);
    let win_clone = win.clone();
    preferences_action.connect_activate(move |_, _| {
        ui::preferences::show_preferences_dialog(&win_clone);
    });
    win.add_action(&preferences_action);

    win.set_titlebar(Some(&header));

    // Create main horizontal layout (sidebar + content)
    let main_box = GtkBox::new(Orientation::Horizontal, 0);

    // Left sidebar - Contacts list (fixed width, no resizing)
    let paned_for_callback = main_box.clone();
    let contacts_box = ui::contacts::create_contacts_sidebar(contacts.clone(), move |contact| {
        // Fetch messages for the selected contact
        let messages = data::fetch_messages_or_fallback(&contact.remote_jid);

        // Create new conversation view
        let conversation_box = ui::conversation::create_conversation_view(contact, messages);

        // Remove old conversation if exists and add new one
        while let Some(child) = paned_for_callback.last_child() {
            if child != paned_for_callback.first_child().unwrap() {
                paned_for_callback.remove(&child);
            } else {
                break;
            }
        }
        paned_for_callback.append(&conversation_box);
    });
    main_box.append(&contacts_box);

    // Right side - Generic pane with icon and text (will be replaced when contact is selected)
    let generic_pane = create_generic_pane();
    main_box.append(&generic_pane);

    win.set_child(Some(&main_box));
    win.present();
}

fn create_generic_pane() -> gtk4::Box {
    let main_box = gtk4::Box::new(Orientation::Vertical, 10);
    main_box.set_valign(gtk4::Align::Center);
    main_box.set_halign(gtk4::Align::Center);

    // Icon (using a chat icon, assuming it's available; adjust if needed)
    let icon = gtk4::Image::from_icon_name("chat-bubbles-empty-symbolic");
    icon.set_pixel_size(64);
    main_box.append(&icon);

    // Text label
    let label = gtk4::Label::new(Some("Select a contact to start chatting"));
    label.add_css_class("title-3");
    main_box.append(&label);

    main_box
}
