use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gio::Cancellable;
use gtk4::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

/// Create an avatar widget with profile picture URL or initials fallback
pub fn create_avatar_with_pic(profile_pic_url: Option<&str>, name: &str, size: i32) -> adw::Avatar {
    let avatar = adw::Avatar::builder().size(size).text(name).build();

    if let Some(url) = profile_pic_url {
        // Clone for async operation
        let url_str = url.to_string();
        let avatar_clone = avatar.clone();

        // Download and set the profile picture asynchronously
        glib::MainContext::default().spawn_local(async move {
            if let Ok(texture) = download_and_create_texture(&url_str).await {
                avatar_clone.set_custom_image(Some(&texture));
            }
        });
    }

    avatar
}

/// Download image from URL and create a texture
async fn download_and_create_texture(
    url: &str,
) -> Result<gtk4::gdk::Texture, Box<dyn std::error::Error>> {
    // Use tokio for async HTTP request
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    // Create texture from bytes using gtk4::glib
    let bytes_glib = gtk4::glib::Bytes::from(&bytes.to_vec());
    let stream = gtk4::gio::MemoryInputStream::from_bytes(&bytes_glib);
    let pixbuf = Pixbuf::from_stream(&stream, None::<&Cancellable>)?;

    let texture = gtk4::gdk::Texture::for_pixbuf(&pixbuf);
    Ok(texture)
}

/// A button that displays the user's profile avatar and opens a menu
pub struct ProfileButton {
    button: gtk4::MenuButton,
    avatar: adw::Avatar,
    profile_pic_url: Rc<RefCell<Option<String>>>,
}

impl ProfileButton {
    pub fn new() -> Self {
        let avatar = adw::Avatar::builder().size(32).text("ME").build();

        let button = gtk4::MenuButton::new();
        button.set_icon_name("avatar-default-symbolic");
        button.add_css_class("flat");
        button.add_css_class("circular");

        // Create popover menu
        let menu = gtk4::gio::Menu::new();
        menu.append(Some("Preferences"), Some("win.preferences"));
        menu.append(Some("About"), Some("win.about"));

        let popover = gtk4::PopoverMenu::builder().menu_model(&menu).build();

        button.set_popover(Some(&popover));

        Self {
            button,
            avatar: avatar.clone(),
            profile_pic_url: Rc::new(RefCell::new(None)),
        }
    }

    /// Set the profile picture URL and update the avatar
    pub fn set_profile_pic(&self, url: Option<String>, name: Option<&str>) {
        *self.profile_pic_url.borrow_mut() = url.clone();

        if let Some(n) = name {
            self.avatar.set_text(Some(n));
        }

        if let Some(pic_url) = url {
            let avatar_clone = self.avatar.clone();
            gtk4::glib::MainContext::default().spawn_local(async move {
                if let Ok(texture) = download_and_create_texture(&pic_url).await {
                    avatar_clone.set_custom_image(Some(&texture));
                }
            });
        }
    }

    /// Get the underlying Button widget
    pub fn widget(&self) -> &gtk4::MenuButton {
        &self.button
    }
}

impl Default for ProfileButton {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate avatar color from string (for consistent colors)
pub fn generate_avatar_color(text: &str) -> (f64, f64, f64) {
    // Simple hash function to generate consistent colors
    let mut hash: u32 = 0;
    for byte in text.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }

    // Generate pleasant colors using HSL
    let hue = (hash % 360) as f64;
    let saturation = 0.5 + ((hash >> 8) % 30) as f64 / 100.0; // 0.5-0.8
    let lightness = 0.4 + ((hash >> 16) % 20) as f64 / 100.0; // 0.4-0.6

    hsl_to_rgb(hue, saturation, lightness)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r + m, g + m, b + m)
}
