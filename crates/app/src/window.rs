//! The main window: start view, or PDF and fields side by side (spec 0007).

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;

use adw::prelude::*;
use e_invoice_viewer_core::attachments::{Attachment, Content};
use e_invoice_viewer_core::detect::DetectError;
use e_invoice_viewer_core::document::{self, Document, Invoice};
use e_invoice_viewer_core::fields::FieldModel;
use e_invoice_viewer_core::i18n::Language;
use e_invoice_viewer_core::validate::{Validation, Validator};
use gtk::{gio, glib};

use crate::fields_view::{FieldsView, ShowInXml};
use crate::language_setting::Choice;
use crate::pdf_pane::PdfPane;
use crate::recent::RecentStore;
use crate::save_as::save_as;
use crate::status_line::StatusLine;
use crate::texts::ValidationState;
use crate::window_size::{self, MINIMUM_SIZE, SizeKeeper};
use crate::xml_pane::XmlPane;
use crate::{attachments_view, local, texts};

thread_local! {
    /// One validator per worker thread, so that the rules are compiled once
    /// per thread and kept (ADR 0002).
    static VALIDATOR: Validator = Validator::new();
}

/// Validates on the calling (worker) thread, with notes in `language`;
/// `embedded` for the invoice of a PDF (spec 0027).
fn validate(xml: &[u8], embedded: bool, language: Language) -> Result<Validation, DetectError> {
    VALIDATOR.with(|validator| {
        if embedded {
            validator.validate_embedded(xml, language)
        } else {
            validator.validate(xml, language)
        }
    })
}

const MINIMUM_PANE_WIDTH: i32 = 300;

/// The document a window shows.
#[derive(Clone)]
struct Shown {
    /// File name.
    name: String,
    bytes: Rc<[u8]>,
}

/// Widgets and state of one window.
#[derive(Clone)]
pub struct Window {
    window: adw::ApplicationWindow,
    title: adw::WindowTitle,
    close_button: gtk::Button,
    stack: gtk::Stack,
    start_page: adw::StatusPage,
    pdf_pane: gtk::Stack,
    pdf_view: PdfPane,
    note_slot: gtk::Box,
    fields_container: gtk::Box,
    view_stack: gtk::Stack,
    fields_toggle: gtk::ToggleButton,
    xml_toggle: gtk::ToggleButton,
    xml_pane: XmlPane,
    paned: gtk::Paned,
    status: Rc<StatusLine>,
    /// Field pane of the shown invoice, if it has one.
    fields_view: Rc<RefCell<Option<FieldsView>>>,
    /// Counts opened documents; a validation result for an older one is
    /// discarded.
    generation: Rc<Cell<u64>>,
    /// Name and bytes of the shown document, `None` in the start view; kept
    /// to show it again in another language (0019).
    current: Rc<RefCell<Option<Shown>>>,
    open_button: gtk::Button,
    recent_button: gtk::MenuButton,
    menu_button: gtk::MenuButton,
    start_button: gtk::Button,
    pdf_message: adw::StatusPage,
    toasts: adw::ToastOverlay,
    /// Remembers the window size (0015).
    size: SizeKeeper,
    /// Recently opened files, shared by all windows.
    recent: Rc<RecentStore>,
}

