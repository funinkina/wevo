use base64::{Engine as _, engine::general_purpose::STANDARD};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Image, Label, Orientation};
use libadwaita as adw;
use std::sync::{Arc, Mutex};

pub struct QrView {
    pub widget: adw::StatusPage,
    qr_image: Arc<Mutex<Option<Image>>>,
}

impl QrView {
    pub fn new() -> Self {
        let status_page = adw::StatusPage::builder()
            .icon_name("hourglass-symbolic")
            .title("Connecting to WhatsApp")
            .description("Waiting for QR code from server...")
            .build();

        Self {
            widget: status_page,
            qr_image: Arc::new(Mutex::new(None)),
        }
    }

    pub fn show_qr(&self, qr_data: &str) {
        println!("Rendering QR code in UI...");

        // Check if qr_data is a base64 data URL
        if qr_data.starts_with("data:image/png;base64,") {
            // Extract base64 part
            if let Some(base64_data) = qr_data.strip_prefix("data:image/png;base64,") {
                // Decode base64
                if let Ok(image_data) = STANDARD.decode(base64_data) {
                    // Create Pixbuf from image data using PixbufLoader
                    let loader = gdk_pixbuf::PixbufLoader::new();

                    if loader.write(&image_data).is_ok() && loader.close().is_ok() {
                        if let Some(pixbuf) = loader.pixbuf() {
                            // Create image widget
                            let qr_image = Image::from_pixbuf(Some(&pixbuf));
                            qr_image.set_pixel_size(256); // Set the image size

                            let vbox = GtkBox::new(Orientation::Vertical, 12);
                            vbox.set_halign(gtk4::Align::Center);
                            vbox.set_valign(gtk4::Align::Center);
                            vbox.append(&qr_image);

                            let instruction = Label::builder()
                                .label("Open WhatsApp on your phone\nTap Menu or Settings\nTap Linked Devices\nTap Link a Device")
                                .justify(gtk4::Justification::Center)
                                .css_classes(vec!["dim-label"])
                                .build();
                            vbox.append(&instruction);

                            self.widget.set_child(Some(&vbox));
                            println!("QR code image displayed successfully!");
                            return;
                        } else {
                            eprintln!("Failed to get pixbuf from loader");
                        }
                    } else {
                        eprintln!("Failed to load image data into pixbuf loader");
                    }
                } else {
                    eprintln!("Failed to decode base64 data");
                }
            }
        }

        // Fallback: show error message
        eprintln!("Failed to render QR code image");
        self.widget
            .set_description(Some("Error: Could not display QR code"));
    }

    pub fn show_connecting(&self) {
        self.widget.set_icon_name(Some("emblem-ok-symbolic"));
        self.widget.set_title("Connected!");
        self.widget.set_description(Some("Loading your chats..."));
        self.widget.set_child(gtk4::Widget::NONE);
    }
}
