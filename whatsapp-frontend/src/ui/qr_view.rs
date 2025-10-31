use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Image, Label, Orientation};
use libadwaita as adw;
use qrcode::QrCode;
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
        // Generate QR code
        if let Ok(code) = QrCode::new(qr_data.as_bytes()) {
            let qr_string = code
                .render::<char>()
                .quiet_zone(false)
                .module_dimensions(2, 1)
                .build();

            // Create a label with monospace font to display QR code
            let qr_label = Label::builder()
                .label(&qr_string)
                .css_classes(vec!["monospace"])
                .build();

            let vbox = GtkBox::new(Orientation::Vertical, 12);
            vbox.set_halign(gtk4::Align::Center);
            vbox.set_valign(gtk4::Align::Center);
            vbox.append(&qr_label);

            let instruction = Label::builder()
                .label("Open WhatsApp on your phone\nTap Menu or Settings\nTap Linked Devices\nTap Link a Device")
                .justify(gtk4::Justification::Center)
                .css_classes(vec!["dim-label"])
                .build();
            vbox.append(&instruction);

            self.widget.set_child(Some(&vbox));
        }
    }

    pub fn show_connecting(&self) {
        self.widget.set_icon_name(Some("emblem-ok-symbolic"));
        self.widget.set_title("Connected!");
        self.widget.set_description(Some("Loading your chats..."));
        self.widget.set_child(gtk4::Widget::NONE);
    }
}
