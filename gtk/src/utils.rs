use gtk::prelude::*;
use gtk::{TextBuffer, SearchEntry};

/// Obtains the entire inner string of a given text buffer.
pub(crate) fn get_buffer(buffer: &TextBuffer) -> Option<String> {
    let start = buffer.get_start_iter();
    let end = buffer.get_end_iter();
    buffer.get_text(&start, &end, true)
}

/// Obtains the value of the search entry from the UI
pub(crate) fn get_search(search: &SearchEntry) -> Option<String> {
    match search.get_text().take() {
        Some(ref text) if text.is_empty() => None,
        Some(text) => Some(text),
        None => None,
    }
}

/// A simple convenience function for adding a style class to a widget.
pub(crate) fn set_class<W: WidgetExt>(widget: &W, class: &str) {
    widget.get_style_context().map(|c| c.add_class(class));
}

pub(crate) fn set_margin<W: WidgetExt>(widget: &W, t: i32, r: i32, b: i32, l: i32) {
    widget.set_margin_top(t);
    widget.set_margin_end(r);
    widget.set_margin_bottom(b);
    widget.set_margin_start(l);
}
