mod main;
mod widgets;

pub use self::main::Main;
pub use self::widgets::{FontList, FontRow, Header};

use async_channel::Sender;
use fontfinder::{
    dirs,
    fonts::{self, FontsList, Sorting},
    html,
};

use gtk::prelude::*;
use gtk::traits::WidgetExt;
use webkit2gtk::WebViewExt;

use crate::utils::{get_buffer, get_search, spawn_local};
use std::path::{Path, PathBuf};

pub enum Event {
    Filter,
    Install,
    Select(usize),
    Sort(Sorting),
    UpdatePreview,
    Uninstall,
    TriggerFontCache,
}

pub struct State {
    pub fonts: FontsList,
    pub path: PathBuf,
    pub row_id: usize,
    pub tx: async_channel::Sender<Event>,
}

pub struct App {
    window: gtk::Window,
    pub header: Header,
    pub main: Main,
    pub state: State,
}

impl App {
    pub fn new(tx: Sender<Event>) -> App {
        // Grabs the local font directory, which is "~/.local/share/fonts/"
        let path = match dirs::font_cache() {
            Ok(path) => path,
            Err(why) => {
                eprintln!("{why}");
                std::process::exit(1);
            }
        };

        // Grab the information on Google's archive of free fonts.
        let fonts = match fonts::obtain(Sorting::Trending) {
            Ok(fonts) => fonts,
            Err(why) => {
                eprintln!("failed to get font archive: {why:?}");
                std::process::exit(1);
            }
        };

        let state = State {
            fonts,
            row_id: 0,
            path,
            tx: tx.clone(),
        };

        // Collect a list of unique categories from that font list.
        let categories = &state.fonts.get_categories();

        let header = Header::new(tx.clone());
        let main = Main::new(&state.fonts.items, categories, tx.clone());

        let window = cascade! {
            gtk::Window::new(gtk::WindowType::Toplevel);
            ..set_titlebar(Some(header.as_ref()));
            ..set_title("Font Finder");
            ..set_default_size(800, 600);
            ..add(&main.container);
            ..connect_delete_event(move |_, _| {
                gtk::main_quit();
                Inhibit(false)
            });
        };

        gtk::Window::set_default_icon_name("typecatcher");

        let app = App {
            window,
            header,
            main,
            state,
        };

        app.show();

        app
    }

    pub fn install(&self) {
        let font = &self.main.fonts.get_rows()[self.state.row_id];
        let mut string = Vec::new();
        match self.state.fonts.download(&mut string, &font.family) {
            Ok(_) => {
                self.header.install.set_visible(false);
                self.header.uninstall.set_visible(true);
                font.set_visible(self.header.show_installed.is_active());

                let tx = self.state.tx.clone();
                let _ = spawn_local(async move {
                    let _ = tx.send(Event::TriggerFontCache).await;
                });

                eprintln!("{} installed", &font.family);
            }
            Err(why) => {
                eprintln!("unable to install font: {}", why);
            }
        }
    }

    pub fn filter_categories(&self) {
        let path = &self.state.path;
        let fonts = &self.state.fonts;

        if let Some(category) = self.main.categories.active_text() {
            filter_category(
                &category,
                get_search(&self.main.search).as_ref().map(|x| x.as_str()),
                &self.main.fonts.get_rows(),
                |family| {
                    self.header.show_installed.is_active() || !is_installed(fonts, family, path)
                },
            );
        }
    }

    pub fn select_row(&mut self, id: usize) {
        // Store that ID in an atomic value, for future re-use by other closures.
        self.state.row_id = id;

        // Obtain the data relevant to the selected row, by it's ID.
        let font = &self.main.fonts.get_rows()[id];

        // Set the header bar's title the name of the font.
        self.header.set_title(Some(font.family.as_str()));

        // If there is some sample text, update the font preview.
        self.update_preview();

        // Then set the visibility of the Install & Uninstall buttons accordingly.
        match dirs::font_cache() {
            Ok(path) => {
                // Obtain the font from the font archive, so that we may get the files.
                let font = self.state.fonts.get_family(&font.family).unwrap();

                // This returns true if all variants of the font exists.
                let font_exists = font
                    .files
                    .iter()
                    .all(|(variant, uri)| dirs::font_exists(&path, &font.family, &variant, &uri));

                self.header.install.set_visible(!font_exists);
                self.header.uninstall.set_visible(font_exists);
            }
            Err(why) => {
                // Write the error to stderr & the console.
                eprintln!("unable to get font cache: {}", why);

                self.header.install.set_visible(false);
                self.header.uninstall.set_visible(false);
            }
        }
    }

    pub fn show(&self) {
        // Shows the application window and all of the widgets owned by that window.
        self.window.show_all();

        // Additionally hides some widgets that should be hidden by default.
        self.header.install.set_visible(false);
        self.header.uninstall.set_visible(false);
    }

    pub fn sort(&mut self, sorting: Sorting) {
        let fonts = &mut self.state.fonts;
        *fonts = match fontfinder::fonts::obtain(sorting) {
            Ok(fonts) => fonts,
            Err(why) => {
                eprintln!("failed to get font archive: {why}");
                return;
            }
        };

        self.main.fonts.update(&fonts.items);
        self.filter_categories();
    }

    pub fn uninstall(&self) {
        let font = &self.main.fonts.get_rows()[self.state.row_id];
        let mut string = Vec::new();
        match self.state.fonts.remove(&mut string, &font.family) {
            Ok(_) => {
                self.header.uninstall.set_visible(false);
                self.header.install.set_visible(true);

                let tx = self.state.tx.clone();
                let _ = spawn_local(async move {
                    let _ = tx.send(Event::TriggerFontCache).await;
                });

                eprintln!("{} uninstalled", &font.family);
            }
            Err(why) => {
                eprintln!("unable to remove font: {why}");
            }
        }
    }

    pub fn update_preview(&self) {
        let font = &self.main.fonts.get_rows()[self.state.row_id];

        if let Some(sample_text) = get_buffer(&self.main.sample_buffer) {
            html::generate(
                &font.family,
                &font.variants,
                self.header.font_size.value(),
                &sample_text[..],
                self.header.dark_preview.is_active(),
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
    });
}

/// Evaluates whether each variant of a given font family is locally installed.
fn is_installed(archive: &FontsList, family: &str, path: &Path) -> bool {
    let font = archive.get_family(&family).unwrap();
    font.files
        .iter()
        .all(|(variant, uri)| dirs::font_exists(path, family, variant.as_str(), uri.as_str()))
}