impl Window {
    pub fn new(app: &adw::Application, recent: &Rc<RecentStore>) -> Self {
        let title = adw::WindowTitle::new(texts::APP_TITLE, "");
        let open_button = gtk::Button::builder().action_name("win.open").build();
        let close_button = gtk::Button::builder()
            .action_name("win.close")
            .visible(false)
            .build();
        let recent_button = gtk::MenuButton::builder()
            .icon_name("document-open-recent-symbolic")
            .menu_model(recent.menu())
            .build();
        // The main menu opens with F10 (0019).
        let menu_button = gtk::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .primary(true)
            .build();
        let header = adw::HeaderBar::builder().title_widget(&title).build();
        header.pack_start(&open_button);
        header.pack_start(&recent_button);
        header.pack_end(&menu_button);
        header.pack_end(&close_button);

        let start_button = gtk::Button::builder()
            .action_name("win.open")
            .halign(gtk::Align::Center)
            .css_classes(["pill", "suggested-action"])
            .build();
        let start_page = adw::StatusPage::builder()
            .icon_name("x-office-document-symbolic")
            .child(&start_button)
            .build();

        let pdf_view = PdfPane::new();
        let pdf_message = adw::StatusPage::builder()
            .icon_name("image-missing-symbolic")
            .css_classes(["compact"])
            .build();
        let pdf_pane = gtk::Stack::builder()
            .width_request(MINIMUM_PANE_WIDTH)
            .hexpand(true)
            .build();
        pdf_pane.add_named(pdf_view.widget(), Some("page"));
        pdf_pane.add_named(&pdf_message, Some("message"));

        let note_slot = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let fields_container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .vexpand(true)
            .build();
        let xml_pane = XmlPane::new();
        let view_stack = gtk::Stack::builder().vexpand(true).build();
        view_stack.add_named(&fields_container, Some("fields"));
        view_stack.add_named(xml_pane.widget(), Some("xml"));

        let fields_toggle = gtk::ToggleButton::builder().active(true).build();
        let xml_toggle = gtk::ToggleButton::builder()
            .group(&fields_toggle)
            .sensitive(false)
            .build();
        let toggles = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .halign(gtk::Align::Center)
            .margin_top(6)
            .margin_bottom(6)
            .css_classes(["linked"])
            .build();
        toggles.append(&fields_toggle);
        toggles.append(&xml_toggle);

        let status = Rc::new(StatusLine::new());

        let fields_pane = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .width_request(MINIMUM_PANE_WIDTH)
            .hexpand(true)
            .build();
        fields_pane.append(&note_slot);
        fields_pane.append(status.widget());
        fields_pane.append(&toggles);
        fields_pane.append(&view_stack);

        let paned = gtk::Paned::builder()
            .orientation(gtk::Orientation::Horizontal)
            .start_child(&pdf_pane)
            .end_child(&fields_pane)
            .shrink_start_child(false)
            .shrink_end_child(false)
            .resize_start_child(true)
            .resize_end_child(true)
            .position(window_size::DEFAULT_SIZE.0 / 2)
            .build();

        let stack = gtk::Stack::new();
        stack.add_named(&start_page, Some("start"));
        stack.add_named(&paned, Some("document"));

        let toasts = adw::ToastOverlay::new();
        toasts.set_child(Some(&stack));
        let content = adw::ToolbarView::new();
        content.add_top_bar(&header);
        content.set_content(Some(&toasts));

        let size = SizeKeeper::initial_state();
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title(texts::APP_TITLE)
            .default_width(size.width)
            .default_height(size.height)
            .maximized(size.maximized)
            .width_request(MINIMUM_SIZE.0)
            .height_request(MINIMUM_SIZE.1)
            .content(&content)
            .build();

        let this = Self {
            window: window.clone(),
            title,
            close_button,
            stack,
            start_page,
            pdf_pane,
            pdf_view,
            note_slot,
            fields_container,
            view_stack,
            fields_toggle,
            xml_toggle,
            xml_pane,
            paned,
            status,
            fields_view: Rc::default(),
            generation: Rc::default(),
            current: Rc::default(),
            open_button,
            recent_button,
            menu_button,
            start_button,
            pdf_message,
            toasts,
            size: SizeKeeper::attach(&window, size),
            recent: recent.clone(),
        };
        this.apply_texts();
        this.add_actions();
        this.add_drop_target();
        this
    }

    pub fn present(&self) {
        self.window.present();
    }

    /// Sets the texts of the header bar, the start view and the view switch
    /// in the language of the user interface (0019).
    fn apply_texts(&self) {
        self.open_button.set_label(texts::t(texts::OPEN_BUTTON));
        self.close_button.set_label(texts::t(texts::CLOSE_BUTTON));
        self.start_button.set_label(texts::t(texts::OPEN_BUTTON));
        self.start_page.set_title(texts::t(texts::START_TITLE));
        self.pdf_message
            .set_title(texts::t(texts::PDF_NOT_RENDERABLE));
        self.fields_toggle.set_label(texts::t(texts::VIEW_FIELDS));
        self.xml_toggle.set_label(texts::t(texts::VIEW_XML));
        for (button, text) in [
            (&self.recent_button, texts::RECENT_TITLE),
            (&self.menu_button, texts::MAIN_MENU),
        ] {
            button.set_tooltip_text(Some(texts::t(text)));
            button.update_property(&[gtk::accessible::Property::Label(texts::t(text))]);
        }
        self.menu_button.set_menu_model(Some(&main_menu()));
    }

