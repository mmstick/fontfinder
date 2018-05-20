extern crate fontfinder;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate webkit2gtk;

mod utils;
mod ui;

use fontfinder::fc_cache::{fc_cache_event_loop, RUN_FC_CACHE};
use fontfinder::{dirs, FontError};
use fontfinder::fonts::{self, Sorting};
use gtk::*;
use ui::App;
use std::process;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use utils::get_buffer;

fn main() {
    glib::set_program_name("Font Finder".into());
    glib::set_application_name("Font Finder");

    // Spawn a thread to wait for fc-cache events
    fc_cache_event_loop();

    // Initialize GTK before proceeding.
    if gtk::init().is_err() {
        eprintln!("failed to initialize GTK Application");
        process::exit(1);
    }

    // Grabs the local font directory, which is "~/.local/share/fonts/"
    let local_font_path = match dirs::font_cache().ok_or(FontError::FontDirectory) {
        Ok(path) => Arc::new(path),
        Err(why) => {
            eprintln!("failed to get font archive: {}", why);
            process::exit(1);
        }
    };

    // Grab the information on Google's archive of free fonts.
    // I'm wrapping it in Arc so it can be shared across multiple closures.
    let fonts_archive = match fonts::obtain(Sorting::Trending) {
        Ok(fonts_archive) => Arc::new(RwLock::new(fonts_archive)),
        Err(why) => {
            eprintln!("failed to get font archive: {}", why);
            process::exit(1);
        }
    };

    // Contains the ID of the currently-selected row, to cut down on lookups.
    let row_id = Arc::new(AtomicUsize::new(0));

    let app = {
        // Collect a list of unique categories from that font list.
        let categories = fonts_archive.read().unwrap().get_categories();

        // Initializes the complete structure of the GTK application.
        // Contains all relevant widgets that we will manipulate.
        Rc::new(App::new(&fonts_archive.read().unwrap(), &categories))
    };

    // The following code will program the widgets in the UI. Each `clone()` will
    // merely increment reference counters, and they exist to allow these
    // widgets to be shared across multiple closures.

    {
        // Updates the UI when a row is selected.
        let app = app.clone();
        let row_id = row_id.clone();
        let fonts_archive = fonts_archive.clone();
        app.main.fonts.container.clone().connect_row_selected(move |_, row| {
            if let Some(row) = row.as_ref() {
                // Get the ID of the currently-selected row.
                let id = row.get_index() as usize;
                // Store that ID in an atomic value, for future re-use by other closures.
                row_id.store(id, Ordering::SeqCst);
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
                        let archive = fonts_archive.read().unwrap();
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

    let path = local_font_path;

    // A macro that's shared among each action that triggers an update of the
    // preview.
    macro_rules! update_preview {
        ($($value:ident => $method:tt),+) => {{
            $({
                let app = app.clone();
                let row_id = row_id.clone();
                #[allow(unused)]
                $value.$method(move |$value| {
                    get_buffer(&app.main.sample_buffer).map(|sample| {
                        let font = &app.main.fonts.get_rows()[row_id.load(Ordering::SeqCst)];
                        app.update_preview(font);
                    });
                });
            })+
        }};
    }

    {
        let size = app.header.font_size.clone();
        let dark_preview = app.header.dark_preview.clone();
        let sample = app.main.sample_buffer.clone();

        update_preview!{
            // Triggers when the font size spin button's value is changed.
            size => connect_property_value_notify,
            // Triggers when the dark preview check button is toggled.
            dark_preview => connect_toggled,
            // Triggers when the sample text is changed.
            sample => connect_changed
        };
    }

    // A macro that's shared among each action that triggers font filtration.
    macro_rules! filter_fonts {
        ($($value:ident => $method:tt),+) => {{
            $({
                let path = path.clone();
                let app = app.clone();
                let fonts_archive = fonts_archive.clone();
                #[allow(unused)]
                $value.$method(move |$value| {
                    app.filter_categories(&path, &fonts_archive.read().unwrap());
                });
            })+
        }};
    }

    {
        let category = app.main.categories.clone();
        let search = app.main.search.clone();
        let show_installed = app.header.show_installed.clone();

        filter_fonts!{
            // Triggers when the category combo box is changed.
            category => connect_changed,
            // Triggers when the search entry is changed.
            search => connect_search_changed,
            // Triggers when the show installed button is toggled.
            show_installed => connect_toggled
        }
    }

    {
        let app = app.clone();
        let fonts_archive = fonts_archive.clone();
        app.main.sort_by.clone().connect_changed(move |sort_by| {
            let sorting = match sort_by.get_active() {
                0 => Sorting::Trending,
                1 => Sorting::Popular,
                2 => Sorting::DateAdded,
                3 => Sorting::Alphabetical,
                _ => unreachable!("unknown sorting"),
            };

            let mut fonts_archive = fonts_archive.write().unwrap();
            *fonts_archive = match fonts::obtain(sorting) {
                Ok(fonts_archive) => fonts_archive,
                Err(why) => {
                    eprintln!("failed to get font archive: {}", why);
                    return;
                }
            };

            app.main.fonts.update(&fonts_archive.items);
            app.filter_categories(&path, &fonts_archive);
        });
    }

    {
        // Programs the install button
        let app = app.clone();
        let row_id = row_id.clone();
        let fonts_archive = fonts_archive.clone();
        app.header.install.clone().connect_clicked(move |install| {
            let font = &app.main.fonts.get_rows()[row_id.load(Ordering::SeqCst)];
            let mut string = Vec::new();
            match fonts_archive
                .read()
                .unwrap()
                .download(&mut string, &font.family)
            {
                Ok(_) => {
                    install.set_visible(false);
                    app.header.uninstall.set_visible(true);
                    font.container.set_visible(app.header.show_installed.get_active());
                    RUN_FC_CACHE.store(true, Ordering::Relaxed);
                    eprintln!("{} installed", &font.family);
                }
                Err(why) => {
                    eprintln!("unable to install font: {}", why);
                }
            }
        });
    }

    {
        // Programs the uninstall button
        let app = app.clone();
        let fonts_archive = fonts_archive.clone();
        app.header.uninstall.clone().connect_clicked(move |uninstall| {
            let font = &app.main.fonts.get_rows()[row_id.load(Ordering::SeqCst)];
            let mut string = Vec::new();
            match fonts_archive
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

    // Shows the application window and all of the widgets owned by that window.
    app.window.show_all();
    // Additionally hides some widgets that should be hidden by default.
    app.header.install.set_visible(false);
    app.header.uninstall.set_visible(false);

    // Begins the main event loop of GTK, which will display the GUI and handle all
    // the actions that were mapped to each of the widgets in the UI.
    gtk::main();
}
