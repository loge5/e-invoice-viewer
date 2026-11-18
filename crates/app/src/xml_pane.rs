//! The XML view: formatted, highlighted, searchable, with copy and save of
//! the original bytes (spec 0008).

use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use e_invoice_viewer_core::attribution::Target;
use e_invoice_viewer_core::fields::FieldModel;
use e_invoice_viewer_core::link::{XmlLayout, assignment};
use e_invoice_viewer_core::xml_view::{decode, pretty_print};
use gtk::glib;
use sourceview::prelude::*;

use crate::save_as::save_as;
use crate::texts;

/// The invoice XML currently shown.
struct Source {
    /// Original bytes, written unchanged by "XML speichern unter…".
    bytes: Vec<u8>,
    /// Name of the opened file, for the proposed save name.
    file_name: String,
    /// The field model, for the assignment line (0014).
    model: FieldModel,
    /// Set when the view is first shown: the formatted text with element
    /// positions, or `None` if the XML cannot be linked (it is still shown).
    layout: OnceCell<Option<Rc<XmlLayout>>>,
}

impl Source {
    /// The original text, as "XML kopieren" puts it on the clipboard.
    fn text(&self) -> String {
        decode(&self.bytes)
            .map(|text| text.into_owned())
            .unwrap_or_default()
    }

    /// The layout once the view has been shown and the XML can be linked.
    fn layout(&self) -> Option<&Rc<XmlLayout>> {
        self.layout.get().and_then(Option::as_ref)
    }
}