    /// Shows everything in the language of the user interface, which has
    /// just changed (0019): the texts, and the open document again, keeping
    /// its PDF page and zoom and the chosen view.
    pub fn retranslate(&self) {
        self.apply_texts();
        self.xml_pane.apply_texts();
        self.pdf_view.apply_texts();
        let current = self.current.borrow().clone();
        if let Some(Shown { name, bytes }) = current {
            let xml_shown = self.xml_toggle.is_active();
            if self.show_content(&name, &bytes, false) && xml_shown {
                self.xml_toggle.set_active(true);
            }
        }
    }

    /// Stores the window size (0015).
    pub fn save_size(&self) {
        self.size.save();
    }

    /// The application window.
    pub fn widget(&self) -> &adw::ApplicationWindow {
        &self.window
    }

    /// Whether the window shows the start view, so that a file opened from
    /// outside can reuse it.
    pub fn is_empty(&self) -> bool {
        self.current.borrow().is_none()
    }

    /// Opens dropped files: the first one, with a note if there were more.
    /// A link dragged from a browser arrives as text; it is refused with the
    /// not-local message like any other address.
    fn add_drop_target(&self) {
        let target = gtk::DropTarget::new(glib::Type::INVALID, gtk::gdk::DragAction::COPY);
        target.set_types(&[gtk::gdk::FileList::static_type(), String::static_type()]);
        let this = self.clone();
        target.connect_drop(move |_, value, _, _| {
            let files = if let Ok(list) = value.get::<gtk::gdk::FileList>() {
                list.files()
            } else if let Ok(text) = value.get::<String>() {
                local::dropped_address(&text).into_iter().collect()
            } else {
                Vec::new()
            };
            let Some(first) = files.first().cloned() else {
                return false;
            };
            let more = files.len() > 1;
            let this = this.clone();
            glib::spawn_future_local(async move {
                this.open_file(&first).await;
                if more {
                    // Added to, not replacing, a message about the first file.
                    this.add_note(texts::t(texts::DROP_ONLY_FIRST));
                }
            });
            true
        });
        self.window.add_controller(target);
    }

    fn add_actions(&self) {
        let open = gio::SimpleAction::new("open", None);
        let this = self.clone();
        open.connect_activate(move |_, _| this.choose_file());
        self.window.add_action(&open);

        let close = gio::SimpleAction::new("close", None);
        let this = self.clone();
        close.connect_activate(move |_, _| {
            if !this.is_empty() {
                this.show_start(None);
            } else {
                this.window.close();
            }
        });
        self.window.add_action(&close);

        let find = gio::SimpleAction::new("find", None);
        let this = self.clone();
        find.connect_activate(move |_, _| {
            if this.xml_toggle.is_sensitive() {
                this.xml_toggle.set_active(true);
                this.xml_pane.start_search();
            }
        });
        self.window.add_action(&find);

        let open_recent = gio::SimpleAction::new("open-recent", Some(glib::VariantTy::STRING));
        let this = self.clone();
        open_recent.connect_activate(move |_, target| {
            let Some(path) = target.and_then(glib::Variant::get::<String>) else {
                return;
            };
            let this = this.clone();
            glib::spawn_future_local(async move { this.open_recent(Path::new(&path)).await });
        });
        self.window.add_action(&open_recent);

        let this = self.clone();
        self.status.connect_clicked(move || {
            this.fields_toggle.set_active(true);
            if let Some(view) = this.fields_view.borrow().as_ref() {
                view.scroll_to_summary();
            }
        });

        // "In Feldern zeigen" of the XML view (0014).
        let this = self.clone();
        self.xml_pane.connect_show_in_fields(move |target| {
            this.fields_toggle.set_active(true);
            let view = this.fields_view.clone();
            // After switching from the XML view, wait for the layout.
            glib::idle_add_local_once(move || {
                if let Some(view) = view.borrow().as_ref() {
                    view.jump_to(&target);
                }
            });
        });

        let this = self.clone();
        self.xml_toggle.connect_toggled(move |toggle| {
            if toggle.is_active() {
                this.xml_pane.ensure_loaded();
                this.view_stack.set_visible_child_name("xml");
            } else {
                this.view_stack.set_visible_child_name("fields");
            }
        });
    }

