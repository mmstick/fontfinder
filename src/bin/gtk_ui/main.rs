use fontfinder::fonts::Font;
use gtk::*;
use std::cell::RefCell;
use std::rc::Rc;
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
        let container = Paned::new(Orientation::Horizontal);

        // Generate a list box from the list of fonts in the archive.
        let fonts_box = ListBox::new();
        let mut fonts = Vec::with_capacity(fonts_archive.len());
        for font in fonts_archive {
            let row = FontRow::new(font.category.clone(), font.family.clone());
            fonts_box.insert(&row.container, -1);
            fonts.push(row);
        }

        // Allows the font list box to scroll
        let scroller = ScrolledWindow::new(None, None);
        scroller.set_min_content_width(200);
        scroller.add(&fonts_box);

        // The category menu for filtering based on category.
        let menu = ComboBoxText::new();
        menu.insert_text(0, "All");
        for (id, category) in categories.iter().enumerate() {
            menu.insert_text((id + 1) as i32, category.as_str());
        }
        menu.set_active(0);

        // Search bar beneath the category menu for doing name-based filters.
        let search = SearchEntry::new();

        // Construct the left pane's box
        let lbox = Box::new(Orientation::Vertical, 0);
        lbox.pack_start(&menu, false, false, 0);
        lbox.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);
        lbox.pack_start(&search, false, false, 0);
        lbox.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);
        lbox.pack_start(&scroller, true, true, 0);

        let context = WebContext::get_default().unwrap();
        let view = WebView::new_with_context_and_user_content_manager(
            &context,
            &UserContentManager::new(),
        );

        let buffer = TextBuffer::new(None);
        buffer.set_text(
            "Lorem ipsum dolor sit amet, consectetur adipiscing \
                elit, sed do eiusmod tempor incididunt ut labore \
                et dolore magna aliqua.",
        );

        let sample_text = TextView::new_with_buffer(&buffer);
        sample_text.set_wrap_mode(WrapMode::Word);
        sample_text.set_left_margin(5);
        sample_text.set_right_margin(5);
        sample_text.set_top_margin(5);
        sample_text.set_bottom_margin(5);

        let rbox = Box::new(Orientation::Vertical, 0);
        rbox.pack_start(&sample_text, false, false, 0);
        rbox.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);
        rbox.pack_start(&view, true, true, 0);

        container.pack1(&lbox, false, false);
        container.pack2(&rbox, true, true);

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
}

impl FontRow {
    pub fn new(category: String, family: String) -> FontRow {
        // Create the inner label of the row that contains the family in bold.
        let label = Label::new("");
        label.set_markup(&["<b>", family.as_str(), "</b>"].concat());
        label.set_justify(Justification::Left);
        label.set_halign(Align::Start);

        // Store the label within the list box row.
        let container = ListBoxRow::new();
        container.add(&label);

        FontRow {
            container,
            category,
            family,
        }
    }

    pub fn set_visibility(&self, visibility: bool) { self.container.set_visible(visibility); }

    pub fn contains(&self, pattern: &str) -> bool {
        // TODO: do this without making any allocations.
        self.family.to_lowercase().contains(&pattern.to_lowercase())
    }
}
