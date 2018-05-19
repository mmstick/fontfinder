extern crate fontfinder;
extern crate gio;
extern crate gtk;
extern crate webkit2gtk;

mod fc_cache;
mod gtk_ui;

use self::fc_cache::{fc_cache_event_loop, RUN_FC_CACHE};
use fontfinder::{
    dirs, fonts::{self, FontsList}, html, FontError,
};
use gtk::*;
use gtk_ui::{App, FontRow};
use std::{
    path::Path, process, str, sync::{
        atomic::{AtomicUsize, Ordering}, Arc,
    },
};
use webkit2gtk::*;

fn main() {
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
    let fonts_archive = match fonts::obtain() {
        Ok(fonts_archive) => Arc::new(fonts_archive),
        Err(why) => {
            eprintln!("failed to get font archive: {}", why);
            process::exit(1);
        }
    };

    // Collect a list of unique categories from that font list.
    let categories = fonts_archive.get_categories();
    // Contains the ID of the currently-selected row, to cut down on lookups.
    let row_id = Arc::new(AtomicUsize::new(0));

    // Initializes the complete structure of the GTK application.
    // Contains all relevant widgets that we will manipulate.
    let app = App::new(&fonts_archive, &categories);

    // The following code will program the widgets in the UI. Each `clone()` will merely
    // increment reference counters, and they exist to allow these widgets to be shared across
    // multiple closures.

    {
        // Updates the UI when a row is selected.
        let sample = app.main.sample_buffer.clone();
        let preview = app.main.view.clone();
        let rows = app.main.fonts.clone();
        let list = app.main.fonts_box.clone();
        let uninstall = app.header.uninstall.clone();
        let install = app.header.install.clone();
        let title = app.header.container.clone();
        let size = app.header.font_size.clone();
        let row_id = row_id.clone();
        let fonts_archive = fonts_archive.clone();
        let dark_preview = app.header.dark_preview.clone();
        list.connect_row_selected(move |_, row| {
            if let Some(row) = row.as_ref() {
                // Get the ID of the currently-selected row.
                let id = row.get_index() as usize;
                // Store that ID in an atomic value, for future re-use by other closures.
                row_id.store(id, Ordering::SeqCst);
                // Obtain the data relevant to the selected row, by it's ID.
                let font = &(*rows.borrow())[id];
                // Set the header bar's title the name of the font.
                title.set_title(font.family.as_str());

                // If there is some sample text, update the font preview.
                if let Some(sample_text) = get_buffer(&sample) {
                    html::generate(
                        &font.family,
                        &font.variants,
                        size.get_value(),
                        &sample_text[..],
                        dark_preview.get_active(),
                        |html| preview.load_html(html, None),
                    );
                }

                // Then set the visibility of the Install & Uninstall buttons accordingly.
                match dirs::font_cache().ok_or(FontError::FontDirectory) {
                    Ok(path) => {
                        // Obtain the font from the font archive, so that we may get the files.
                        let font = fonts_archive.get_family(&font.family).unwrap();
                        // This returns true if all variants of the font exists.
                        let font_exists = font.files.iter().all(|(variant, uri)| {
                            dirs::font_exists(&path, &font.family, &variant, &uri)
                        });

                        install.set_visible(!font_exists);
                        uninstall.set_visible(font_exists);
                    }
                    Err(why) => {
                        // Write the error to stderr & the console.
                        eprintln!("unable to get font cache: {}", why);

                        install.set_visible(false);
                        uninstall.set_visible(false);
                    }
                }
            }
        });
    }

    let sample = app.main.sample_buffer;
    let preview = app.main.view;
    let rows = app.main.fonts;
    let size = app.header.font_size;
    let dark_preview = app.header.dark_preview;
    let category = app.main.categories;
    let search = app.main.search;
    let path = local_font_path;
    let show_installed = app.header.show_installed;

    // A macro that's shared among each action that triggers an update of the preview.
    macro_rules! update_preview {
        ($value:tt, $method:tt) => {{
            let sample = sample.clone();
            let preview = preview.clone();
            let rows = rows.clone();
            let size = size.clone();
            let row_id = row_id.clone();
            let dark_preview = dark_preview.clone();
            #[allow(unused)]
            $value.$method(move |$value| {
                get_buffer(&sample).map(|sample| {
                    let font = &(*rows.borrow())[row_id.load(Ordering::SeqCst)];
                    html::generate(
                        &font.family,
                        &font.variants[..],
                        size.get_value(),
                        &sample,
                        dark_preview.get_active(),
                        |html| preview.load_html(html, None),
                    )
                });
            });
        }};
    }

    // Triggers when the font size spin button's value is changed.
    update_preview!(size, connect_property_value_notify);
    // Triggers when the dark preview check button is toggled.
    update_preview!(dark_preview, connect_toggled);
    // Triggers when the sample text is changed.
    update_preview!(sample, connect_changed);

    // A macro that's shared among each action that triggers font filtration.
    macro_rules! filter_fonts {
        ($value:tt, $method:tt) => {{
            let category = category.clone();
            let rows = rows.clone();
            let search = search.clone();
            let path = path.clone();
            let fonts_archive = fonts_archive.clone();
            let show_installed = show_installed.clone();
            #[allow(unused)]
            $value.$method(move |$value| {
                if let Some(category) = category.get_active_text() {
                    filter_category(&category, get_search(&search), &rows.borrow(), |family| {
                        show_installed.get_active() || !is_installed(&fonts_archive, family, &path)
                    });
                }
            });
        }};
    }

    // Triggers when the category combo box is changed.
    filter_fonts!(category, connect_changed);
    // Triggers when the search entry is changed.
    filter_fonts!(search, connect_search_changed);
    // Triggers when the show installed button is toggled.
    filter_fonts!(show_installed, connect_toggled);

    {
        // Programs the install button
        let install = app.header.install.clone();
        let uninstall = app.header.uninstall.clone();
        let row_id = row_id.clone();
        let rows = rows.clone();
        let fonts_archive = fonts_archive.clone();
        let installed = show_installed.clone();
        install.connect_clicked(move |install| {
            let font = &(*rows.borrow())[row_id.load(Ordering::SeqCst)];
            let mut string = Vec::new();
            match fonts_archive.download(&mut string, &font.family) {
                Ok(_) => {
                    install.set_visible(false);
                    uninstall.set_visible(true);
                    font.container.set_visible(installed.get_active());
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
        let install = app.header.install.clone();
        let uninstall = app.header.uninstall.clone();
        let row_id = row_id.clone();
        let rows = rows.clone();
        let fonts_archive = fonts_archive.clone();
        uninstall.connect_clicked(move |uninstall| {
            let font = &(*rows.borrow())[row_id.load(Ordering::SeqCst)];
            let mut string = Vec::new();
            match fonts_archive.remove(&mut string, &font.family) {
                Ok(_) => {
                    uninstall.set_visible(false);
                    install.set_visible(true);
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

    // Begins the main event loop of GTK, which will display the GUI and handle all the
    // actions that were mapped to each of the widgets in the UI.
    gtk::main();
}

/// Filters visibility of associated font ListBoxRow's, according to a given category filter,
/// The contents of the search bar, and a closure that determines whether the font is installed
/// or not.
fn filter_category<F>(category: &str, search: Option<String>, fonts: &[FontRow], installed: F)
where
    F: Fn(&str) -> bool,
{
    fonts.iter().for_each(|font| {
        let visible = (category == "All" || &font.category == category)
            && search.as_ref().map_or(true, |s| font.contains(s.as_str()));

        font.set_visibility(visible && installed(&font.family));
    })
}

/// Obtains the entire inner string of a given text buffer.
fn get_buffer(buffer: &TextBuffer) -> Option<String> {
    let start = buffer.get_start_iter();
    let end = buffer.get_end_iter();
    buffer.get_text(&start, &end, true)
}

/// Obtains the value of the search entry from the UI
fn get_search(search: &SearchEntry) -> Option<String> {
    match search.get_text().take() {
        Some(ref text) if text.is_empty() => None,
        Some(text) => Some(text),
        None => None,
    }
}

/// Evaluates whether each variant of a given font family is locally installed.
fn is_installed(archive: &FontsList, family: &str, path: &Path) -> bool {
    let font = archive.get_family(&family).unwrap();
    font.files
        .iter()
        .all(|(variant, uri)| dirs::font_exists(path, family, variant.as_str(), uri.as_str()))
}