    fn choose_file(&self) {
        let filter = gtk::FileFilter::new();
        filter.set_name(Some(texts::t(texts::INVOICE_FILTER)));
        for mime_type in ["application/pdf", "application/xml", "text/xml"] {
            filter.add_mime_type(mime_type);
        }
        filter.add_suffix("pdf");
        filter.add_suffix("xml");
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let dialog = gtk::FileDialog::builder()
            .title(texts::t(texts::START_TITLE))
            .modal(true)
            .filters(&filters)
            .default_filter(&filter)
            .build();
        let this = self.clone();
        glib::spawn_future_local(async move {
            // Cancelling the dialog is not an error.
            if let Ok(file) = dialog.open_future(Some(&this.window)).await {
                this.open_file(&file).await;
            }
        });
    }

    /// Opens `file` in this window, replacing what is shown. Only local
    /// files are read; anything else is refused with a message.
    pub async fn open_file(&self, file: &gio::File) {
        match local::local_path(file) {
            Ok(path) => self.open_path(&path).await,
            Err(message) => self.notify(&message),
        }
    }

    /// Opens a recent entry; a file that no longer exists is removed from
    /// the list, and the window keeps showing what it showed.
    async fn open_recent(&self, path: &Path) {
        match self.read(path).await {
            Some(Err(error)) if error.matches(gio::IOErrorEnum::NotFound) => {
                self.notify(&texts::not_found_message(&file_name(path)));
                self.recent.remove(path);
            }
            Some(result) => self.show_read(path, result),
            None => {}
        }
    }

    async fn open_path(&self, path: &Path) {
        if let Some(result) = self.read(path).await {
            self.show_read(path, result);
        }
    }

    /// Reads a local file, never through a network backend. `None` if
    /// another file was opened meanwhile: the newer one wins.
    async fn read(&self, path: &Path) -> Option<Result<glib::Slice<u8>, glib::Error>> {
        let generation = self.next_generation();
        let result = gio::File::for_path(path).load_contents_future().await;
        (self.generation.get() == generation).then(|| result.map(|(bytes, _)| bytes))
    }

    /// Shows what was read; an unreadable file closes the shown invoice, as
    /// an invalid one does (0007).
    fn show_read(&self, path: &Path, result: Result<glib::Slice<u8>, glib::Error>) {
        let name = file_name(path);
        match result {
            Ok(bytes) => {
                if self.show(&name, &bytes) {
                    self.recent.add(path);
                }
            }
            Err(error) => {
                self.show_start(Some(&texts::read_failed_message(&name, error.message())));
            }
        }
    }

    /// The section "Anhänge" of the opened document (0013).
    fn attachments_group(&self, document: &Document) -> adw::PreferencesGroup {
        let this = self.clone();
        attachments_view::group(document.attachments(), move |attachment, position| {
            this.save_attachment(attachment, position);
        })
    }

    /// Saves an attachment where the user chooses; nothing is written
    /// otherwise.
    fn save_attachment(&self, attachment: &Attachment, position: usize) {
        let Content::Embedded(content) = &attachment.content else {
            return;
        };
        let content = glib::Bytes::from(content);
        let name = attachment.display_name(position);
        let initial_name = attachment.proposed_file_name(position);
        let this = self.clone();
        glib::spawn_future_local(async move {
            let parent = this.window.clone().upcast::<gtk::Window>();
            let result = save_as(
                Some(&parent),
                texts::t(texts::SAVE_ATTACHMENT),
                &initial_name,
                content,
            )
            .await;
            if let Err(error) = result {
                this.notify(&texts::save_failed_message(&name, error.message()));
            }
        });
    }

    /// Adds a note to the message of the start view, or shows it while an
    /// invoice stays shown.
    fn add_note(&self, message: &str) {
        match self.start_page.description().filter(|_| self.is_empty()) {
            Some(shown) if !shown.is_empty() => {
                self.start_page
                    .set_description(Some(&format!("{shown}\n\n{message}")));
            }
            _ => self.notify(message),
        }
    }

