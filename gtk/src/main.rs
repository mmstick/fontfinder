#![allow(unknown_lints)]
#![allow(option_map_unit_fn)]

#[macro_use]
extern crate cascade;

mod utils;
mod ui;

use fontfinder::fc_cache::fc_cache_event_loop;
use fontfinder::{dirs, FontError};
use fontfinder::fonts::{self, Sorting};
use self::ui::{App, Connect, State};
use std::process;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicUsize;

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
    let path = match dirs::font_cache().ok_or(FontError::FontDirectory) {
        Ok(path) => path,
        Err(why) => {
            eprintln!("failed to get font archive: {}", why);
            process::exit(1);
        }
    };

    // Grab the information on Google's archive of free fonts.
    // I'm wrapping it in Arc so it can be shared across multiple closures.
    let fonts_archive = match fonts::obtain(Sorting::Trending) {
        Ok(fonts_archive) => RwLock::new(fonts_archive),
        Err(why) => {
            eprintln!("failed to get font archive: {}", why);
            process::exit(1);
        }
    };

    // Contains the ID of the currently-selected row, to cut down on lookups.
    let row_id = AtomicUsize::new(0);

    let app = {
        // Collect a list of unique categories from that font list.
        let categories = fonts_archive.read().unwrap().get_categories();

        // Initializes the complete structure of the GTK application.
        // Contains all relevant widgets that we will manipulate.
        Rc::new(App::new(&fonts_archive.read().unwrap(), &categories))
    };

    let state = Arc::new(State {
        path,
        fonts_archive,
        row_id,
    });

    app.connect_events(state).then_execute();
}
