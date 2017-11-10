mod header;
mod main;

pub use self::header::Header;
pub use self::main::{FontRow, Main};
use fontfinder::fonts::FontsList;
use gtk::*;

#[derive(Clone)]
pub struct App {
    pub window: Window,
    pub header: Header,
    pub main:   Main,
}

impl App {
    pub fn new(font_archive: &FontsList, categories: &[String]) -> App {
        Window::set_default_icon_name("typecatcher");
        let window = Window::new(WindowType::Toplevel);
        let header = Header::new();
        let main = Main::new(&font_archive.items, categories);

        window.set_titlebar(&header.container);
        window.add(&main.container);
        window.set_title("Font Finder");
        window.set_default_size(600, 400);
        window.set_wmclass("font-finder", "Font Finder");

        window.connect_delete_event(move |_, _| {
            main_quit();
            Inhibit(false)
        });

        App {
            window,
            header,
            main,
        }
    }
}

pub fn set_margin<W: WidgetExt>(widget: &W, t: i32, r: i32, b: i32, l: i32) {
    widget.set_margin_top(t);
    widget.set_margin_right(r);
    widget.set_margin_bottom(b);
    widget.set_margin_left(l);
}
