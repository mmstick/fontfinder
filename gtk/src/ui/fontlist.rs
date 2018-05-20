use fontfinder::fonts::Font;
use gtk::prelude::*;
use gtk::{Align, Label, ListBox, ListBoxRow, ScrolledWindow};
use std::cell::{Ref, RefCell};

#[derive(Clone)]
pub struct FontList {
    pub container: ListBox,
    pub scroller: ScrolledWindow,
    fonts: RefCell<Vec<FontRow>>,
}

impl FontList {
    pub fn new(fonts_archive: &[Font]) -> FontList {
        let container = ListBox::new();
        let mut fonts = Vec::with_capacity(fonts_archive.len());
        for font in fonts_archive {
            let row = FontRow::new(
                font.category.clone(),
                font.family.clone(),
                font.files.keys().cloned().collect(),
            );
            container.insert(&row.container, -1);
            fonts.push(row);
        }

        // Allows the font list box to scroll
        let scroller = ScrolledWindow::new(None, None);
        scroller.set_min_content_width(200);
        scroller.add(&container);

        FontList {
            container,
            scroller,
            fonts: RefCell::new(fonts),
        }
    }

    pub fn update(&self, fonts_archive: &[Font]) {
        self.container
            .get_children()
            .iter()
            .for_each(|c| c.destroy());
        let mut fonts = self.fonts.borrow_mut();

        fonts.clear();

        for font in fonts_archive {
            let row = FontRow::new(
                font.category.clone(),
                font.family.clone(),
                font.files.keys().cloned().collect(),
            );
            self.container.insert(&row.container, -1);
            fonts.push(row);
        }

        self.container.show_all();
    }

    pub fn get_rows<'a>(&'a self) -> Ref<'a, Vec<FontRow>> {
        self.fonts.borrow()
    }
}

#[derive(Clone)]
pub struct FontRow {
    pub container: ListBoxRow,
    pub category: String,
    pub family: String,
    pub variants: Vec<String>,
}

impl FontRow {
    pub fn new(category: String, family: String, variants: Vec<String>) -> FontRow {
        // Create the inner label of the row that contains the family in bold.
        let label = Label::new("");
        label.set_markup(&["<b>", family.as_str(), "</b>"].concat());
        label.set_halign(Align::Start);
        label.set_margin_top(3);
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

    pub fn set_visibility(&self, visibility: bool) {
        self.container.set_visible(visibility);
    }

    pub fn contains(&self, pattern: &str) -> bool {
        // TODO: do this without making any allocations.
        self.family.to_lowercase().contains(&pattern.to_lowercase())
    }
}
