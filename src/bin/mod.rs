extern crate fontfinder;
extern crate gio;
extern crate gtk;
extern crate webkit2gtk;

mod gtk_ui;

use fontfinder::{dirs, fonts, html, FontError};
use fontfinder::fonts::FontsList;
use gtk::*;
use gtk_ui::{App, FontRow};
use std::path::Path;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use webkit2gtk::*;

fn main() {
    // Initialize GTK before proceeding.
    if gtk::init().is_err() {
        eprintln!("failed to initialize GTK Application");
        process::exit(1);
    }

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
    let current_row_id = Arc::new(AtomicUsize::new(0));

    // Initializes the complete structure of the GTK application.
    // Contains all relevant widgets that we will manipulate.
    let app = App::new(&fonts_archive, &categories);

    // The following code will program the widgets in the UI. Each `clone()` will merely
    // increment reference counters, and they exist to allow these widgets to be shared across
    // multiple closures.

    {
        // Updates the UI when a row is selected.
        let sample = app.main.sample_text.clone();
        let preview = app.main.view.clone();
        let rows = app.main.fonts.clone();
        let list = app.main.fonts_box.clone();
        let uninstall = app.header.uninstall.clone();
        let install = app.header.install.clone();
        let title = app.header.container.clone();
        let size = app.header.font_size.clone();
        let row_id = current_row_id.clone();
        let fonts_archive = fonts_archive.clone();
        list.connect_row_selected(move |_, row| {
            if let Some(row) = row.clone() {
                let id = row.get_index() as usize;
                row_id.store(id, Ordering::SeqCst);
                let font = &(*rows.borrow())[id];
                title.set_title(font.family.as_str());
                if let Some(sample_text) = get_text(&sample) {
                    html::generate(&font.family, size.get_value(), &sample_text, |html| {
                        preview.load_html(html, None)
                    });
                }

                match dirs::font_cache().ok_or(FontError::FontDirectory) {
                    Ok(path) => {
                        let font = fonts_archive.get_family(&font.family).unwrap();
                        let font_exists = font.files.iter().all(
                            |(variant, uri)| dirs::font_exists(&path, &font.family, &variant, &uri),
                        );
                        install.set_visible(!font_exists);
                        uninstall.set_visible(font_exists);
                    }
                    Err(why) => {
                        eprintln!("fontfinder: unable to get font cache: {}", why);
                        install.set_visible(false);
                        uninstall.set_visible(false);
                    }
                }
            }
        });
    }

    {
        // Updates the preview when the value of the font size spinner is changed.
        let sample = app.main.sample_buffer.clone();
        let preview = app.main.view.clone();
        let rows = app.main.fonts.clone();
        let size = app.header.font_size.clone();
        let row_id = current_row_id.clone();
        size.connect_property_value_notify(move |size| {
            let font = &(*rows.borrow())[row_id.load(Ordering::SeqCst)];
            if let Some(sample_text) = get_buffer(&sample) {
                html::generate(
                    &font.family,
                    size.get_value(),
                    &sample_text,
                    |html| preview.load_html(html, None),
                );
            }
        });
    }

    {
        // Updates the preview when the sample text is updated.
        let sample = app.main.sample_buffer.clone();
        let preview = app.main.view.clone();
        let rows = app.main.fonts.clone();
        let size = app.header.font_size.clone();
        let row_id = current_row_id.clone();
        sample.connect_changed(move |sample| {
            let font = &(*rows.borrow())[row_id.load(Ordering::SeqCst)];
            if let Some(sample_text) = get_buffer(&sample) {
                html::generate(
                    &font.family,
                    size.get_value(),
                    &sample_text,
                    |html| preview.load_html(html, None),
                );
            }
        });
    }

    {
        // Filters all fonts that don't match a selected category.
        let category = app.main.categories.clone();
        let rows = app.main.fonts.clone();
        let search = app.main.search.clone();
        let path = local_font_path.clone();
        let archive = fonts_archive.clone();
        let installed = app.header.show_installed.clone();
        category.connect_changed(move |category| {
            if let Some(category) = category.get_active_text() {
                filter_category(&category, get_search(&search), &rows.borrow(), |family| {
                    !installed.get_active() || !is_installed(&archive, family, &path)
                });
            }
        });
    }

    {
        // Filters fonts based on the search + category.
        let category = app.main.categories.clone();
        let rows = app.main.fonts.clone();
        let search = app.main.search.clone();
        let path = local_font_path.clone();
        let archive = fonts_archive.clone();
        let installed = app.header.show_installed.clone();
        search.connect_search_changed(move |search| {
            if let Some(category) = category.get_active_text() {
                filter_category(&category, get_search(&search), &rows.borrow(), |family| {
                    installed.get_active() || !is_installed(&archive, family, &path)
                });
            }
        });
    }

    {
        // Filters fonts when the show_installed checkbox is toggled.
        let category = app.main.categories.clone();
        let rows = app.main.fonts.clone();
        let search = app.main.search.clone();
        let path = local_font_path.clone();
        let archive = fonts_archive.clone();
        let installed = app.header.show_installed.clone();
        installed.connect_toggled(move |installed| {
            if let Some(category) = category.get_active_text() {
                filter_category(&category, get_search(&search), &rows.borrow(), |family| {
                    installed.get_active() || !is_installed(&archive, family, &path)
                });
            }
        });
    }

    {
        // Programs the install button
        let install = app.header.install.clone();
        let uninstall = app.header.uninstall.clone();
        let row_id = current_row_id.clone();
        let rows = app.main.fonts.clone();
        let fonts_archive = fonts_archive.clone();
        let installed = app.header.show_installed.clone();
        install.connect_clicked(move |install| {
            let font = &(*rows.borrow())[row_id.load(Ordering::SeqCst)];
            match fonts_archive.download(&font.family) {
                Ok(_) => {
                    install.set_visible(false);
                    uninstall.set_visible(true);
                    font.container.set_visible(installed.get_active());
                }
                Err(why) => eprintln!("fontfinder: unable to install font: {}", why),
            }
        });
    }

    {
        // Programs the uninstall button
        let install = app.header.install.clone();
        let uninstall = app.header.uninstall.clone();
        let row_id = current_row_id.clone();
        let rows = app.main.fonts.clone();
        let fonts_archive = fonts_archive.clone();
        uninstall.connect_clicked(move |uninstall| {
            let font = &(*rows.borrow())[row_id.load(Ordering::SeqCst)];
            match fonts_archive.remove(&font.family) {
                Ok(_) => {
                    uninstall.set_visible(false);
                    install.set_visible(true);
                }
                Err(why) => eprintln!("fontfinder: unable to remove font: {}", why),
            }
        });
    }

    app.window.show_all();
    app.header.install.set_visible(false);
    app.header.uninstall.set_visible(false);

    gtk::main();
}