    /// Shows a message: in the start view, or as a note while an invoice
    /// stays shown.
    fn notify(&self, message: &str) {
        if self.is_empty() {
            self.show_start(Some(message));
        } else {
            self.toasts
                .add_toast(adw::Toast::builder().title(message).timeout(5).build());
        }
    }

    /// Shows the document in `bytes`; returns whether it could be opened.
    fn show(&self, name: &str, bytes: &[u8]) -> bool {
        let shown = self.show_content(name, bytes, true);
        if shown {
            *self.current.borrow_mut() = Some(Shown {
                name: name.to_string(),
                bytes: Rc::from(bytes),
            });
        }
        shown
    }

    /// Opens and shows a document; the PDF pages are loaded only with
    /// `load_pdf`, so that showing it again keeps the page and zoom.
    fn show_content(&self, name: &str, bytes: &[u8], load_pdf: bool) -> bool {
        // Whatever is shown next, a running validation is outdated.
        self.next_generation();
        let document = match document::open(bytes, texts::language()) {
            Ok(document) => document,
            Err(error) => {
                self.show_start(Some(&texts::error_message(name, &error)));
                return false;
            }
        };
        let attachments = self.attachments_group(&document);
        match document {
            Document::Pdf { invoice, .. } => {
                if load_pdf {
                    self.show_pdf_pane(Some(bytes));
                }
                match invoice {
                    Some(invoice) => self.show_invoice(invoice, true, name, None, &attachments),
                    None => {
                        // The PDF still shows what it embeds instead.
                        let content = gtk::Box::builder()
                            .orientation(gtk::Orientation::Vertical)
                            .spacing(18)
                            .margin_start(12)
                            .margin_end(12)
                            .margin_bottom(24)
                            .build();
                        content.append(&message_page(texts::t(texts::NO_INVOICE_IN_PDF)));
                        content.append(&attachments);
                        let scroller = gtk::ScrolledWindow::builder()
                            .hscrollbar_policy(gtk::PolicyType::Never)
                            .vexpand(true)
                            .child(&content)
                            .build();
                        self.set_fields(scroller.upcast(), None);
                        self.set_xml(None);
                    }
                }
                self.show_document(name);
            }
            Document::Xml { invoice } => {
                self.show_pdf_pane(None);
                self.show_invoice(
                    invoice,
                    false,
                    name,
                    Some(texts::t(texts::XML_ONLY_NOTE)),
                    &attachments,
                );
                self.show_document(name);
            }
        }
        true
    }

    /// Shows the fields and XML of an invoice and starts validating it;
    /// `embedded` if it comes from a PDF.
    fn show_invoice(
        &self,
        invoice: Invoice,
        embedded: bool,
        name: &str,
        note: Option<&str>,
        attachments: &adw::PreferencesGroup,
    ) {
        let Invoice { xml, fields, .. } = invoice;
        self.set_xml(Some((&xml, name, fields.clone())));
        let this = self.clone();
        let show_in_xml: ShowInXml = Rc::new(move |location, attribute| {
            this.xml_toggle.set_active(true);
            this.xml_pane.reveal(location, attribute);
        });
        let view = FieldsView::new(fields, attachments, show_in_xml);
        self.set_fields(view.widget(), note);
        *self.fields_view.borrow_mut() = Some(view);
        self.start_validation(xml, embedded);
    }

    /// Validates on a worker thread, so that the window stays responsive,
    /// and shows the result unless another document was opened meanwhile.
    fn start_validation(&self, xml: Vec<u8>, embedded: bool) {
        let generation = self.generation.get();
        self.set_validation_state(&ValidationState::Running);
        let this = self.clone();
        glib::spawn_future_local(async move {
            // The worker thread has its own language setting: pass it on.
            let language = texts::language();
            let result = gio::spawn_blocking(move || validate(&xml, embedded, language)).await;
            if this.generation.get() != generation {
                return;
            }
            let state = match result {
                Ok(Ok(validation)) => ValidationState::Done(validation),
                Ok(Err(error)) => ValidationState::Failed(texts::validation_error_reason(&error)),
                // The worker panicked.
                Err(_) => {
                    ValidationState::Failed(texts::t(texts::VALIDATION_FAILED_REASON).to_string())
                }
            };
            this.set_validation_state(&state);
        });
    }

