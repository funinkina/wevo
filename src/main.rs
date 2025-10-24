mod config;
mod data;
mod models;
mod ui;

use adwaita::prelude::*;
use gtk4::{Box as GtkBox, Label, MenuButton, Orientation, gio};
use std::cell::RefCell;
use std::rc::Rc;

use ui::profile_button::ProfileButton;

fn main() {
    // Initialize libadwaita
    adwaita::init().expect("Failed to initialize libadwaita");

    let app = adwaita::Application::builder()
        .application_id("com.funinkina.wevo")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adwaita::Application) {
    let win = adwaita::ApplicationWindow::builder()
        .application(app)
        .default_width(1200)
        .default_height(800)
        .build();

    // Get contacts from API (or sample data as fallback)
    let contacts = data::fetch_chats_or_fallback();

    // Get user profile picture from instance API
    let user_profile_pic = data::fetch_instance_or_none();

    // Create toolbar view for proper AdwApplicationWindow integration
    let toolbar_view = adwaita::ToolbarView::new();

    // Create header bar
    let header_bar = adwaita::HeaderBar::builder().build();

    // Left side - Profile button with menu
    let profile_button = ProfileButton::new();
    profile_button.set_profile_pic(user_profile_pic);
    header_bar.pack_start(profile_button.widget());

    // Center - Contact info (will be updated when contact is selected)
    let contact_title_box = GtkBox::new(Orientation::Horizontal, 8);
    contact_title_box.set_visible(false); // Hidden initially

    let contact_avatar_placeholder = Rc::new(RefCell::new(ui::widgets::create_avatar_with_pic(
        None, "", "#000000", 32,
    )));
    contact_title_box.append(&*contact_avatar_placeholder.borrow());

    let contact_name_label = Label::new(None);
    contact_name_label.add_css_class("title");
    contact_title_box.append(&contact_name_label);

    header_bar.set_title_widget(Some(&contact_title_box));

    // Right side - Menu button (for future use: call, video, info, etc.)
    let menu_button = MenuButton::builder()
        .icon_name("view-more-symbolic")
        .visible(false) // Hidden until a contact is selected
        .build();

    let conversation_menu = gio::Menu::new();
    conversation_menu.append(Some("Contact Info"), Some("win.contact-info"));

    let popover = gtk4::PopoverMenu::builder()
        .menu_model(&conversation_menu)
        .build();
    menu_button.set_popover(Some(&popover));

    header_bar.pack_end(&menu_button);

    // Add header bar to toolbar view
    toolbar_view.add_top_bar(&header_bar);

    // Add preferences action
    let preferences_action = gio::SimpleAction::new("preferences", None);
    let win_clone = win.clone();
    preferences_action.connect_activate(move |_, _| {
        ui::preferences::show_preferences_dialog(&win_clone);
    });
    win.add_action(&preferences_action);

    // Create main horizontal layout (sidebar + content)
    let main_box = GtkBox::new(Orientation::Horizontal, 0);

    // Create Rc wrappers for widgets that need to be updated from callbacks
    let contact_title_box_clone = contact_title_box.clone();
    let contact_name_label_clone = contact_name_label.clone();
    let menu_button_clone = menu_button.clone();
    let contact_avatar_clone = contact_avatar_placeholder.clone();

    // Left sidebar - Contacts list (fixed width, no resizing)
    let paned_for_callback = main_box.clone();
    let contacts_box = ui::contacts::create_contacts_sidebar(contacts.clone(), move |contact| {
        // Update header bar with selected contact info
        contact_title_box_clone.set_visible(true);
        contact_name_label_clone.set_text(&contact.name);
        menu_button_clone.set_visible(true);

        // Update avatar in header
        let new_avatar = ui::widgets::create_avatar_with_pic(
            contact.profile_pic_url.as_deref(),
            &contact.initials(),
            &contact.avatar_color,
            32,
        );

        // Remove old avatar and add new one
        if let Some(old_avatar) = contact_title_box_clone.first_child() {
            contact_title_box_clone.remove(&old_avatar);
        }
        contact_title_box_clone.prepend(&new_avatar);
        *contact_avatar_clone.borrow_mut() = new_avatar;

        // Fetch messages for the selected contact
        let messages = data::fetch_messages_or_fallback(&contact.remote_jid);

        // Create new conversation view (without header, as it's now in titlebar)
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

    // Set main content in toolbar view and then set toolbar view as window content
    toolbar_view.set_content(Some(&main_box));
    win.set_content(Some(&toolbar_view));
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
