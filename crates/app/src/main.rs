//! GTK4 / libadwaita application of E-Invoice Viewer (presentation only).

mod fields_view;
mod pdf_view;
mod texts;
mod window;

use adw::prelude::*;
use gtk::{gio, glib};

use window::Window;

const APP_ID: &str = "de.loges.EInvoiceViewer";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_startup(|app| {
        let quit = gio::SimpleAction::new("quit", None);
        let target = app.clone();
        quit.connect_activate(move |_, _| target.quit());
        app.add_action(&quit);
        app.set_accels_for_action("app.quit", &["<Control>q"]);
        app.set_accels_for_action("win.open", &["<Control>o"]);
        app.set_accels_for_action("win.close", &["<Control>w"]);
    });

    // Started without files: an empty start view; nothing is reopened.
    app.connect_activate(|app| Window::new(app).present());

    // Started with files (command line): one window per file.
    app.connect_open(|app, files, _| {
        for file in files {
            let window = Window::new(app);
            window.present();
            let file = file.clone();
            glib::spawn_future_local(async move { window.open_file(&file).await });
        }
    });

    app.run()
}
