use crate::utils::block_on;
use crate::Event;

use fontfinder::fonts::Font;
use gtk;
use gtk::prelude::*;
use std::cell::{Ref, RefCell};
use std::ops::Deref;

#[derive(Clone)]
pub struct FontList {
    container: gtk::ListBox,
    pub scroller: gtk::ScrolledWindow,
    fonts: RefCell<Vec<FontRow>>,
}

impl FontList {
    pub fn new(fonts_archive: &[Font], tx: async_channel::Sender<Event>) -> FontList {
        let fonts = Vec::with_capacity(fonts_archive.len());

        let container = cascade! {
            gtk::ListBox::new();
            ..connect_row_selected(move |_, row| {
                if let Some(row) = row.as_ref() {
                    let _ = block_on(tx.send(Event::Select(row.get_index() as usize)));
                }
            });
        };

        // Allows the font list box to scroll
        let scroller = cascade! {
            gtk::ScrolledWindow::new::<gtk::Adjustment, gtk::Adjustment>(None, None);
            ..set_property_hscrollbar_policy(gtk::PolicyType::Never);
            ..set_min_content_width(200);
            ..add(&container);
        };

        let list = FontList {
            container,
            scroller,
            fonts: RefCell::new(fonts),
        };

        list.update(fonts_archive);
        list
    }

    pub fn update(&self, fonts_archive: &[Font]) {
        self.container
            .get_children()
            .iter()
            .for_each(|c| unsafe { c.destroy() });
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
    container: gtk::ListBoxRow,
    pub category: String,
    pub family: String,
    pub variants: Vec<String>,
}

impl AsRef<gtk::ListBoxRow> for FontRow {
    fn as_ref(&self) -> &gtk::ListBoxRow {
        &self.container
    }
}

impl Deref for FontRow {
    type Target = gtk::ListBoxRow;
    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

impl FontRow {
    pub fn new(category: String, family: String, variants: Vec<String>) -> FontRow {
        // Create the inner label of the row that contains the family in bold.
        let label = cascade! {
            gtk::Label::new(None);
            ..set_markup(&["<b>", family.as_str(), "</b>"].concat());
            ..set_halign(gtk::Align::Start);
            ..set_margin_top(3);
            ..set_margin_start(6);
        };

        // Store the label within the list box row.
        let container = gtk::ListBoxRow::new();
        container.add(&label);

        FontRow {
            container,
            category,
            family,
            variants,
        }
    }

    pub fn contains(&self, pattern: &str) -> bool {
        self.family.to_lowercase().contains(&pattern.to_lowercase())
    }
}
