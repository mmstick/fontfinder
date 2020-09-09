use super::App;
use flume::Sender;
use fontfinder::fonts::Sorting;
use gtk;
use gtk::prelude::*;
use std::rc::Rc;

pub enum Event {
    Filter,
    Install,
    Select(usize),
    Sort(Sorting),
    UpdatePreview,
    Uninstall,
}

pub trait Connect<T> {
    fn connect_events(&self, sender: Sender<T>);
    fn connect_row_selected(&self, sender: Sender<T>);
    fn connect_preview_updates(&self, sender: Sender<T>);
    fn connect_filter_fonts(&self, sender: Sender<T>);
    fn connect_sorting(&self, sender: Sender<T>);
    fn connect_install(&self, sender: Sender<T>);
    fn connect_uninstall(&self, sender: Sender<T>);
}

impl Connect<Event> for App {
    fn connect_events(&self, sender: Sender<Event>) {
        self.connect_row_selected(sender.clone());
        self.connect_preview_updates(sender.clone());
        self.connect_filter_fonts(sender.clone());
        self.connect_sorting(sender.clone());
        self.connect_install(sender.clone());
        self.connect_uninstall(sender);
    }

    fn connect_install(&self, sender: Sender<Event>) {
        self.header.install.connect_clicked(move |_| {
            let _ = sender.send(Event::Install);
        });
    }

    fn connect_uninstall(&self, sender: Sender<Event>) {
        self.header.uninstall.connect_clicked(move |_| {
            let _ = sender.send(Event::Uninstall);
        });
    }

    fn connect_sorting(&self, sender: Sender<Event>) {
        self.main.sort_by.connect_changed(move |sort_by| {
            let _ = sender.send(Event::Sort(match sort_by.get_active() {
                Some(0) => Sorting::Trending,
                Some(1) => Sorting::Popular,
                Some(2) => Sorting::DateAdded,
                Some(3) => Sorting::Alphabetical,
                _ => unreachable!("unknown sorting"),
            }));
        });
    }

    fn connect_filter_fonts(&self, sender: Sender<Event>) {
        let filter = Rc::new(move || {
            let _ = sender.send(Event::Filter);
        });

        self.main.categories.connect_changed({
            let filter = filter.clone();
            move |_| filter()
        });

        self.main.search.connect_search_changed({
            let filter = filter.clone();
            move |_| filter()
        });

        self.header.show_installed.connect_toggled({
            let filter = filter.clone();
            move |_| filter()
        });
    }

    fn connect_preview_updates(&self, sender: Sender<Event>) {
        // This closure will be shared by multiple GTK signals.
        let update_preview = Rc::new(move || {
            let _ = sender.send(Event::UpdatePreview);
        });

        // Updates the preview when the font size has chanegd.
        self.header.font_size.connect_property_value_notify({
            let update_preview = update_preview.clone();
            move |_| update_preview()
        });

        // Updates the preview when the sample text has been modified.
        self.main.sample_buffer.connect_changed({
            let update_preview = update_preview.clone();
            move |_| update_preview()
        });

        // Updates the preview when the dark preview button has been toggled.
        self.header
            .dark_preview
            .connect_toggled(move |_| update_preview());
    }

    fn connect_row_selected(&self, sender: Sender<Event>) {
        self.main.fonts.connect_row_selected(move |_, row| {
            if let Some(row) = row.as_ref() {
                let _ = sender.send(Event::Select(row.get_index() as usize));
            }
        });
    }
}
