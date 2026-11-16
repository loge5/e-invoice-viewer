//! The main window: start view, or PDF and fields side by side (spec 0007).

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use e_invoice_viewer_core::document::{self, Document};
use gtk::{gio, glib};

use crate::xml_pane::XmlPane;
use crate::{fields_view, pdf_view, texts};

const DEFAULT_SIZE: (i32, i32) = (1200, 800);
const MINIMUM_SIZE: (i32, i32) = (900, 600);
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
    pdf_picture: gtk::Picture,
    note_slot: gtk::Box,
    fields_container: gtk::Box,
    view_stack: gtk::Stack,
    fields_toggle: gtk::ToggleButton,
    xml_toggle: gtk::ToggleButton,
    xml_pane: XmlPane,
    paned: gtk::Paned,
    /// File name of the shown invoice, `None` in the start view.
    current: Rc<RefCell<Option<String>>>,
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
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
        let header = adw::HeaderBar::builder().title_widget(&title).build();
        header.pack_start(&open_button);
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

        let pdf_picture = gtk::Picture::builder()
            .can_shrink(true)
            .content_fit(gtk::ContentFit::Contain)
            .valign(gtk::Align::Start)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        let pdf_scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&pdf_picture)
            .build();
        let pdf_message = adw::StatusPage::builder()
            .icon_name("image-missing-symbolic")
            .title(texts::PDF_NOT_RENDERABLE)
            .css_classes(["compact"])
            .build();
        let pdf_pane = gtk::Stack::builder()
            .width_request(MINIMUM_PANE_WIDTH)
            .hexpand(true)
            .build();
        pdf_pane.add_named(&pdf_scroller, Some("page"));
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

        let fields_pane = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .width_request(MINIMUM_PANE_WIDTH)
            .hexpand(true)
            .build();
        fields_pane.append(&note_slot);
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
            .position(DEFAULT_SIZE.0 / 2)
            .build();

        let stack = gtk::Stack::new();
        stack.add_named(&start_page, Some("start"));
        stack.add_named(&paned, Some("document"));

        let content = adw::ToolbarView::new();
        content.add_top_bar(&header);
        content.set_content(Some(&stack));

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title(texts::APP_TITLE)
            .default_width(DEFAULT_SIZE.0)
            .default_height(DEFAULT_SIZE.1)
            .width_request(MINIMUM_SIZE.0)
            .height_request(MINIMUM_SIZE.1)
            .content(&content)
            .build();

        let this = Self {
            window,
            title,
            close_button,
            stack,
            start_page,
            pdf_pane,
            pdf_picture,
            note_slot,
            fields_container,
            view_stack,
            fields_toggle,
            xml_toggle,
            xml_pane,
            paned,
            current: Rc::default(),
        };
        this.add_actions();
        this
    }

    pub fn present(&self) {
        self.window.present();
    }

    fn add_actions(&self) {
        let open = gio::SimpleAction::new("open", None);
        let this = self.clone();
        open.connect_activate(move |_, _| this.choose_file());
        self.window.add_action(&open);

        let close = gio::SimpleAction::new("close", None);
        let this = self.clone();
        close.connect_activate(move |_, _| {
            if this.current.borrow().is_some() {
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
        filter.set_name(Some("Rechnungen (PDF, XML)"));
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

    /// Opens `file` in this window, replacing what is shown.
    pub async fn open_file(&self, file: &gio::File) {
        let name = file
            .basename()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| file.uri().to_string());
        match file.load_contents_future().await {
            Ok((bytes, _)) => self.show(&name, &bytes),
            Err(error) => self.show_start(Some(&format!(
                "Die Datei {name} kann nicht gelesen werden: {}",
                error.message()
            ))),
        }
    }

    fn show(&self, name: &str, bytes: &[u8]) {
        match document::open(bytes) {
            Ok(Document::Pdf { invoice }) => {
                self.show_pdf_pane(Some(bytes));
                match invoice {
                    Some(invoice) => {
                        self.set_fields(fields_view::build(&invoice.fields), None);
                        self.set_xml(Some((&invoice.xml, name)));
                    }
                    None => {
                        self.set_fields(message_page(texts::NO_INVOICE_IN_PDF), None);
                        self.set_xml(None);
                    }
                }
                self.show_document(name);
            }
            Ok(Document::Xml { invoice }) => {
                self.show_pdf_pane(None);
                self.set_fields(
                    fields_view::build(&invoice.fields),
                    Some(texts::XML_ONLY_NOTE),
                );
                self.set_xml(Some((&invoice.xml, name)));
                self.show_document(name);
            }
            Err(error) => self.show_start(Some(&texts::error_message(name, &error))),
        }
    }

    /// Shows the first page of `pdf`, or hides the PDF pane for `None`.
    fn show_pdf_pane(&self, pdf: Option<&[u8]>) {
        let Some(pdf) = pdf else {
            self.pdf_pane.set_visible(false);
            return;
        };
        self.pdf_pane.set_visible(true);
        match pdf_view::render_first_page(pdf) {
            Some(texture) => {
                self.pdf_picture.set_paintable(Some(&texture));
                self.pdf_pane.set_visible_child_name("page");
            }
            None => {
                self.pdf_picture.set_paintable(None::<&gtk::gdk::Paintable>);
                self.pdf_pane.set_visible_child_name("message");
            }
        }
    }

    fn set_fields(&self, content: gtk::Widget, note: Option<&str>) {
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
    fn set_xml(&self, xml: Option<(&[u8], &str)>) {
        match xml {
            Some((bytes, file_name)) => self.xml_pane.set_xml(bytes, file_name),
            None => self.xml_pane.clear(),
        }
        self.xml_toggle.set_sensitive(xml.is_some());
    }

    fn show_document(&self, name: &str) {
        *self.current.borrow_mut() = Some(name.to_string());
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

    /// Shows the start view, optionally with an error message.
    fn show_start(&self, message: Option<&str>) {
        *self.current.borrow_mut() = None;
        self.window.set_title(Some(&texts::window_title(None)));
        self.title.set_title(texts::APP_TITLE);
        self.title.set_subtitle("");
        self.close_button.set_visible(false);
        self.start_page.set_description(message);
        self.pdf_picture.set_paintable(None::<&gtk::gdk::Paintable>);
        self.set_fields(gtk::Box::new(gtk::Orientation::Vertical, 0).upcast(), None);
        self.set_xml(None);
        self.stack.set_visible_child_name("start");
    }
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
