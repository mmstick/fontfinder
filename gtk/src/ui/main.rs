use utils::set_margin;
use fontfinder::fonts::Font;
use gtk::*;
use std::rc::Rc;
use webkit2gtk::*;
use super::FontList;

#[derive(Clone)]
pub struct Main {
    pub container: Paned,
    pub categories: ComboBoxText,
    pub sort_by: ComboBoxText,
    pub fonts: Rc<FontList>,
    pub context: WebContext,
    pub view: WebView,
    pub sample_text: TextView,
    pub sample_buffer: TextBuffer,
    pub search: SearchEntry,
}

impl Main {
    pub fn new(fonts_archive: &[Font], categories: &[String]) -> Main {
        let container = Paned::new(Orientation::Vertical);
        let content = Paned::new(Orientation::Horizontal);

        let fonts = FontList::new(fonts_archive);

        // The category menu for filtering based on category.
        let menu = ComboBoxText::new();
        set_margin(&menu, 3, 5, 0, 5);
        menu.insert_text(0, "All");
        for (id, category) in categories.iter().enumerate() {
            menu.insert_text((id + 1) as i32, category.as_str());
        }
        menu.set_active(0);

        // Ability to toggle between sorting methods.
        let sort_by = ComboBoxText::new();
        set_margin(&sort_by, 3, 5, 0, 5);
        sort_by.insert_text(0, "Trending");
        sort_by.insert_text(1, "Popular");
        sort_by.insert_text(2, "Date Added");
        sort_by.insert_text(3, "Alphabetical");
        sort_by.set_active(0);

        // Search bar beneath the category menu for doing name-based filters.
        let search = SearchEntry::new();
        set_margin(&search, 3, 5, 0, 5);

        // Construct the left pane's box
        let lbox = Box::new(Orientation::Vertical, 0);
        lbox.pack_start(&menu, false, false, 0);
        lbox.pack_start(&sort_by, false, false, 0);
        lbox.pack_start(&search, false, false, 0);
        lbox.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);
        lbox.pack_start(&fonts.scroller, true, true, 0);

        // Initializes the webkit2gtk preview that will display the fonts.
        let context = WebContext::get_default().unwrap();
        let view = WebView::new_with_context_and_user_content_manager(
            &context,
            &UserContentManager::new(),
        );

        // Initializes the sample text buffer that the preview is generated from.
        let buffer = TextBuffer::new(None);
        buffer.set_text(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor \
             incididunt ut labore et dolore magna aliqua.",
        );

        // And assigns that text buffer to this text view, so the user can enter text
        // into it.
        let sample_text = TextView::new_with_buffer(&buffer);
        sample_text.set_wrap_mode(WrapMode::Word);
        set_margin(&sample_text, 5, 5, 5, 5);

        // Wraps up the sample text and it's associated preview as the right panel.
        let rbox = Box::new(Orientation::Vertical, 0);
        rbox.pack_start(&sample_text, false, false, 0);
        rbox.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);
        rbox.pack_start(&view, true, true, 0);

        // Attaches all of contents of the window accordingly.
        content.pack1(&lbox, false, false);
        content.pack2(&rbox, true, true);
        container.pack1(&content, true, true);

        Main {
            container,
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
