mod fontlist;
mod header;
mod main;

pub use self::header::Header;
pub use self::main::Main;
pub use self::fontlist::{FontList, FontRow};
use fontfinder::fonts::FontsList;
use gtk::*;

#[derive(Clone)]
pub struct App {
    pub window: Window,
    pub header: Header,
    pub main: Main,
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
