// Reusable UI widgets
use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};

/// Create an avatar widget with initials
pub fn create_avatar(initials: &str, color: &str, size: i32) -> Box {
    let avatar = Box::new(Orientation::Horizontal, 0);
    avatar.set_width_request(size);
    avatar.set_height_request(size);
    avatar.set_halign(gtk4::Align::Center);
    avatar.set_valign(gtk4::Align::Center);

    let label = Label::new(Some(initials));
    label.set_halign(gtk4::Align::Center);
    label.set_valign(gtk4::Align::Center);

    // Add CSS classes for styling
    avatar.add_css_class("avatar");
    if size <= 40 {
        label.add_css_class("caption");
    }

    // Apply inline CSS for background color
    let css = format!(
        ".avatar {{ 
            background-color: {}; 
            border-radius: {}px;
            min-width: {}px;
            min-height: {}px;
        }}",
        color,
        size / 2,
        size,
        size
    );

    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(&css);

    avatar
        .style_context()
        .add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

    label.set_markup(&format!(
        "<span foreground='white' weight='bold'>{}</span>",
        initials
    ));

    avatar.append(&label);
    avatar
}