    fn set_validation_state(&self, state: &ValidationState) {
        self.status.show(state);
        if let Some(view) = self.fields_view.borrow().as_ref() {
            view.set_state(state);
        }
    }

    /// Shows the first page of `pdf`, or hides the PDF pane for `None`.
    fn show_pdf_pane(&self, pdf: Option<&[u8]>) {
        let Some(pdf) = pdf else {
            self.pdf_pane.set_visible(false);
            return;
        };
        self.pdf_pane.set_visible(true);
        let shown = self.pdf_view.load(pdf);
        self.pdf_pane
            .set_visible_child_name(if shown { "page" } else { "message" });
    }

    fn set_fields(&self, content: gtk::Widget, note: Option<&str>) {
        *self.fields_view.borrow_mut() = None;
        self.status.hide();
        for container in [&self.note_slot, &self.fields_container] {
            while let Some(child) = container.first_child() {
                container.remove(&child);
            }
        }
        if let Some(note) = note {
            let banner = adw::Banner::builder().title(note).revealed(true).build();
            self.note_slot.append(&banner);
        }
        self.fields_container.append(&content);
        self.fields_toggle.set_active(true);
    }

    /// Sets the invoice XML for the XML view; `None` disables the view.
    fn set_xml(&self, xml: Option<(&[u8], &str, FieldModel)>) {
        let shown = xml.is_some();
        match xml {
            Some((bytes, file_name, model)) => self.xml_pane.set_xml(bytes, file_name, model),
            None => self.xml_pane.clear(),
        }
        self.xml_toggle.set_sensitive(shown);
    }

    fn show_document(&self, name: &str) {
        let title = texts::window_title(Some(name));
        self.window.set_title(Some(&title));
        self.title.set_title(name);
        self.title.set_subtitle(texts::APP_TITLE);
        self.close_button.set_visible(true);
        if self.paned.position() <= 0 {
            self.paned.set_position(self.window.width() / 2);
        }
        self.stack.set_visible_child_name("document");
    }

    /// Starts a new generation: results of older reads and validations are
    /// discarded.
    fn next_generation(&self) -> u64 {
        let generation = self.generation.get() + 1;
        self.generation.set(generation);
        generation
    }

