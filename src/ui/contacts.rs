// Contacts list UI
use gtk4::prelude::*;
use gtk4::{Box, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow};
use std::rc::Rc;

use crate::models::Contact;
use crate::ui::widgets;

/// Create a modern sidebar for contacts list
pub fn create_contacts_sidebar<F>(contacts: Vec<Contact>, on_select: F) -> Box
where
    F: Fn(&Contact) + 'static,
{
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.set_size_request(350, -1); // Fixed width for sidebar
    main_box.set_hexpand(false);
    main_box.add_css_class("sidebar");

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
    main_row.set_margin_start(8);
    main_row.set_margin_end(8);
    main_row.set_margin_top(8);
    main_row.set_margin_bottom(8);

    // Avatar with fixed size to prevent it from being cut off
    let avatar = widgets::create_avatar_with_pic(
        contact.profile_pic_url.as_deref(),
        &contact.initials(),
        &contact.avatar_color,
        48,
    );
    avatar.set_size_request(48, 48);
    main_row.append(&avatar);

    // Contact info with overflow handling
    let info_box = Box::new(Orientation::Vertical, 4);
    info_box.set_hexpand(true);
    info_box.set_overflow(gtk4::Overflow::Hidden);
    info_box.set_valign(gtk4::Align::Center);

    // Contact name and time
    let top_row = Box::new(Orientation::Horizontal, 10);

    let name = Label::new(Some(&contact.name));
    name.set_halign(gtk4::Align::Start);
    name.set_hexpand(true);
    name.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    name.set_max_width_chars(22);
    name.set_xalign(0.0);
    name.set_lines(1);
    name.add_css_class("body");
    top_row.append(&name);

    let time = Label::new(Some(&contact.time));
    time.set_halign(gtk4::Align::End);
    time.add_css_class("dim-label");
    time.add_css_class("caption");
    time.set_margin_start(5);
    top_row.append(&time);

    info_box.append(&top_row);

    // Last message with ellipsis
    let last_msg = Label::new(Some(&contact.last_message));
    last_msg.set_halign(gtk4::Align::Start);
    last_msg.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    last_msg.set_max_width_chars(35);
    last_msg.set_xalign(0.0);
    last_msg.set_lines(1);
    last_msg.set_single_line_mode(true);
    last_msg.set_wrap(false);
    last_msg.add_css_class("dim-label");
    last_msg.add_css_class("caption");
    info_box.append(&last_msg);

    main_row.append(&info_box);

    row.set_child(Some(&main_row));
    row
}