/// Filters visibility of associated font ListBoxRow's, according to a given category filter.
fn filter_category<F>(category: &str, search: Option<String>, fonts: &[FontRow], installed: F)
    where F: Fn(&str) -> bool
{
    fonts.iter().for_each(|font| {
        let visible = (category == "All" || &font.category == category)
            && search.as_ref().map_or(true, |s| font.contains(s.as_str()));

        font.set_visibility(visible && installed(&font.family));
    })
}

/// Attempt to get the text from thhe given `TextView`'s `TextBuffer`.
fn get_text(view: &TextView) -> Option<String> { view.get_buffer().and_then(|x| get_buffer(&x)) }

fn get_buffer(buffer: &TextBuffer) -> Option<String> {
    let start = buffer.get_start_iter();
    let end = buffer.get_end_iter();
    buffer.get_text(&start, &end, true)
}

fn get_search(search: &SearchEntry) -> Option<String> {
    match search.get_text().take() {
        Some(ref text) if text.is_empty() => None,
        Some(text) => Some(text),
        None => None,
    }
}

fn is_installed(archive: &FontsList, family: &str, path: &Path) -> bool {
    let font = archive.get_family(&family).unwrap();
    font.files
        .iter()
        .all(|(variant, uri)| dirs::font_exists(path, family, variant.as_str(), uri.as_str()))
}
