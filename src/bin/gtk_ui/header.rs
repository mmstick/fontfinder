use gtk::*;

#[derive(Clone)]
pub struct Header {
    pub container:  HeaderBar,
    pub font_size:  SpinButton,
    pub install:    Button,
    pub uninstall:  Button
}

impl Header {
    pub fn new() -> Header {
        let container = HeaderBar::new();
        container.set_show_close_button(true);
        container.set_title("Font Finder");

        let font_size = SpinButton::new(&Adjustment::new(2.0, 1.0, 50.0, 0.25, 0.0, 0.0), 0.1, 2);

        let install = Button::new_with_label("Install");
        install.set_visible(false);

        let uninstall = Button::new_with_label("Uninstall");
        uninstall.set_visible(false);

        container.pack_start(&font_size);
        container.pack_end(&install);
        container.pack_end(&uninstall);

        Header {
            container,
            font_size,
            install,
            uninstall
        }
    }
}
