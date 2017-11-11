use super::set_margin;
use gtk::*;

#[derive(Clone)]
pub struct Header {
    pub container:      HeaderBar,
    pub font_size:      SpinButton,
    pub install:        Button,
    pub uninstall:      Button,
    pub show_installed: CheckButton,
}

macro_rules! button { ($label:expr) => { Button::new_with_label($label) } }

impl Header {
    pub fn new() -> Header {
        // Headers need header bars, right?
        let container = HeaderBar::new();
        container.set_show_close_button(true);
        container.set_title("Font Finder");

        // Buttons for installing and uninstalling fonts.
        let (install, uninstall) = (button!("Install"), button!("Uninstall"));

        // Set styles for those buttons.
        set_class(&install, "suggested-action");
        set_class(&uninstall, "destructive-action");

        // Add a font size spin button.
        let font_size = SpinButton::new(&Adjustment::new(1.5, 1.0, 50.0, 0.25, 0.0, 0.0), 0.1, 2);
        let show_installed = CheckButton::new_with_label("Installed");
        show_installed.set_active(true);

        // The settings menu, contained within a vertical box.
        let menu_box = Box::new(Orientation::Vertical, 5);
        menu_box.pack_start(&Label::new("Show"), true, false, 0);
        menu_box.pack_start(&show_installed, true, false, 0);
        set_margin(&menu_box, 5, 5, 5, 5);

        // Create the popover menu for the settings menu button.
        let popover = PopoverMenu::new();
        popover.add(&menu_box);
        menu_box.show_all();

        // Attach the popover to the settings menu button.
        let settings = MenuButton::new();
        settings.set_image(&Image::new_from_icon_name("preferences-system", 0));
        settings.set_popover(&popover);
        settings.set_use_popover(true);

        // Attach everything to the headerbar
        container.pack_start(&settings);
        container.pack_start(&show_installed);
        container.pack_start(&font_size);
        container.pack_end(&install);
        container.pack_end(&uninstall);

        Header {
            container,
            font_size,
            install,
            uninstall,
            show_installed,
        }
    }
}

fn set_class<W: WidgetExt>(widget: &W, class: &str) {
    widget.get_style_context().map(|c| c.add_class(class));
}
