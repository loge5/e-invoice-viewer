//! GTK4 / libadwaita application of E-Invoice Viewer (presentation only).

use adw::prelude::*;

const APP_ID: &str = "de.loges.EInvoiceViewer";

fn main() -> gtk::glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        adw::ApplicationWindow::builder()
            .application(app)
            .title("E-Invoice Viewer")
            .build()
            .present();
    });
    app.run()
}
