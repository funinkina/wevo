use crate::models::contact::Contact;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};
use libadwaita as adw;

pub struct ContactRow {
    pub widget: GtkBox,
    pub jid: String,
}

impl ContactRow {
    pub fn new(contact: &Contact) -> Self {
        let row = GtkBox::new(Orientation::Horizontal, 12);
        row.set_margin_start(12);
        row.set_margin_end(12);
        row.set_margin_top(8);
        row.set_margin_bottom(8);

        // Profile picture/avatar
        let avatar = Self::create_avatar(contact);
        row.append(&avatar);

        // Middle: Contact info
        let middle_box = GtkBox::new(Orientation::Vertical, 4);
        middle_box.set_hexpand(true);
        middle_box.set_valign(gtk4::Align::Center);

        // Name with group indicator
        let mut display_name = contact.display_name();
        if contact.is_group {
            display_name = format!("👥 {}", display_name);
        }

        let name_label = Label::builder()
            .label(&display_name)
            .halign(gtk4::Align::Start)
            .css_classes(vec!["heading"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();

        middle_box.append(&name_label);

        // Last message preview
        if let Some(msg) = &contact.last_message {
            let msg_label = Label::builder()
                .label(msg)
                .halign(gtk4::Align::Start)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .max_width_chars(40)
                .css_classes(vec!["dim-label", "caption"])
                .build();
            middle_box.append(&msg_label);
        }

        row.append(&middle_box);

        // Right side: Metadata (timestamp, unread count)
        let right_box = GtkBox::new(Orientation::Vertical, 4);
        right_box.set_valign(gtk4::Align::Start);

        // Timestamp
        if contact.conversation_timestamp > 0 {
            let timestamp = format_timestamp(contact.conversation_timestamp);
            let time_label = Label::builder()
                .label(&timestamp)
                .halign(gtk4::Align::End)
                .css_classes(vec!["dim-label", "caption"])
                .build();
            right_box.append(&time_label);
        }

        // Unread count badge
        if contact.unread_count > 0 {
            let unread_label = Label::builder()
                .label(&contact.unread_count.to_string())
                .halign(gtk4::Align::End)
                .css_classes(vec!["badge", "accent"])
                .width_request(24)
                .height_request(24)
                .build();
            right_box.append(&unread_label);
        }

        // Muted indicator
        if contact.is_muted() {
            let mute_label = Label::builder()
                .label("🔇")
                .halign(gtk4::Align::End)
                .build();
            right_box.append(&mute_label);
        }

        row.append(&right_box);

        Self {
            widget: row,
            jid: contact.jid.clone(),
        }
    }

    fn create_avatar(contact: &Contact) -> adw::Avatar {
        let avatar = adw::Avatar::builder()
            .size(48)
            .text(&contact.display_name())
            .build();

        // Set icon name for groups
        if contact.is_group {
            avatar.set_icon_name(Some("system-users-symbolic"));
        } else if let Some(url) = &contact.profile_picture_url {
            // Load profile picture asynchronously
            let url_str = url.clone();
            let avatar_clone = avatar.clone();

            glib::MainContext::default().spawn_local(async move {
                if let Ok(texture) = Self::download_profile_picture(&url_str).await {
                    avatar_clone.set_custom_image(Some(&texture));
                }
            });
        }

        avatar
    }

    async fn download_profile_picture(
        url: &str,
    ) -> Result<gtk4::gdk::Texture, Box<dyn std::error::Error>> {
        use gtk4::gdk_pixbuf::Pixbuf;

        // Download image
        let response = reqwest::get(url).await?;
        let bytes = response.bytes().await?;

        // Create texture from bytes using GdkPixbuf with gtk4::glib
        let bytes_glib = gtk4::glib::Bytes::from(&bytes.to_vec());
        let stream = gtk4::gio::MemoryInputStream::from_bytes(&bytes_glib);

        // Load pixbuf synchronously (this is fine in glib spawn_local context)
        let pixbuf = Pixbuf::from_stream(&stream, gtk4::gio::Cancellable::NONE)?;
        let texture = gtk4::gdk::Texture::for_pixbuf(&pixbuf);

        Ok(texture)
    }
}

fn format_timestamp(timestamp_ms: i64) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let timestamp_secs = timestamp_ms / 1000;
    let msg_time = UNIX_EPOCH + Duration::from_secs(timestamp_secs as u64);
    let now = SystemTime::now();

    if let Ok(duration) = now.duration_since(msg_time) {
        let secs = duration.as_secs();

        if secs < 60 {
            return "Now".to_string();
        } else if secs < 3600 {
            let mins = secs / 60;
            return format!("{}m", mins);
        } else if secs < 86400 {
            let hours = secs / 3600;
            return format!("{}h", hours);
        } else if secs < 604800 {
            let days = secs / 86400;
            return format!("{}d", days);
        } else {
            let weeks = secs / 604800;
            return format!("{}w", weeks);
        }
    }

    "".to_string()
}
