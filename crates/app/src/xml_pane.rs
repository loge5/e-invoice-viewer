//! The XML view: formatted, highlighted, searchable, with copy and save of
//! the original bytes (spec 0008).

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use e_invoice_viewer_core::xml_view::{decode, pretty_print};
use gtk::{gio, glib};
use sourceview::prelude::*;

use crate::texts;

/// The invoice XML currently shown.
struct Source {
    /// Original bytes, written unchanged by "XML speichern unter…".
    bytes: Vec<u8>,
    /// Original text, put on the clipboard by "XML kopieren".
    text: String,
    /// Name of the opened file, for the proposed save name.
    file_name: String,
    /// Whether the formatted text has been loaded into the view yet.
    loaded: bool,
}

#[derive(Clone)]
pub struct XmlPane {
    root: gtk::Box,
    buffer: sourceview::Buffer,
    view: sourceview::View,
    search_bar: gtk::SearchBar,
    search_entry: gtk::SearchEntry,
    search_status: gtk::Label,
    search: sourceview::SearchContext,
    source: Rc<RefCell<Option<Source>>>,
}

impl XmlPane {
    pub fn new() -> Self {
        let buffer = sourceview::Buffer::new(None);
        buffer.set_language(
            sourceview::LanguageManager::default()
                .language("xml")
                .as_ref(),
        );
        buffer.set_highlight_matching_brackets(false);
        let view = sourceview::View::builder()
            .buffer(&buffer)
            .editable(false)
            .monospace(true)
            .show_line_numbers(true)
            .wrap_mode(gtk::WrapMode::None)
            .left_margin(6)
            .top_margin(6)
            .bottom_margin(6)
            .build();
        follow_dark_mode(&buffer);

        let settings = sourceview::SearchSettings::new();
        settings.set_case_sensitive(false);
        settings.set_wrap_around(true);
        let search = sourceview::SearchContext::new(&buffer, Some(&settings));
        search.set_highlight(true);

        let search_entry = gtk::SearchEntry::builder()
            .placeholder_text(texts::SEARCH_PLACEHOLDER)
            .hexpand(true)
            .build();
        let search_status = gtk::Label::builder()
            .css_classes(["dim-label", "numeric"])
            .width_chars(12)
            .xalign(1.0)
            .build();
        let search_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        search_box.append(&search_entry);
        search_box.append(&search_status);
        let search_bar = gtk::SearchBar::builder()
            .child(&search_box)
            .show_close_button(true)
            .build();
        search_bar.connect_entry(&search_entry);

        let copy_button = gtk::Button::builder().label(texts::COPY_XML).build();
        let save_button = gtk::Button::builder().label(texts::SAVE_XML).build();
        let actions = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .margin_start(6)
            .margin_end(6)
            .margin_top(6)
            .margin_bottom(6)
            .halign(gtk::Align::End)
            .build();
        actions.append(&copy_button);
        actions.append(&save_button);

        let scroller = gtk::ScrolledWindow::builder()
            .child(&view)
            .vexpand(true)
            .build();
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.append(&actions);
        root.append(&search_bar);
        root.append(&scroller);

        let this = Self {
            root,
            buffer,
            view,
            search_bar,
            search_entry,
            search_status,
            search,
            source: Rc::default(),
        };
        this.connect_signals(&copy_button, &save_button);
        this
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }

    /// Sets the invoice XML; it is formatted when the view is first shown.
    pub fn set_xml(&self, bytes: &[u8], file_name: &str) {
        let text = decode(bytes)
            .map(|text| text.into_owned())
            .unwrap_or_default();
        *self.source.borrow_mut() = Some(Source {
            bytes: bytes.to_vec(),
            text,
            file_name: file_name.to_string(),
            loaded: false,
        });
        self.buffer.set_text("");
        self.search_entry.set_text("");
        self.search_bar.set_search_mode(false);
    }

    pub fn clear(&self) {
        *self.source.borrow_mut() = None;
        self.buffer.set_text("");
    }

