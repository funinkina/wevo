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
    main_row.set_margin_start(15);
    main_row.set_margin_end(15);
    main_row.set_margin_top(10);
    main_row.set_margin_bottom(10);

    // Avatar
    let avatar = widgets::create_avatar(&contact.initials(), &contact.avatar_color, 40);
    main_row.append(&avatar);

    // Contact info
    let info_box = Box::new(Orientation::Vertical, 5);
    info_box.set_hexpand(true);

    // Contact name and time
    let top_row = Box::new(Orientation::Horizontal, 10);

    let name = Label::new(Some(&contact.name));
    name.set_halign(gtk4::Align::Start);
    name.set_hexpand(true);
    name.add_css_class("heading");
    top_row.append(&name);

    let time = Label::new(Some(&contact.time));
    time.set_halign(gtk4::Align::End);
    time.add_css_class("dim-label");
    time.add_css_class("caption");
    top_row.append(&time);

    info_box.append(&top_row);

    // Last message
    let last_msg = Label::new(Some(&contact.last_message));
    last_msg.set_halign(gtk4::Align::Start);
    last_msg.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    last_msg.add_css_class("dim-label");
    info_box.append(&last_msg);

    main_row.append(&info_box);

    row.set_child(Some(&main_row));
    row
}
