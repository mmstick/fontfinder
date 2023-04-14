use crate::utils::{block_on, set_class, set_margin};
use crate::{fl, Event};
use async_channel::Sender;
use gtk::prelude::*;
use std::ops::Deref;

#[derive(Clone)]
pub struct Header {
    container: gtk::HeaderBar,
    pub font_size: gtk::SpinButton,
    pub install: gtk::Button,
    pub uninstall: gtk::Button,
    pub show_installed: gtk::CheckButton,
    pub dark_preview: gtk::CheckButton,
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
    pub fn new(tx: Sender<Event>) -> Header {
        let install = cascade! {
            let install = gtk::Button::with_label(&fl!("button-install"));
            ..connect_clicked(closure!(clone tx, |_| {
                let _ = block_on(tx.send(Event::Install));
            }));
            set_class(&install, "suggested-action");
        };

        let uninstall = cascade! {
            let uninstall = gtk::Button::with_label(&fl!("button-uninstall"));
            ..connect_clicked(closure!(clone tx, |_| {
                let _ = block_on(tx.send(Event::Uninstall));
            }));
            set_class(&uninstall, "destructive-action");
        };

        // Add a font size spin button.
        let font_size = cascade! {
            gtk::SpinButton::new(
                Some(&gtk::Adjustment::new(1.5, 1.0, 50.0, 0.25, 0.0, 0.0)),
                0.1,
                2,
            );
            ..connect_value_notify(closure!(clone tx, |_| {
                let _ = block_on(tx.send(Event::UpdatePreview));
            }));
        };

        let dark_preview = cascade! {
            gtk::CheckButton::with_label(&fl!("button-dark-preview"));
            ..connect_toggled(closure!(clone tx, |_| {
                let _ = block_on(tx.send(Event::UpdatePreview));
            }));
        };

        let show_installed = cascade! {
            gtk::CheckButton::with_label(&fl!("button-show-installed"));
            ..set_active(true);
            ..connect_toggled(move |_| {
                let _ = block_on(tx.send(Event::Filter));
            });
        };

        // The settings menu, contained within a vertical box.
        let menu_box = cascade! {
            let menu_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
            ..pack_start(&gtk::Label::new(Some(&fl!("menu-show"))), false, false, 0);
            ..pack_start(&show_installed, false, false, 0);
            ..pack_start(&gtk::Separator::new(gtk::Orientation::Horizontal), false, false, 0);
            ..pack_start(&gtk::Label::new(Some(&fl!("menu-preview"))), false, false, 0);
            ..pack_start(&dark_preview, false, false, 0);
            set_margin(&menu_box, 5, 5, 5, 5);
        };

        // Create the popover menu for the settings menu button.
        let popover = cascade! {
            gtk::PopoverMenu::new();
            ..add(&menu_box);
            menu_box.show_all();
        };

        // Attach the popover to the settings menu button.
        let settings = cascade! {
            gtk::MenuButton::new();
            ..set_image(Some(&gtk::Image::from_icon_name(Some("preferences-system"), gtk::IconSize::from(0))));
            ..set_popover(Some(&popover));
            ..set_use_popover(true);
        };

        // Attach everything to the headerbar
        let container = cascade! {
            gtk::HeaderBar::new();
            ..set_show_close_button(true);
            ..set_title("Font Finder".into());
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
