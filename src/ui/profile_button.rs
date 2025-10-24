use gtk4::prelude::*;
use gtk4::{Button, PopoverMenu};
use std::cell::RefCell;

/// A button that displays the user's profile avatar and opens a menu
pub struct ProfileButton {
    button: Button,
    profile_pic_url: RefCell<Option<String>>,
}

impl ProfileButton {
    pub fn new() -> Self {
        // Use a regular Button instead of MenuButton for more flexibility
        let button = Button::builder().build();

        // Create avatar placeholder
        let avatar = crate::ui::widgets::create_avatar_with_pic(None, "ME", "#5E81AC", 32);
        button.set_child(Some(&avatar));

        // Create popover menu
        let menu = gtk4::gio::Menu::new();
        menu.append(Some("Preferences"), Some("win.preferences"));
        menu.append(Some("About Wevo"), Some("win.about"));

        let popover = PopoverMenu::builder().menu_model(&menu).build();

        // Connect button to show popover
        button.connect_clicked(move |btn| {
            popover.set_parent(btn);
            popover.popup();
        });

        Self {
            button,
            profile_pic_url: RefCell::new(None),
        }
    }

    /// Set the profile picture URL and update the avatar
    pub fn set_profile_pic(&self, url: Option<String>) {
        *self.profile_pic_url.borrow_mut() = url.clone();

        // Update the avatar widget
        let new_avatar =
            crate::ui::widgets::create_avatar_with_pic(url.as_deref(), "ME", "#5E81AC", 32);
        self.button.set_child(Some(&new_avatar));
    }

    /// Get the underlying Button widget
    pub fn widget(&self) -> &Button {
        &self.button
    }
}

impl Default for ProfileButton {
    fn default() -> Self {
        Self::new()
    }
}
