use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gio::Cancellable;
use gtk4::prelude::*;
use gtk4::{Box, Image, Label, Orientation};

// Create an avatar widget with profile picture URL or initials fallback
pub fn create_avatar_with_pic(
    profile_pic_url: Option<&str>,
    initials: &str,
    color: &str,
    size: i32,
) -> Box {
    if let Some(url) = profile_pic_url {
        create_image_avatar(url, size)
    } else {
        create_avatar(initials, color, size)
    }
}

// Create an avatar widget with a profile picture from URL
fn create_image_avatar(url: &str, size: i32) -> Box {
    // Create a fixed-size container with overflow hidden
    let avatar = Box::new(Orientation::Horizontal, 0);
    avatar.set_size_request(size, size);
    avatar.set_hexpand(false);
    avatar.set_vexpand(false);
    avatar.set_halign(gtk4::Align::Center);
    avatar.set_valign(gtk4::Align::Center);
    avatar.set_overflow(gtk4::Overflow::Hidden);

    // Create an Image widget instead of Picture for better size control
    let image = Image::new();
    image.set_halign(gtk4::Align::Center);
    image.set_valign(gtk4::Align::Center);
    image.set_pixel_size(size);
    image.set_hexpand(false);
    image.set_vexpand(false);

    // Try to load and scale the image from URL
    // Download in a separate thread, then update UI on main thread
    let url_str = url.to_string();
    let size_i32 = size;
    let image_clone = image.clone();

    // Create channel for communication
    let (sender, receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        // Download image synchronously in background thread
        if let Ok(response) = reqwest::blocking::get(&url_str) {
            if let Ok(bytes) = response.bytes() {
                let _ = sender.send(bytes.to_vec());
            }
        }
    });

    // Poll for result on main thread
    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        if let Ok(bytes_vec) = receiver.try_recv() {
            let stream =
                gtk4::gio::MemoryInputStream::from_bytes(&gtk4::glib::Bytes::from(&bytes_vec));
            if let Ok(pixbuf) = Pixbuf::from_stream(&stream, None::<&Cancellable>) {
                // Scale pixbuf to exact size
                if let Some(scaled) =
                    pixbuf.scale_simple(size_i32, size_i32, gtk4::gdk_pixbuf::InterpType::Bilinear)
                {
                    image_clone.set_from_pixbuf(Some(&scaled));
                }
            }
            gtk4::glib::ControlFlow::Break
        } else {
            gtk4::glib::ControlFlow::Continue
        }
    });

    // Apply CSS for rounded corners
    avatar.add_css_class("avatar-container");
    let css = format!(
        ".avatar-container {{
            border-radius: {}px;
        }}",
        size / 2
    );

    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(&css);

    avatar
        .style_context()
        .add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

    avatar.append(&image);
    avatar
}

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
