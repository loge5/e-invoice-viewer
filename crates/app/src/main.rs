//! GTK4 / libadwaita application of E-Invoice Viewer (presentation only).

mod attachments_view;
mod fields_view;
mod local;
mod pdf_view;
mod recent;
mod texts;
mod window;
mod xml_pane;

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};

use recent::RecentStore;
use window::Window;

const APP_ID: &str = "de.loges.EInvoiceViewer";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_startup(|app| {
        load_css();
        let quit = gio::SimpleAction::new("quit", None);
        let target = app.clone();
        quit.connect_activate(move |_, _| target.quit());
        app.add_action(&quit);
        app.set_accels_for_action("app.quit", &["<Control>q"]);
        app.set_accels_for_action("win.open", &["<Control>o"]);
        app.set_accels_for_action("win.close", &["<Control>w"]);
        app.set_accels_for_action("win.find", &["<Control>f"]);
    });

    // Shared by all windows; read on first use.
    let recent = RecentStore::new();
    let windows = Windows::default();

    // Started without files: an empty start view; nothing is reopened.
    let (shared, list) = (recent.clone(), windows.clone());
    app.connect_activate(move |app| list.create(app, &shared).present());

    // Files from outside (command line, file manager): one window per file.
    // An empty window showing the start view is reused for the first one.
    app.connect_open(move |app, files, _| {
        let mut reusable = windows.empty();
        for file in files {
            let window = reusable
                .take()
                .unwrap_or_else(|| windows.create(app, &recent));
            window.present();
            let file = file.clone();
            glib::spawn_future_local(async move { window.open_file(&file).await });
        }
    });

    app.run()
}

/// The open windows of the application.
#[derive(Clone, Default)]
struct Windows(Rc<RefCell<Vec<Window>>>);

impl Windows {
    fn create(&self, app: &adw::Application, recent: &Rc<RecentStore>) -> Window {
        let window = Window::new(app, recent);
        let list = self.0.clone();
        let closed = window.widget().clone();
        window.widget().connect_destroy(move |_| {
            list.borrow_mut()
                .retain(|window| window.widget() != &closed);
        });
        self.0.borrow_mut().push(window.clone());
        window
    }

    /// An open window that shows the start view.
    fn empty(&self) -> Option<Window> {
        self.0
            .borrow()
            .iter()
            .find(|window| window.is_empty())
            .cloned()
    }
}

/// Styles that libadwaita has no class for.
fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&format!(
        ".{} {{ background-color: alpha(@accent_bg_color, 0.25); }}",
        fields_view::HIGHLIGHT_CLASS
    ));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
