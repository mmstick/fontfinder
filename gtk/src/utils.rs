use std::future::Future;

use glib::GString;
use gtk::prelude::*;
use gtk::{SearchEntry, TextBuffer};

/// Block task on the GLib default executor
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    glib::MainContext::default().block_on(future)
}

/// Spawn new task on the GLib default executor
pub fn spawn_local<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    glib::MainContext::default().spawn_local(future);
}

/// Obtains the entire inner string of a given text buffer.
pub fn get_buffer(buffer: &TextBuffer) -> Option<GString> {
    let start = buffer.get_start_iter();
    let end = buffer.get_end_iter();
    buffer.get_text(&start, &end, true)
}

/// Obtains the value of the search entry from the UI
pub fn get_search(search: &SearchEntry) -> Option<GString> {
    let text = search.get_text();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// A simple convenience function for adding a style class to a widget.
pub fn set_class<W: WidgetExt>(widget: &W, class: &str) {
    widget.get_style_context().add_class(class);
}

pub fn set_margin<W: WidgetExt>(widget: &W, t: i32, r: i32, b: i32, l: i32) {
    widget.set_margin_top(t);
    widget.set_margin_end(r);
    widget.set_margin_bottom(b);
    widget.set_margin_start(l);
}
