#![allow(unknown_lints)]

#[macro_use]
extern crate cascade;
#[macro_use]
extern crate closure;

mod app;
mod localize;
mod utils;

use self::app::{App, Event};
use i18n_embed::DesktopLanguageRequester;
use std::process;

fn main() {
    let requested_languages = DesktopLanguageRequester::requested_languages();

    let localizers = vec![
        ("fontfinder-gtk", crate::localize::localizer()),
        ("fontfinder", fontfinder::localize::localizer()),
    ];

    for (lib, localizer) in localizers {
        if let Err(error) = localizer.select(&requested_languages) {
            eprintln!("Error while loading languages for {lib} {error}");
        }
    }

    glib::set_program_name("Font Finder".into());
    glib::set_application_name("Font Finder");

    // Initialize GTK before proceeding.
    if gtk::init().is_err() {
        eprintln!("failed to initialize GTK Application");
        process::exit(1);
    }

    let (tx, rx) = async_channel::unbounded();

    // Initializes the complete structure of the GTK application.
    // Contains all relevant widgets that we will manipulate.
    let mut app = App::new(tx);

    // Async event loop for handling all application events;
    let event_handler = async move {
        while let Ok(event) = rx.recv().await {
            match event {
                Event::Filter => app.filter_categories(),
                Event::Install => app.install(),
                Event::Select(row) => app.select_row(row),
                Event::Sort(sorting) => app.sort(sorting),
                Event::UpdatePreview => app.update_preview(),
                Event::Uninstall => app.uninstall(),
                Event::TriggerFontCache => {
                    utils::spawn_local(fontfinder::run_fc_cache());
                }
            }
        }
    };

    utils::spawn_local(event_handler);

    // Begins the main event loop of GTK, which will display the GUI and handle all
    // the actions that were mapped to each of the widgets in the UI.
    gtk::main();
}