    /// Shows the start view, optionally with an error message.
    fn show_start(&self, message: Option<&str>) {
        *self.current.borrow_mut() = None;
        self.next_generation();
        self.window.set_title(Some(&texts::window_title(None)));
        self.title.set_title(texts::APP_TITLE);
        self.title.set_subtitle("");
        self.close_button.set_visible(false);
        self.start_page.set_description(message);
        self.pdf_view.clear();
        self.set_fields(gtk::Box::new(gtk::Orientation::Vertical, 0).upcast(), None);
        self.set_xml(None);
        self.stack.set_visible_child_name("start");
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

fn message_page(title: &str) -> gtk::Widget {
    adw::StatusPage::builder()
        .icon_name("dialog-information-symbolic")
        .title(title)
        .vexpand(true)
        .css_classes(["compact"])
        .build()
        .upcast()
}

/// The main menu: the language choice (0019), with each language named in
/// itself.
fn main_menu() -> gio::Menu {
    let languages = gio::Menu::new();
    for choice in Choice::ALL {
        let item = gio::MenuItem::new(Some(choice.label()), None);
        item.set_action_and_target_value(Some("app.language"), Some(&choice.code().to_variant()));
        languages.append_item(&item);
    }
    let menu = gio::Menu::new();
    menu.append_section(Some(texts::t(texts::LANGUAGE_MENU)), &languages);
    menu
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Interactive widgets: something a user activates or types into.
    fn is_interactive(widget: &gtk::Widget) -> bool {
        widget.is::<gtk::Button>()
            || widget.is::<gtk::MenuButton>()
            || widget.is::<gtk::ToggleButton>()
            || widget.is::<gtk::CheckButton>()
            || widget.is::<gtk::Switch>()
            || widget.is::<gtk::Entry>()
            || widget.is::<gtk::SearchEntry>()
            || widget.is::<gtk::DropDown>()
            || widget
                .downcast_ref::<gtk::ListBoxRow>()
                .is_some_and(gtk::ListBoxRow::is_activatable)
    }

    /// The text a user sees or hears for a widget: its own label, a label
    /// inside it, or its tooltip (which the app always sets together with
    /// the accessible name, e.g. `pdf_pane::apply_texts`).
    fn name_of(widget: &gtk::Widget) -> Option<String> {
        if let Some(button) = widget.downcast_ref::<gtk::Button>()
            && let Some(label) = button.label()
        {
            return Some(label.to_string());
        }
        if let Some(button) = widget.downcast_ref::<gtk::MenuButton>()
            && let Some(label) = button.label()
        {
            return Some(label.to_string());
        }
        if let Some(tooltip) = widget.tooltip_text() {
            return Some(tooltip.to_string());
        }
        let mut child = widget.first_child();
        while let Some(widget) = child {
            if let Some(label) = widget.downcast_ref::<gtk::Label>() {
                return Some(label.text().to_string());
            }
            if let Some(name) = name_of(&widget).filter(|_| !is_interactive(&widget)) {
                return Some(name);
            }
            child = widget.next_sibling();
        }
        None
    }

    /// Inner parts of GTK's own composite widgets, which GTK names for
    /// assistive technology itself (not readable through the GTK API, so
    /// they are checked through AT-SPI instead): the window controls, the
    /// close button of a search bar, the toggle inside a menu button.
    fn is_gtk_internal(widget: &gtk::Widget) -> bool {
        let parent_is = |name: &str| {
            std::iter::successors(widget.parent(), gtk::Widget::parent)
                .take(4)
                .any(|ancestor| ancestor.type_().name() == name)
        };
        parent_is("GtkWindowControls")
            || (widget.is::<gtk::Button>() && parent_is("GtkSearchBar"))
            || (widget.is::<gtk::ToggleButton>()
                && widget.parent().is_some_and(|p| p.is::<gtk::MenuButton>()))
    }

    fn problems(widget: &gtk::Widget, path: &str, found: &mut Vec<String>) {
        let path = format!("{path}/{}", widget.type_().name());
        if widget.is_visible() && is_interactive(widget) && !is_gtk_internal(widget) {
            if name_of(widget).is_none_or(|name| name.trim().is_empty()) {
                found.push(format!("{path}: no name"));
            }
            if !widget.is_focusable() && !widget.can_focus() {
                found.push(format!("{path}: not reachable by keyboard"));
            }
        }
        let mut child = widget.first_child();
        while let Some(next) = child {
            problems(&next, &path, found);
            child = next.next_sibling();
        }
    }

    // Spec 0028 AC1: needs a display; runs when EIV_GUI_TESTS is set (the CI
    // Linux job under a virtual display, or locally with a separate one).
    #[test]
    fn every_control_is_named_and_focusable() {
        if std::env::var_os("EIV_GUI_TESTS").is_none() {
            eprintln!("skipped: set EIV_GUI_TESTS and provide a display");
            return;
        }
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../core/tests/fixtures");
        let invoice = std::fs::read(fixtures.join("cii/sample-c.xml")).expect("sample C");
        let found = Rc::new(RefCell::new(None::<Vec<String>>));
        // A separate, non-unique id: never talks to a running viewer.
        let app = adw::Application::builder()
            .application_id("de.loges.EInvoiceViewer.GuiTest")
            .flags(gio::ApplicationFlags::NON_UNIQUE)
            .build();
        let result = found.clone();
        app.connect_activate(move |app| {
            let dir = std::env::temp_dir().join(format!("eiv-gui-test-{}", std::process::id()));
            let recent = RecentStore::with_path(dir.join("recent"));
            let window = Window::new(app, &recent);
            window.present();
            window.show_content("sample-c.xml", &invoice, true);
            let app = app.clone();
            let result = result.clone();
            glib::timeout_add_local_once(std::time::Duration::from_secs(2), move || {
                let mut found = Vec::new();
                problems(window.widget().upcast_ref(), "", &mut found);
                *result.borrow_mut() = Some(found);
                app.quit();
            });
        });
        app.run_with_args::<&str>(&[]);
        let found = found.borrow().clone().expect("the window was checked");
        assert!(found.is_empty(), "{found:#?}");
    }
}