/// Name of the text tag marking the element shown by "Im XML zeigen".
const LINK_TAG: &str = "link-highlight";

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
    assignment_label: gtk::Label,
    show_in_fields: gtk::Button,
    /// What the element at the cursor belongs to.
    assigned: Rc<Cell<Target>>,
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
        buffer.create_tag(
            Some(LINK_TAG),
            &[(
                "background-rgba",
                &gtk::gdk::RGBA::new(0.21, 0.52, 0.89, 0.25),
            )],
        );

        let settings = sourceview::SearchSettings::new();
        settings.set_case_sensitive(false);
        settings.set_wrap_around(true);
        let search = sourceview::SearchContext::new(&buffer, Some(&settings));
        search.set_highlight(true);

        let search_entry = gtk::SearchEntry::builder()
            .placeholder_text(texts::t(texts::SEARCH_PLACEHOLDER))
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

        let copy_button = gtk::Button::builder()
            .label(texts::t(texts::COPY_XML))
            .build();
        let save_button = gtk::Button::builder()
            .label(texts::t(texts::SAVE_XML))
            .build();
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
        // The assignment line (0014), separate from the validation status.
        let assignment_label = gtk::Label::builder()
            .label(texts::t(texts::NO_FIELD))
            .xalign(0.0)
            .hexpand(true)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .build();
        let show_in_fields = gtk::Button::builder()
            .label(texts::t(texts::SHOW_IN_FIELDS))
            .sensitive(false)
            .build();
        let assignment_line = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .margin_start(6)
            .margin_end(6)
            .margin_top(6)
            .margin_bottom(6)
            .build();
        assignment_line.append(&assignment_label);
        assignment_line.append(&show_in_fields);

        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.append(&actions);
        root.append(&search_bar);
        root.append(&scroller);
        root.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        root.append(&assignment_line);

        let this = Self {
            root,
            buffer,
            view,
            search_bar,
            search_entry,
            search_status,
            search,
            source: Rc::default(),
            assignment_label,
            show_in_fields,
            assigned: Rc::new(Cell::new(Target::Document)),
        };
        this.connect_signals(&copy_button, &save_button);
        this
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }

    /// Sets the invoice XML; it is formatted when the view is first shown.
    pub fn set_xml(&self, bytes: &[u8], file_name: &str, model: FieldModel) {
        *self.source.borrow_mut() = Some(Source {
            bytes: bytes.to_vec(),
            file_name: file_name.to_string(),
            model,
            layout: OnceCell::new(),
        });
        self.set_assigned(Target::Document, texts::t(texts::NO_FIELD));
        self.buffer.set_text("");
        self.search_entry.set_text("");
        self.search_bar.set_search_mode(false);
    }

    pub fn clear(&self) {
        *self.source.borrow_mut() = None;
        self.buffer.set_text("");
        self.set_assigned(Target::Document, texts::t(texts::NO_FIELD));
    }

    /// Calls `show` with the field or section of the assignment line when
    /// "In Feldern zeigen" is clicked.
    pub fn connect_show_in_fields(&self, show: impl Fn(Target) + 'static) {
        let assigned = self.assigned.clone();
        self.show_in_fields
            .connect_clicked(move |_| show(assigned.get()));
    }

    /// Shows the element at `location`: scrolls it into view, marks it from
    /// start to end tag and places the cursor in its text or in the value
    /// of `attribute`.
    pub fn reveal(&self, location: &str, attribute: Option<&str>) {
        self.ensure_loaded();
        let Some(layout) = self
            .source
            .borrow()
            .as_ref()
            .and_then(|s| s.layout().cloned())
        else {
            return;
        };
        let (Some(range), Some(cursor)) = (
            layout.element_range(location),
            layout.cursor_offset(location, attribute),
        ) else {
            return;
        };
        let iter_at = |offset: usize| {
            let (line, index) = layout.position(offset);
            let line = i32::try_from(line).unwrap_or(i32::MAX);
            let index = i32::try_from(index).unwrap_or(i32::MAX);
            self.buffer.iter_at_line_index(line, index)
        };
        let (start, end) = (iter_at(range.start), iter_at(range.end));
        let (Some(start), Some(end), Some(cursor)) = (start, end, iter_at(cursor)) else {
            return;
        };
        self.buffer.remove_tag_by_name(
            LINK_TAG,
            &self.buffer.start_iter(),
            &self.buffer.end_iter(),
        );
        self.buffer.apply_tag_by_name(LINK_TAG, &start, &end);
        self.buffer.place_cursor(&cursor);
        let view = self.view.clone();
        let mark = self.buffer.create_mark(None, &cursor, true);
        // After switching to the XML view, wait for the layout.
        glib::idle_add_local_once(move || {
            view.scroll_to_mark(&mark, 0.1, true, 0.0, 0.3);
            view.grab_focus();
            if let Some(buffer) = mark.buffer() {
                buffer.delete_mark(&mark);
            }
        });
    }

    /// Updates the assignment line for the cursor position.
    fn update_assignment(&self) {
        let source = self.source.borrow();
        let Some((model, layout)) = source
            .as_ref()
            .and_then(|source| Some((&source.model, source.layout()?)))
        else {
            return;
        };
        let cursor = self.buffer.iter_at_mark(&self.buffer.get_insert());
        let line = usize::try_from(cursor.line()).unwrap_or_default();
        let index = usize::try_from(cursor.line_index()).unwrap_or_default();
        let target = assignment(model, layout, layout.offset(line, index));
        let text = texts::assignment_text(model, &target);
        drop(source);
        self.set_assigned(target, &text);
    }

    fn set_assigned(&self, target: Target, text: &str) {
        self.assigned.set(target);
        self.assignment_label.set_text(text);
        self.show_in_fields
            .set_sensitive(target != Target::Document);
    }

    /// Loads the formatted XML into the view if not done yet.
    pub fn ensure_loaded(&self) {
        let text = {
            let source = self.source.borrow();
            let Some(source) = source
                .as_ref()
                .filter(|source| source.layout.get().is_none())
            else {
                return;
            };
            let layout = XmlLayout::new(&source.bytes).ok().map(Rc::new);
            let text = layout.as_ref().map_or_else(
                || pretty_print(&source.text()),
                |layout| layout.text().to_string(),
            );
            let _ = source.layout.set(layout);
            text
        };
        // Setting the text moves the cursor, which reads the source.
        self.buffer.set_text(&text);
        self.buffer.place_cursor(&self.buffer.start_iter());
        self.update_assignment();
    }

    pub fn start_search(&self) {
        self.search_bar.set_search_mode(true);
        self.search_entry.grab_focus();
    }

    fn connect_signals(&self, copy_button: &gtk::Button, save_button: &gtk::Button) {
        let this = self.clone();
        copy_button.connect_clicked(move |button| {
            if let Some(source) = this.source.borrow().as_ref() {
                button.clipboard().set_text(&source.text());
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

        let this = self.clone();
        self.buffer
            .connect_cursor_position_notify(move |_| this.update_assignment());
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
        let parent = button.root().and_downcast::<gtk::Window>();
        glib::spawn_future_local(async move {
            let result = save_as(
                parent.as_ref(),
                texts::t(texts::SAVE_XML),
                &suggested,
                glib::Bytes::from_owned(bytes),
            )
            .await;
            if let (Err(error), Some(parent)) = (result, parent) {
                let alert = adw::AlertDialog::new(
                    Some(texts::t(texts::SAVE_XML_FAILED)),
                    Some(error.message()),
                );
                alert.add_response("close", texts::t(texts::CLOSE_DIALOG));
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
