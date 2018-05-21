use std::rc::Rc;
use std::sync::Arc;
use fontfinder::{dirs, FontError};
use fontfinder::fc_cache::RUN_FC_CACHE;
use fontfinder::fonts::{self, Sorting};
use super::{App, State};
use std::sync::atomic::Ordering;
use gtk::prelude::*;
use gtk;
use utils::get_buffer;

pub struct Connected(Rc<App>);

impl Connected {
    pub fn then_execute(self) {
        // Shows the application window and all of the widgets owned by that window.
        self.0.window.show_all();

        // Additionally hides some widgets that should be hidden by default.
        self.0.header.install.set_visible(false);
        self.0.header.uninstall.set_visible(false);

        // Begins the main event loop of GTK, which will display the GUI and handle all
        // the actions that were mapped to each of the widgets in the UI.
        gtk::main();
    }
}

pub trait Connect {
    fn connect_events(self, state: Arc<State>) -> Connected;

    fn connect_row_selected(&self, state: Arc<State>);
    fn connect_preview_updates(&self, state: Arc<State>);
    fn connect_filter_fonts(&self, state: Arc<State>);
    fn connect_sorting(&self, state: Arc<State>);
    fn connect_install(&self, state: Arc<State>);
    fn connect_uninstall(&self, state: Arc<State>);
}

impl Connect for Rc<App> {
    fn connect_events(self, state: Arc<State>) -> Connected {
        self.connect_row_selected(state.clone());
        self.connect_preview_updates(state.clone());
        self.connect_filter_fonts(state.clone());
        self.connect_sorting(state.clone());
        self.connect_install(state.clone());
        self.connect_uninstall(state.clone());

        Connected(self)
    }

    fn connect_install(&self, state: Arc<State>) {
        let app = self.clone();
        app.header.install.clone().connect_clicked(move |install| {
            let font = &app.main.fonts.get_rows()[state.row_id.load(Ordering::SeqCst)];
            let mut string = Vec::new();
            match state
                .fonts_archive
                .read()
                .unwrap()
                .download(&mut string, &font.family)
            {
                Ok(_) => {
                    install.set_visible(false);
                    app.header.uninstall.set_visible(true);
                    font.container
                        .set_visible(app.header.show_installed.get_active());
                    RUN_FC_CACHE.store(true, Ordering::Relaxed);
                    eprintln!("{} installed", &font.family);
                }
                Err(why) => {
                    eprintln!("unable to install font: {}", why);
                }
            }
        });
    }

    fn connect_uninstall(&self, state: Arc<State>) {
        let app = self.clone();
        app.header
            .uninstall
            .clone()
            .connect_clicked(move |uninstall| {
                let font = &app.main.fonts.get_rows()[state.row_id.load(Ordering::SeqCst)];
                let mut string = Vec::new();
                match state
                    .fonts_archive
                    .read()
                    .unwrap()
                    .remove(&mut string, &font.family)
                {
                    Ok(_) => {
                        uninstall.set_visible(false);
                        app.header.install.set_visible(true);
                        RUN_FC_CACHE.store(true, Ordering::Relaxed);
                        eprintln!("{} uninstalled", &font.family);
                    }
                    Err(why) => {
                        eprintln!("unable to remove font: {}", why);
                    }
                }
            });
    }

    fn connect_sorting(&self, state: Arc<State>) {
        let app = self.clone();
        app.main.sort_by.clone().connect_changed(move |sort_by| {
            let sorting = match sort_by.get_active() {
                0 => Sorting::Trending,
                1 => Sorting::Popular,
                2 => Sorting::DateAdded,
                3 => Sorting::Alphabetical,
                _ => unreachable!("unknown sorting"),
            };

            let mut fonts_archive = state.fonts_archive.write().unwrap();
            *fonts_archive = match fonts::obtain(sorting) {
                Ok(fonts_archive) => fonts_archive,
                Err(why) => {
                    eprintln!("failed to get font archive: {}", why);
                    return;
                }
            };

            app.main.fonts.update(&fonts_archive.items);
            app.filter_categories(&state.path, &fonts_archive);
        });
    }

    fn connect_filter_fonts(&self, state: Arc<State>) {
        // A macro that's shared among each action that triggers font filtration.
        macro_rules! filter_fonts {
            ($($value:ident => $method:tt),+) => {{
                $({
                    let app = self.clone();
                    let state = state.clone();
                    #[allow(unused)]
                    $value.$method(move |$value| {
                        app.filter_categories(&state.path, &state.fonts_archive.read().unwrap());
                    });
                })+
            }};
        }

        {
            let category = self.main.categories.clone();
            let search = self.main.search.clone();
            let show_installed = self.header.show_installed.clone();

            filter_fonts!{
                // Triggers when the category combo box is changed.
                category => connect_changed,
                // Triggers when the search entry is changed.
                search => connect_search_changed,
                // Triggers when the show installed button is toggled.
                show_installed => connect_toggled
            }
        }
    }

    fn connect_preview_updates(&self, state: Arc<State>) {
        // A macro that's shared among each action that triggers an update of the
        // preview.
        macro_rules! update_preview {
            ($($value:ident => $method:tt),+) => {{
                $({
                    let app = self.clone();
                    let state = state.clone();
                    #[allow(unused)]
                    $value.$method(move |$value| {
                        get_buffer(&app.main.sample_buffer).map(|sample| {
                            let font = &app.main.fonts.get_rows()[state.row_id.load(Ordering::SeqCst)];
                            app.update_preview(font);
                        });
                    });
                })+
            }};
        }

        {
            let size = self.header.font_size.clone();
            let dark_preview = self.header.dark_preview.clone();
            let sample = self.main.sample_buffer.clone();

            update_preview!{
                // Triggers when the font size spin button's value is changed.
                size => connect_property_value_notify,
                // Triggers when the dark preview check button is toggled.
                dark_preview => connect_toggled,
                // Triggers when the sample text is changed.
                sample => connect_changed
            };
        }
    }

    fn connect_row_selected(&self, state: Arc<State>) {
        let app = self.clone();
        app.main
            .fonts
            .container
            .clone()
            .connect_row_selected(move |_, row| {
                if let Some(row) = row.as_ref() {
                    // Get the ID of the currently-selected row.
                    let id = row.get_index() as usize;
                    // Store that ID in an atomic value, for future re-use by other closures.
                    state.row_id.store(id, Ordering::SeqCst);
                    // Obtain the data relevant to the selected row, by it's ID.
                    let font = &app.main.fonts.get_rows()[id];
                    // Set the header bar's title the name of the font.
                    app.header.container.set_title(font.family.as_str());

                    // If there is some sample text, update the font preview.
                    app.update_preview(font);

                    // Then set the visibility of the Install & Uninstall buttons accordingly.
                    match dirs::font_cache().ok_or(FontError::FontDirectory) {
                        Ok(path) => {
                            // Obtain the font from the font archive, so that we may get the files.
                            let archive = state.fonts_archive.read().unwrap();
                            let font = archive.get_family(&font.family).unwrap();

                            // This returns true if all variants of the font exists.
                            let font_exists = font.files.iter().all(|(variant, uri)| {
                                dirs::font_exists(&path, &font.family, &variant, &uri)
                            });

                            app.header.install.set_visible(!font_exists);
                            app.header.uninstall.set_visible(font_exists);
                        }
                        Err(why) => {
                            // Write the error to stderr & the console.
                            eprintln!("unable to get font cache: {}", why);

                            app.header.install.set_visible(false);
                            app.header.uninstall.set_visible(false);
                        }
                    }
                }
            });
    }
}
