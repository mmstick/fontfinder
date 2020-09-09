use super::FontList;
use crate::utils::set_margin;
use fontfinder::fonts::Font;
use gtk;
use gtk::prelude::*;
use std::rc::Rc;
use webkit2gtk::*;

#[derive(Clone)]
pub struct Main {
    pub container: gtk::Paned,
    pub categories: gtk::ComboBoxText,
    pub sort_by: gtk::ComboBoxText,
    pub fonts: Rc<FontList>,
    pub context: WebContext,
    pub view: WebView,
    pub sample_text: gtk::TextView,
    pub sample_buffer: gtk::TextBuffer,
    pub search: gtk::SearchEntry,
}

impl Main {
    pub fn new(fonts_archive: &[Font], categories: &[String]) -> Main {
        let fonts = FontList::new(fonts_archive);

        // The category menu for filtering based on category.
        let menu = cascade! {
            menu: gtk::ComboBoxText::new();
            | set_margin(&menu, 3, 5, 0, 5);
            ..append_text("All");
            | categories.iter().for_each(|c| menu.append_text(c.as_str()));
            ..set_active(Some(0));
        };

        // Ability to toggle between sorting methods.
        let sort_by = cascade! {
            sort_by: gtk::ComboBoxText::new();
            | set_margin(&sort_by, 3, 5, 0, 5);
            ..append_text("Trending");
            ..append_text("Popular");
            ..append_text("Date Added");
            ..append_text("Alphabetical");
            ..set_active(Some(0));
        };

        // Search bar beneath the category menu for doing name-based filters.
        let search = gtk::SearchEntry::new();
        set_margin(&search, 3, 5, 0, 5);

        // Construct the left pane's box
        let lbox = cascade! {
            gtk::Box::new(gtk::Orientation::Vertical, 0);
            ..add(&menu);
            ..add(&sort_by);
            ..add(&search);
            ..add(&gtk::Separator::new(gtk::Orientation::Horizontal));
            ..pack_start(&fonts.scroller, true, true, 0);
        };

        // Initializes the webkit2gtk preview that will display the fonts.
        let context = WebContext::get_default().unwrap();
        let view = WebView::new_with_context_and_user_content_manager(
            &context,
            &UserContentManager::new(),
        );

        // Initializes the sample text buffer that the preview is generated from.
        let buffer = gtk::TextBuffer::new(None::<&gtk::TextTagTable>);

        {
            // Set the text once the UI has loaded, so that it is not hidden.
            let buffer = buffer.clone();
            glib::idle_add_local(move || {
                buffer.set_text(
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor \
                     incididunt ut labore et dolore magna aliqua.",
                );
                Continue(false)
            });
        }

        // And assigns that text buffer to this text view, so the user can enter text
        // into it.
        let sample_text = cascade! {
            sample: gtk::TextView::with_buffer(&buffer);
            ..set_wrap_mode(gtk::WrapMode::Word);
            | set_margin(&sample, 5, 5, 5, 5);
        };

        // Wraps up the sample text and it's associated preview as the right panel.
        let rbox = cascade! {
            gtk::Box::new(gtk::Orientation::Vertical, 0);
            ..add(&sample_text);
            ..add(&gtk::Separator::new(gtk::Orientation::Horizontal));
            ..pack_start(&view, true, true, 0);
        };

        // Attaches all of contents of the window accordingly.
        let content = cascade! {
            gtk::Paned::new(gtk::Orientation::Horizontal);
            ..pack1(&lbox, false, false);
            ..pack2(&rbox, true, true);
        };

        Main {
            container: cascade! {
                gtk::Paned::new(gtk::Orientation::Vertical);
                ..pack1(&content, true, true);
            },
            categories: menu,
            fonts: Rc::new(fonts),
            context,
            view,
            sample_text,
            search,
            sample_buffer: buffer,
            sort_by,
        }
    }
}
