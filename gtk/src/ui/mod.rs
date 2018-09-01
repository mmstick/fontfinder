mod connected;
mod main;
mod widgets;

pub use self::connected::{Connect, Connected};
pub use self::widgets::{Header, FontList, FontRow};
pub use self::main::Main;
use fontfinder::dirs;
use fontfinder::html;
use fontfinder::fonts::FontsList;
use gtk::prelude::*;
use gtk;
use gtk::WidgetExt;
use webkit2gtk::*;

use utils::{get_buffer, get_search};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::sync::atomic::AtomicUsize;

pub struct State {
    pub fonts_archive: RwLock<FontsList>,
    pub row_id: AtomicUsize,
    pub path: PathBuf,
}

#[derive(Clone)]
pub struct App {
    pub window: gtk::Window,
    pub header: Header,
    pub main: Main,
}

impl App {
    pub fn new(font_archive: &FontsList, categories: &[String]) -> App {
        let header = Header::new();
        let main = Main::new(&font_archive.items, categories);

        let window = cascade! {
            gtk::Window::new(gtk::WindowType::Toplevel);
            ..set_titlebar(header.as_ref());
            ..set_title("Font Finder");
            | gtk::Window::set_default_icon_name("typecatcher");
            ..set_default_size(800, 600);
            ..add(&main.container);
            ..connect_delete_event(move |_, _| {
                gtk::main_quit();
                Inhibit(false)
            });
        };

        App {
            window,
            header,
            main: main,
        }
    }

    pub fn filter_categories(&self, path: &Path, fonts_archive: &FontsList) {
        if let Some(category) = self.main.categories.get_active_text() {
            filter_category(
                &category,
                get_search(&self.main.search).as_ref().map(|x| x.as_str()),
                &self.main.fonts.get_rows(),
                |family| {
                    self.header.show_installed.get_active()
                        || !is_installed(fonts_archive, family, path)
                },
            );
        }
    }

    pub fn update_preview(&self, font: &FontRow) {
        if let Some(sample_text) = get_buffer(&self.main.sample_buffer) {
            html::generate(
                &font.family,
                &font.variants,
                self.header.font_size.get_value(),
                &sample_text[..],
                self.header.dark_preview.get_active(),
                |html| self.main.view.load_html(html, None),
            );
        }
    }
}

/// Filters visibility of associated font ListBoxRow's, according to a given category filter,
/// The contents of the search bar, and a closure that determines whether the font is installed
/// or not.
fn filter_category<F>(category: &str, search: Option<&str>, fonts: &[FontRow], installed: F)
where
    F: Fn(&str) -> bool,
{
    fonts.iter().for_each(|font| {
        let visible = (category == "All" || font.category == category)
            && search.as_ref().map_or(true, |s| font.contains(s));

        font.set_visible(visible && installed(&font.family));
    })
}

/// Evaluates whether each variant of a given font family is locally installed.
fn is_installed(archive: &FontsList, family: &str, path: &Path) -> bool {
    let font = archive.get_family(&family).unwrap();
    font.files
        .iter()
        .all(|(variant, uri)| dirs::font_exists(path, family, variant.as_str(), uri.as_str()))
}
