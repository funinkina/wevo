// Conversation view UI
use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Label, Orientation, ScrolledWindow};

use crate::models::{Contact, Message};
use crate::ui::widgets;

pub fn create_conversation_view(contact: &Contact, messages: Vec<Message>) -> Box {
    let main_box = Box::new(Orientation::Vertical, 0);

    // Header with contact name and avatar
    let header = Box::new(Orientation::Horizontal, 12);
    header.set_margin_start(15);
    header.set_margin_end(15);
    header.set_margin_top(15);
    header.set_margin_bottom(10);

    // Avatar
    let avatar = widgets::create_avatar(&contact.initials(), &contact.avatar_color, 32);
    header.append(&avatar);

    // Contact name
    let contact_name = Label::new(Some(&contact.name));
    contact_name.set_halign(gtk4::Align::Start);
    contact_name.add_css_class("title-2");
    header.append(&contact_name);

    main_box.append(&header);

    // Separator
    let separator = gtk4::Separator::new(Orientation::Horizontal);
    main_box.append(&separator);

    // Messages area
    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .build();

    let messages_box = Box::new(Orientation::Vertical, 10);
    messages_box.set_margin_start(15);
    messages_box.set_margin_end(15);
    messages_box.set_margin_top(15);
    messages_box.set_margin_bottom(15);

    for message in messages {
        let message_widget = create_message_bubble(&message);
        messages_box.append(&message_widget);
    }

    scrolled.set_child(Some(&messages_box));

    // Scroll to bottom after the widget is realized
    let scrolled_clone = scrolled.clone();
    scrolled.connect_realize(move |_| {
        // Use idle_add to ensure this happens after layout
        let adj = scrolled_clone.vadjustment();
        glib::idle_add_local_once(move || {
            adj.set_value(adj.upper() - adj.page_size());
        });
    });

    main_box.append(&scrolled);

    // Input area
    let input_box = Box::new(Orientation::Horizontal, 10);
    input_box.set_margin_start(15);
    input_box.set_margin_end(15);
    input_box.set_margin_top(10);
    input_box.set_margin_bottom(15);

    let input_entry = Entry::builder()
        .placeholder_text("Type a message...")
        .hexpand(true)
        .build();

    let send_button = Button::builder().label("Send").build();
    send_button.add_css_class("suggested-action");

    input_box.append(&input_entry);
    input_box.append(&send_button);

    main_box.append(&input_box);

    main_box
}

fn create_message_bubble(message: &Message) -> Box {
    let container = Box::new(Orientation::Horizontal, 0);

    let bubble = Box::new(Orientation::Vertical, 5);
    bubble.set_margin_start(10);
    bubble.set_margin_end(10);
    bubble.set_margin_top(5);
    bubble.set_margin_bottom(5);

    let content = Label::new(Some(&message.content));
    content.set_wrap(true);
    content.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    content.set_xalign(0.0);
    content.set_max_width_chars(50);

    // Add padding around the message text
    content.set_margin_start(10);
    content.set_margin_end(10);
    content.set_margin_top(10);
    content.set_margin_bottom(10);

    let time = Label::new(Some(&message.time));
    time.add_css_class("caption");
    time.add_css_class("dim-label");
    time.set_xalign(0.0);
    // Add padding around the time text
    time.set_margin_start(10);
    time.set_margin_end(10);
    time.set_margin_top(0);
    time.set_margin_bottom(5);

    bubble.append(&content);
    bubble.append(&time);

    if message.is_own {
        // Own messages - align right with blue background
        bubble.add_css_class("accent");
        bubble.add_css_class("card");
        container.set_halign(gtk4::Align::End);
        container.append(&bubble);
    } else {
        // Other messages - align left with default background
        bubble.add_css_class("card");
        container.set_halign(gtk4::Align::Start);
        container.append(&bubble);
    }

    container
}
