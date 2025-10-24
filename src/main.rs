mod config;
mod data;
mod models;
mod ui;

use adwaita::prelude::*;
use gtk4::{Box as GtkBox, Label, MenuButton, Orientation, gio};
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

    // Add preferences action
    let preferences_action = gio::SimpleAction::new("preferences", None);
    let win_clone = win.clone();
    preferences_action.connect_activate(move |_, _| {
        ui::preferences::show_preferences_dialog(&win_clone);
    });
    win.add_action(&preferences_action);

    // Create AdwNavigationSplitView
    let split_view = adwaita::NavigationSplitView::builder()
        .max_sidebar_width(400.0)
        .min_sidebar_width(280.0)
        .sidebar_width_fraction(0.3)
        .build();

    // ===== SIDEBAR (Left) =====
    // Create sidebar toolbar view with its own header
    let sidebar_toolbar = adwaita::ToolbarView::new();
    let sidebar_header = adwaita::HeaderBar::builder().build();

    // Profile button in sidebar header
    let profile_button = ProfileButton::new();
    profile_button.set_profile_pic(user_profile_pic);
    sidebar_header.pack_start(profile_button.widget());

    // Sidebar title
    let sidebar_title = Label::new(Some("Chats"));
    sidebar_title.add_css_class("title");
    sidebar_header.set_title_widget(Some(&sidebar_title));

    sidebar_toolbar.add_top_bar(&sidebar_header);

    // Store split_view in Rc so we can access it from the callback
    let split_view_rc = Rc::new(split_view.clone());
    let split_view_for_callback = split_view_rc.clone();

    let contacts_list = ui::contacts::create_contacts_sidebar(contacts.clone(), move |contact| {
        // Create new header for content area with contact info
        let content_header = adwaita::HeaderBar::builder().build();

        // Contact info in content header
        let contact_title_box = GtkBox::new(Orientation::Horizontal, 8);

        let avatar = ui::widgets::create_avatar_with_pic(
            contact.profile_pic_url.as_deref(),
            &contact.initials(),
            &contact.avatar_color,
            32,
        );
        contact_title_box.append(&avatar);

        let contact_name_label = Label::new(Some(&contact.name));
        contact_name_label.add_css_class("title");
        contact_title_box.append(&contact_name_label);

        content_header.set_title_widget(Some(&contact_title_box));

        // Menu button for contact actions
        let menu_button = MenuButton::builder()
            .icon_name("view-more-symbolic")
            .build();

        let conversation_menu = gio::Menu::new();
        conversation_menu.append(Some("Contact Info"), Some("win.contact-info"));

        let popover = gtk4::PopoverMenu::builder()
            .menu_model(&conversation_menu)
            .build();
        menu_button.set_popover(Some(&popover));

        content_header.pack_end(&menu_button);

        // Fetch messages for the selected contact
        let messages = data::fetch_messages_or_fallback(&contact.remote_jid);

        // Create new conversation view
        let conversation_box = ui::conversation::create_conversation_view(contact, messages);

        // Create new toolbar with the contact's conversation
        let new_toolbar = adwaita::ToolbarView::new();
        new_toolbar.add_top_bar(&content_header);
        new_toolbar.set_content(Some(&conversation_box));

        // Create new navigation page for this conversation
        let new_content_page = adwaita::NavigationPage::builder()
            .title("Conversation")
            .child(&new_toolbar)
            .build();

        // Update the split view's content
        split_view_for_callback.set_content(Some(&new_content_page));
    });

    sidebar_toolbar.set_content(Some(&contacts_list));

    // Wrap sidebar in NavigationPage
    let sidebar_page = adwaita::NavigationPage::builder()
        .title("Sidebar")
        .child(&sidebar_toolbar)
        .build();

    split_view.set_sidebar(Some(&sidebar_page));

    // ===== CONTENT (Right) =====
    // Create initial content with generic pane
    let initial_content_header = adwaita::HeaderBar::builder().build();
    let initial_content_label = Label::new(Some("Wevo"));
    initial_content_label.add_css_class("title");
    initial_content_header.set_title_widget(Some(&initial_content_label));

    let initial_toolbar = adwaita::ToolbarView::new();
    initial_toolbar.add_top_bar(&initial_content_header);
    let generic_pane = create_generic_pane();
    initial_toolbar.set_content(Some(&generic_pane));

    let content_page = adwaita::NavigationPage::builder()
        .title("Content")
        .child(&initial_toolbar)
        .build();

    split_view.set_content(Some(&content_page));

    // Set split view as window content
    win.set_content(Some(&split_view));

    // Configure breakpoint for responsive design
    let breakpoint =
        adwaita::Breakpoint::new(adwaita::BreakpointCondition::parse("max-width: 600sp").unwrap());
    breakpoint.add_setter(&split_view, "collapsed", &true.to_value());
    win.add_breakpoint(breakpoint);

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
