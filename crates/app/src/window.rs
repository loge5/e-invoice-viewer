//! The main window: start view, or PDF and fields side by side (spec 0007).

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;

use adw::prelude::*;
use e_invoice_viewer_core::attachments::{Attachment, Content};
use e_invoice_viewer_core::detect::DetectError;
use e_invoice_viewer_core::document::{self, Document, Invoice};
use e_invoice_viewer_core::fields::FieldModel;
use e_invoice_viewer_core::validate::{Validation, Validator};
use gtk::{gio, glib};

use crate::fields_view::{FieldsView, ShowInXml};
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

/// Validates on the calling (worker) thread.
fn validate(xml: &[u8]) -> Result<Validation, DetectError> {
    VALIDATOR.with(|validator| validator.validate(xml))
}

const MINIMUM_PANE_WIDTH: i32 = 300;

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
    /// Whether a document is shown (not the start view).
    showing: Rc<Cell<bool>>,
    toasts: adw::ToastOverlay,
    /// Remembers the window size (0015).
    size: SizeKeeper,
    /// Recently opened files, shared by all windows.
    recent: Rc<RecentStore>,
}

impl Window {
    pub fn new(app: &adw::Application, recent: &Rc<RecentStore>) -> Self {
        let title = adw::WindowTitle::new(texts::APP_TITLE, "");
        let open_button = gtk::Button::builder()
            .label(texts::OPEN_BUTTON)
            .action_name("win.open")
            .build();
        let close_button = gtk::Button::builder()
            .label(texts::CLOSE_BUTTON)
            .action_name("win.close")
            .visible(false)
            .build();
        let recent_button = gtk::MenuButton::builder()
            .icon_name("document-open-recent-symbolic")
            .tooltip_text(texts::RECENT_TITLE)
            .menu_model(recent.menu())
            .build();
        recent_button.update_property(&[gtk::accessible::Property::Label(texts::RECENT_TITLE)]);
        let header = adw::HeaderBar::builder().title_widget(&title).build();
        header.pack_start(&open_button);
        header.pack_start(&recent_button);
        header.pack_end(&close_button);

        let start_button = gtk::Button::builder()
            .label(texts::OPEN_BUTTON)
            .action_name("win.open")
            .halign(gtk::Align::Center)
            .css_classes(["pill", "suggested-action"])
            .build();
        let start_page = adw::StatusPage::builder()
            .icon_name("x-office-document-symbolic")
            .title(texts::START_TITLE)
            .child(&start_button)
            .build();

        let pdf_view = PdfPane::new();
        let pdf_message = adw::StatusPage::builder()
            .icon_name("image-missing-symbolic")
            .title(texts::PDF_NOT_RENDERABLE)
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

        let fields_toggle = gtk::ToggleButton::builder()
            .label(texts::VIEW_FIELDS)
            .active(true)
            .build();
        let xml_toggle = gtk::ToggleButton::builder()
            .label(texts::VIEW_XML)
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
            showing: Rc::default(),
            toasts,
            size: SizeKeeper::attach(&window, size),
            recent: recent.clone(),
        };
        this.add_actions();
        this.add_drop_target();
        this
    }

    pub fn present(&self) {
        self.window.present();
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
        !self.showing.get()
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
                    this.add_note(texts::DROP_ONLY_FIRST);
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
            if this.showing.get() {
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
        filter.set_name(Some(texts::INVOICE_FILTER));
        for mime_type in ["application/pdf", "application/xml", "text/xml"] {
            filter.add_mime_type(mime_type);
        }
        filter.add_suffix("pdf");
        filter.add_suffix("xml");
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let dialog = gtk::FileDialog::builder()
            .title(texts::START_TITLE)
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
                texts::SAVE_ATTACHMENT,
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
        // Whatever is shown next, a running validation is outdated.
        self.next_generation();
        let document = match document::open(bytes) {
            Ok(document) => document,
            Err(error) => {
                self.show_start(Some(&texts::error_message(name, &error)));
                return false;
            }
        };
        let attachments = self.attachments_group(&document);
        match document {
            Document::Pdf { invoice, .. } => {
                self.show_pdf_pane(Some(bytes));
                match invoice {
                    Some(invoice) => self.show_invoice(invoice, name, None, &attachments),
                    None => {
                        // The PDF still shows what it embeds instead.
                        let content = gtk::Box::builder()
                            .orientation(gtk::Orientation::Vertical)
                            .spacing(18)
                            .margin_start(12)
                            .margin_end(12)
                            .margin_bottom(24)
                            .build();
                        content.append(&message_page(texts::NO_INVOICE_IN_PDF));
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
                self.show_invoice(invoice, name, Some(texts::XML_ONLY_NOTE), &attachments);
                self.show_document(name);
            }
        }
        true
    }

    /// Shows the fields and XML of an invoice and starts validating it.
    fn show_invoice(
        &self,
        invoice: Invoice,
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
        self.start_validation(xml);
    }

    /// Validates on a worker thread, so that the window stays responsive,
    /// and shows the result unless another document was opened meanwhile.
    fn start_validation(&self, xml: Vec<u8>) {
        let generation = self.generation.get();
        self.set_validation_state(&ValidationState::Running);
        let this = self.clone();
        glib::spawn_future_local(async move {
            let result = gio::spawn_blocking(move || validate(&xml)).await;
            if this.generation.get() != generation {
                return;
            }
            let state = match result {
                Ok(Ok(validation)) => ValidationState::Done(validation),
                Ok(Err(error)) => ValidationState::Failed(texts::validation_error_reason(&error)),
                // The worker panicked.
                Err(_) => ValidationState::Failed(texts::VALIDATION_FAILED_REASON.to_string()),
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
        self.showing.set(true);
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
        self.showing.set(false);
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