    /// Loads the formatted XML into the view if not done yet.
    pub fn ensure_loaded(&self) {
        let mut source = self.source.borrow_mut();
        if let Some(source) = source.as_mut().filter(|source| !source.loaded) {
            self.buffer.set_text(&pretty_print(&source.text));
            self.buffer.place_cursor(&self.buffer.start_iter());
            source.loaded = true;
        }
    }

    pub fn start_search(&self) {
        self.search_bar.set_search_mode(true);
        self.search_entry.grab_focus();
    }

    fn connect_signals(&self, copy_button: &gtk::Button, save_button: &gtk::Button) {
        let this = self.clone();
        copy_button.connect_clicked(move |button| {
            if let Some(source) = this.source.borrow().as_ref() {
                button.clipboard().set_text(&source.text);
            }
        });

        let this = self.clone();
        save_button.connect_clicked(move |button| this.save(button));

        let this = self.clone();
        self.search_entry.connect_search_changed(move |entry| {
            let query = entry.text();
            this.search
                .settings()
                .set_search_text((!query.is_empty()).then_some(query.as_str()));
            this.select_next_match(true);
        });

        let this = self.clone();
        self.search_entry
            .connect_activate(move |_| this.select_next_match(false));

        let this = self.clone();
        self.search
            .connect_occurrences_count_notify(move |_| this.update_search_status());
    }

    /// Selects the next match after the cursor (or from the start).
    fn select_next_match(&self, from_start: bool) {
        let from = if from_start {
            self.buffer.start_iter()
        } else {
            match self.buffer.selection_bounds() {
                Some((_, end)) => end,
                None => self.buffer.iter_at_mark(&self.buffer.get_insert()),
            }
        };
        if let Some((start, end, _wrapped)) = self.search.forward(&from) {
            self.buffer.select_range(&start, &end);
            let mut start = start;
            self.view.scroll_to_iter(&mut start, 0.1, false, 0.0, 0.0);
        }
        self.update_search_status();
    }

    fn update_search_status(&self) {
        if self.search.settings().search_text().is_none() {
            self.search_status.set_text("");
            return;
        }
        let total = self.search.occurrences_count();
        if total < 0 {
            // Still counting.
            return;
        }
        let current = self
            .buffer
            .selection_bounds()
            .map(|(start, end)| self.search.occurrence_position(&start, &end))
            .filter(|position| *position > 0)
            .and_then(|position| usize::try_from(position).ok());
        let total = usize::try_from(total).unwrap_or(0);
        self.search_status
            .set_text(&texts::search_status_text(current, total));
    }

    fn save(&self, button: &gtk::Button) {
        let Some((bytes, suggested)) = self.source.borrow().as_ref().map(|source| {
            (
                source.bytes.clone(),
                texts::suggested_xml_file_name(&source.file_name),
            )
        }) else {
            return;
        };
        let dialog = gtk::FileDialog::builder()
            .title(texts::SAVE_XML)
            .modal(true)
            .initial_name(suggested)
            .build();
        let parent = button.root().and_downcast::<gtk::Window>();
        glib::spawn_future_local(async move {
            let Ok(file) = dialog.save_future(parent.as_ref()).await else {
                return;
            };
            let result = file
                .replace_contents_future(bytes, None, false, gio::FileCreateFlags::NONE)
                .await;
            if let (Err((_, error)), Some(parent)) = (result, parent) {
                let alert = adw::AlertDialog::new(
                    Some("XML konnte nicht gespeichert werden"),
                    Some(error.message()),
                );
                alert.add_response("close", "Schließen");
                alert.present(Some(&parent));
            }
        });
    }
}

/// Uses the light or dark Adwaita source style, following the system.
fn follow_dark_mode(buffer: &sourceview::Buffer) {
    let style_manager = adw::StyleManager::default();
    let apply = {
        let buffer = buffer.clone();
        move |manager: &adw::StyleManager| {
            let id = if manager.is_dark() {
                "Adwaita-dark"
            } else {
                "Adwaita"
            };
            buffer.set_style_scheme(
                sourceview::StyleSchemeManager::default()
                    .scheme(id)
                    .as_ref(),
            );
        }
    };
    apply(&style_manager);
    style_manager.connect_dark_notify(apply);
}
