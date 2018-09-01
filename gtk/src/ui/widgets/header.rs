use utils::{set_class, set_margin};
use gtk::prelude::*;
use gtk;
use std::ops::Deref;

#[derive(Clone)]
pub struct Header {
    container:          gtk::HeaderBar,
    pub font_size:      gtk::SpinButton,
    pub install:        gtk::Button,
    pub uninstall:      gtk::Button,
    pub show_installed: gtk::CheckButton,
    pub dark_preview:   gtk::CheckButton,
}

macro_rules! button {
    ($label:expr) => {
        gtk::Button::new_with_label($label)
    };
}

impl AsRef<gtk::HeaderBar> for Header {
    fn as_ref(&self) -> &gtk::HeaderBar {
        &self.container
    }
}

impl Deref for Header {
    type Target = gtk::HeaderBar;
    fn deref(&self) -> &gtk::HeaderBar {
        &self.container
    }
}

impl Header {
    pub fn new() -> Header {
        // Buttons for installing and uninstalling fonts.
        let (install, uninstall) = (button!("Install"), button!("Uninstall"));

        // Set styles for those buttons.
        set_class(&install, "suggested-action");
        set_class(&uninstall, "destructive-action");

        // Add a font size spin button.
        let font_size = gtk::SpinButton::new(&gtk::Adjustment::new(1.5, 1.0, 50.0, 0.25, 0.0, 0.0), 0.1, 2);
        let dark_preview = gtk::CheckButton::new_with_label("Dark Preview");
        let show_installed = gtk::CheckButton::new_with_label("Installed");
        show_installed.set_active(true);

        // The settings menu, contained within a vertical box.
        let menu_box = cascade! {
            menu_box: gtk::Box::new(gtk::Orientation::Vertical, 5);
            ..pack_start(&gtk::Label::new("Show"), false, false, 0);
            ..pack_start(&show_installed, false, false, 0);
            ..pack_start(&gtk::Separator::new(gtk::Orientation::Horizontal), false, false, 0);
            ..pack_start(&gtk::Label::new("Preview"), false, false, 0);
            ..pack_start(&dark_preview, false, false, 0);
            | set_margin(&menu_box, 5, 5, 5, 5);
        };

        // Create the popover menu for the settings menu button.
        let popover = cascade! {
            gtk::PopoverMenu::new();
            ..add(&menu_box);
            | menu_box.show_all();
        };

        // Attach the popover to the settings menu button.
        let settings = cascade! {
            gtk::MenuButton::new();
            ..set_image(&gtk::Image::new_from_icon_name("preferences-system", 0));
            ..set_popover(&popover);
            ..set_use_popover(true);
        };

        // Attach everything to the headerbar
        let container = cascade! {
            gtk::HeaderBar::new();
            ..set_show_close_button(true);
            ..set_title("Font Finder");
            ..pack_start(&settings);
            ..pack_start(&show_installed);
            ..pack_start(&font_size);
            ..pack_end(&install);
            ..pack_end(&uninstall);
        };

        Header {
            container,
            font_size,
            install,
            uninstall,
            show_installed,
            dark_preview,
        }
    }
}
