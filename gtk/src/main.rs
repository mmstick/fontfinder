#![allow(unknown_lints)]
#![allow(option_map_unit_fn)]

#[macro_use]
extern crate cascade;

mod ui;
mod utils;

use self::ui::{App, Connect, Event, State};
use fontfinder::dirs;
use fontfinder::fonts::{self, Sorting};
use std::process;

fn main() {
    glib::set_program_name("Font Finder".into());
    glib::set_application_name("Font Finder");

    // Initialize GTK before proceeding.
    if gtk::init().is_err() {
        eprintln!("failed to initialize GTK Application");
        process::exit(1);
    }

    // Grabs the local font directory, which is "~/.local/share/fonts/"
    let path = match dirs::font_cache() {
        Ok(path) => path,
        Err(why) => {
            eprintln!("{}", why);
            process::exit(1);
        }
    };

    // Grab the information on Google's archive of free fonts.
    let fonts_archive = match fonts::obtain(Sorting::Trending) {
        Ok(fonts_archive) => fonts_archive,
        Err(why) => {
            eprintln!("failed to get font archive: {:?}", why);
            process::exit(1);
        }
    };

    let (tx, rx) = flume::unbounded();

    // Initializes the complete structure of the GTK application.
    // Contains all relevant widgets that we will manipulate.
    let mut app = App::new(State {
        path,
        fonts_archive,
        row_id: 0,
        tx: tx.clone(),
    });

    let (ntx, nrx) = flume::unbounded();

    // For each trigger, update the font cache.
    let font_cacher = async move {
        while let Ok(()) = nrx.recv_async().await {
            fontfinder::run_fc_cache().await;
        }
    };

    // Async event loop for handling all application events;
    let event_handler = async move {
        app.connect_events(tx);
        app.show();

        while let Ok(event) = rx.recv_async().await {
            match event {
                Event::Filter => app.filter_categories(),
                Event::Install => app.install(),
                Event::Select(row) => app.select_row(row),
                Event::Sort(sorting) => app.sort(sorting),
                Event::UpdatePreview => app.update_preview(),
                Event::Uninstall => app.uninstall(),

                Event::TriggerFontCache => {
                    let _ = ntx.send(());
                }
            }
        }
    };

    glib::MainContext::default().spawn_local(async move {
        let _ = futures::future::join(font_cacher, event_handler).await;
    });

    // Begins the main event loop of GTK, which will display the GUI and handle all
    // the actions that were mapped to each of the widgets in the UI.
    gtk::main();
}
