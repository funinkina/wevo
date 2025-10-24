// Contacts list UI
use gtk4::prelude::*;
use gtk4::{
    ApplicationWindow, Box, Button, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow,
};
use std::rc::Rc;

use crate::models::Contact;
use crate::ui::widgets;

pub fn create_contacts_list<F>(
    contacts: Vec<Contact>,
    window: &ApplicationWindow,
    on_select: F,
) -> Box
where
    F: Fn(&Contact) + 'static,
{
    let main_box = Box::new(Orientation::Vertical, 0);
    // Set reasonable size constraints
    main_box.set_size_request(300, -1); // Minimum width of 300px
    main_box.set_hexpand(false); // Don't expand horizontally

    // Header
    let header = Box::new(Orientation::Horizontal, 10);
    header.set_margin_start(15);
    header.set_margin_end(15);
    header.set_margin_top(15);
    header.set_margin_bottom(10);

    let title = Label::new(Some("Contacts"));
    title.set_halign(gtk4::Align::Start);
    title.set_hexpand(true);
    title.add_css_class("title-2");
    header.append(&title);

    // Add preferences button
    let prefs_button = Button::from_icon_name("preferences-system-symbolic");
    prefs_button.set_tooltip_text(Some("Preferences"));
    let window_clone = window.clone();
    prefs_button.connect_clicked(move |_| {
        crate::ui::preferences::show_preferences_dialog(&window_clone);
    });
    header.append(&prefs_button);

    main_box.append(&header);

    // Separator
    let separator = gtk4::Separator::new(Orientation::Horizontal);
    main_box.append(&separator);

    // Contacts list
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .build();

    let list_box = ListBox::new();
    list_box.add_css_class("navigation-sidebar");

    // Store contacts with their rows for the callback
    let contacts_vec = Rc::new(contacts.clone());

    for contact in contacts {
        let row = create_contact_row(&contact);
        list_box.append(&row);
    }

    // Connect to row activation signal on the list box
    let on_select = Rc::new(on_select);
    list_box.connect_row_activated(move |_, row| {
        let index = row.index();
        if let Some(contact) = contacts_vec.get(index as usize) {
            on_select(contact);
        }
    });

    scrolled.set_child(Some(&list_box));
    main_box.append(&scrolled);

    main_box
}

fn create_contact_row(contact: &Contact) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.set_activatable(true);

    let main_row = Box::new(Orientation::Horizontal, 12);
    main_row.set_margin_start(5);
    main_row.set_margin_end(5);
    main_row.set_margin_top(10);
    main_row.set_margin_bottom(10);

    // Avatar with fixed size to prevent it from being cut off
    let avatar = widgets::create_avatar_with_pic(
        contact.profile_pic_url.as_deref(),
        &contact.initials(),
        &contact.avatar_color,
        40,
    );
    avatar.set_size_request(40, 40);
    main_row.append(&avatar);

    // Contact info with overflow handling
    let info_box = Box::new(Orientation::Vertical, 5);
    info_box.set_hexpand(true);
    info_box.set_overflow(gtk4::Overflow::Hidden); // Prevent overflow

    // Contact name and time
    let top_row = Box::new(Orientation::Horizontal, 10);

    let name = Label::new(Some(&contact.name));
    name.set_halign(gtk4::Align::Start);
    name.set_hexpand(true);
    name.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    name.set_max_width_chars(20); // Limit width to prevent overflow
    name.set_xalign(0.0); // Left align
    name.set_lines(1); // Single line only
    name.add_css_class("heading");
    top_row.append(&name);

    let time = Label::new(Some(&contact.time));
    time.set_halign(gtk4::Align::End);
    time.add_css_class("dim-label");
    time.add_css_class("caption");
    time.set_margin_start(5); // Ensure space between name and time
    top_row.append(&time);

    info_box.append(&top_row);

    // Last message with ellipsis
    let last_msg = Label::new(Some(&contact.last_message));
    last_msg.set_halign(gtk4::Align::Start);
    last_msg.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    last_msg.set_max_width_chars(30); // Limit width
    last_msg.set_xalign(0.0); // Left align
    last_msg.set_lines(1); // Single line only
    last_msg.set_single_line_mode(true); // Force single line
    last_msg.set_wrap(false); // Disable wrapping
    last_msg.add_css_class("dim-label");
    info_box.append(&last_msg);

    main_row.append(&info_box);

    row.set_child(Some(&main_row));
    row
}
