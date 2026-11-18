//! GTK4 / libadwaita application of E-Invoice Viewer (presentation only).

mod attachments_view;
mod config_store;
mod fields_view;
mod language_setting;
mod local;
mod pdf_layout;
mod pdf_pane;
mod pdf_view;
mod recent;
mod save_as;
mod status_line;
mod texts;
mod window;
mod window_size;
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

    let windows = Windows::default();
    let list = windows.clone();
    // Shared by all windows.
    let recent = RecentStore::new();
    let recent_store = recent.clone();
    app.connect_startup(move |app| {
        // The language chosen in the app, or the system's (0018, 0019).
        let (choice, warning) = language_setting::load(&language_setting::store_path());
        if let Some(warning) = warning {
            glib::g_warning!(config_store::LOG_DOMAIN, "{warning}");
        }
        texts::set_language(choice.effective(&glib::language_names()));
        app.add_action(&language_action(choice, &list, &recent_store));
        load_css();
        // The app's own icon (data/icons), installed by scripts/install.sh
        // and shipped in the AppImage (0020).
        gtk::Window::set_default_icon_name(APP_ID);
        let quit = gio::SimpleAction::new("quit", None);
        let target = app.clone();
        let list = list.clone();
        quit.connect_activate(move |_, _| {
            // Quitting closes no window, so the sizes are saved here; the
            // active window is saved last and wins (0015).
            list.save_sizes(target.active_window().as_ref());
            target.quit();
        });
        app.add_action(&quit);
        app.set_accels_for_action("app.quit", &["<Control>q"]);
        app.set_accels_for_action("win.open", &["<Control>o"]);
        app.set_accels_for_action("win.close", &["<Control>w"]);
        app.set_accels_for_action("win.find", &["<Control>f"]);
    });

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

    /// Saves the size of every window, `last` at the end.
    fn save_sizes(&self, last: Option<&gtk::Window>) {
        let windows = self.0.borrow().clone();
        let is_last = |window: &Window| {
            last.is_some_and(|last| window.widget().upcast_ref::<gtk::Window>() == last)
        };
        for window in windows.iter().filter(|window| !is_last(window)) {
            window.save_size();
        }
        for window in windows.iter().filter(|window| is_last(window)) {
            window.save_size();
        }
    }

    /// Shows every window in the language that has just been chosen.
    fn retranslate(&self) {
        let windows = self.0.borrow().clone();
        for window in windows {
            window.retranslate();
        }
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

/// The action `app.language` of the language menu (0019): its state is the
/// stored choice; changing it stores the new choice and shows every window
/// in the new language at once.
fn language_action(
    choice: language_setting::Choice,
    windows: &Windows,
    recent: &Rc<RecentStore>,
) -> gio::SimpleAction {
    let action = gio::SimpleAction::new_stateful(
        "language",
        Some(glib::VariantTy::STRING),
        &choice.code().to_variant(),
    );
    let (windows, recent) = (windows.clone(), recent.clone());
    action.connect_change_state(move |action, value| {
        let Some(choice) = value
            .and_then(glib::Variant::get::<String>)
            .and_then(|code| language_setting::Choice::from_code(&code))
        else {
            return;
        };
        action.set_state(&choice.code().to_variant());
        if let Err(error) = language_setting::save(&language_setting::store_path(), choice) {
            glib::g_warning!(
                config_store::LOG_DOMAIN,
                "language setting not saved: {error}"
            );
        }
        texts::set_language(choice.effective(&glib::language_names()));
        recent.retranslate();
        windows.retranslate();
    });
    action
}

/// Styles that libadwaita has no class for.
fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&format!(
        ".{} {{ background-color: alpha(@accent_bg_color, 0.25); }}\n\
         .{} {{ background-color: white; box-shadow: 0 1px 4px alpha(black, 0.3); }}",
        fields_view::HIGHLIGHT_CLASS,
        pdf_pane::PAGE_CLASS,
    ));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
