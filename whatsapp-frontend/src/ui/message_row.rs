use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};

pub struct MessageRow {
    pub widget: GtkBox,
}

impl MessageRow {
    pub fn new(content: &str, is_from_me: bool, timestamp: i64) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 0);
        container.set_margin_start(10);
        container.set_margin_end(10);

        let bubble = GtkBox::new(Orientation::Vertical, 4);
        bubble.set_margin_start(8);
        bubble.set_margin_end(8);
        bubble.set_margin_top(4);
        bubble.set_margin_bottom(4);

        let content_label = Label::builder()
            .label(content)
            .wrap(true)
            .wrap_mode(gtk4::pango::WrapMode::WordChar)
            .xalign(0.0)
            .max_width_chars(50)
            .selectable(true)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .margin_bottom(4)
            .build();

        let time_str = Self::format_timestamp(timestamp);
        let time_label = Label::builder()
            .label(&time_str)
            .xalign(1.0) // Align time to the right
            .margin_start(12)
            .margin_end(12)
            .margin_top(0)
            .margin_bottom(6)
            .css_classes(vec!["caption", "dim-label"])
            .build();

        bubble.append(&content_label);
        bubble.append(&time_label);

        if is_from_me {
            // Own messages - align right with accent color background
            bubble.add_css_class("message-sent");
            container.set_halign(gtk4::Align::End);
        } else {
            // Other messages - align left with card background
            bubble.add_css_class("card");
            bubble.add_css_class("message-received");
            container.set_halign(gtk4::Align::Start);
        }

        container.append(&bubble);

        Self { widget: container }
    }

    fn format_timestamp(timestamp: i64) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let diff = now - timestamp;

        if diff < 60 {
            "Just now".to_string()
        } else if diff < 3600 {
            format!("{}m ago", diff / 60)
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else {
            format!("{}d ago", diff / 86400)
        }
    }
}
