use super::set_margin;
use fontfinder::fonts::Font;
use gtk::*;
use std::{cell::RefCell, rc::Rc};
use webkit2gtk::*;

#[derive(Clone)]
pub struct Main {
    pub container:     Paned,
    pub categories:    ComboBoxText,
    pub fonts_box:     ListBox,
    pub fonts:         Rc<RefCell<Vec<FontRow>>>,
    pub context:       WebContext,
    pub view:          WebView,
    pub sample_text:   TextView,
    pub sample_buffer: TextBuffer,
    pub search:        SearchEntry,
}

impl Main {
    pub fn new(fonts_archive: &[Font], categories: &[String]) -> Main {
        let container = Paned::new(Orientation::Vertical);
        let content = Paned::new(Orientation::Horizontal);

        // Generate a list box from the list of fonts in the archive.
        let fonts_box = ListBox::new();
        let mut fonts = Vec::with_capacity(fonts_archive.len());
        for font in fonts_archive {
            let row = FontRow::new(
                font.category.clone(),
                font.family.clone(),
                font.files.keys().cloned().collect(),
            );
            fonts_box.insert(&row.container, -1);
            fonts.push(row);
        }

        // Allows the font list box to scroll
        let scroller = ScrolledWindow::new(None, None);
        scroller.set_min_content_width(200);
        scroller.add(&fonts_box);

        // The category menu for filtering based on category.
        let menu = ComboBoxText::new();
        set_margin(&menu, 5, 5, 5, 5);
        menu.insert_text(0, "All");
        for (id, category) in categories.iter().enumerate() {
            menu.insert_text((id + 1) as i32, category.as_str());
        }
        menu.set_active(0);

        // Search bar beneath the category menu for doing name-based filters.
        let search = SearchEntry::new();
        set_margin(&search, 0, 5, 5, 5);

        // Construct the left pane's box
        let lbox = Box::new(Orientation::Vertical, 0);
        lbox.pack_start(&menu, false, false, 0);
        lbox.pack_start(&search, false, false, 0);
        lbox.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);
        lbox.pack_start(&scroller, true, true, 0);

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

        // And assigns that text buffer to this text view, so the user can enter text into it.
        let sample_text = TextView::new_with_buffer(&buffer);
        sample_text.set_wrap_mode(WrapMode::Word);
        set_view_margins(&sample_text);

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
            fonts_box,
            fonts: Rc::new(RefCell::new(fonts)),
            context,
            view,
            sample_text,
            search,
            sample_buffer: buffer,
        }
    }
}

#[derive(Clone)]
pub struct FontRow {
    pub container: ListBoxRow,
    pub category:  String,
    pub family:    String,
    pub variants:  Vec<String>,
}

impl FontRow {
    pub fn new(category: String, family: String, variants: Vec<String>) -> FontRow {
        // Create the inner label of the row that contains the family in bold.
        let label = Label::new("");
        label.set_markup(&["<b>", family.as_str(), "</b>"].concat());
        label.set_halign(Align::Start);
        label.set_margin_top(5);
        label.set_margin_start(6);

        // Store the label within the list box row.
        let container = ListBoxRow::new();
        container.add(&label);

        FontRow {
            container,
            category,
            family,
            variants,
        }
    }

    pub fn set_visibility(&self, visibility: bool) { self.container.set_visible(visibility); }

    pub fn contains(&self, pattern: &str) -> bool {
        // TODO: do this without making any allocations.
        self.family.to_lowercase().contains(&pattern.to_lowercase())
    }
}

fn set_view_margins(view: &TextView) {
    view.set_top_margin(5);
    view.set_right_margin(5);
    view.set_bottom_margin(5);
    view.set_left_margin(5);
}
