//! Saving a copy where the user chooses (specs 0008, 0013). Nothing is
//! written next to the opened invoice unless the user picks that place.

use adw::prelude::*;
use gtk::{gio, glib};

/// Asks where to save `bytes`, proposing `initial_name`, and writes them
/// there. Cancelling the dialog is not an error. The file is written to a
/// temporary file and renamed, so that a failure leaves no partial file.
pub async fn save_as(
    parent: Option<&gtk::Window>,
    title: &str,
    initial_name: &str,
    bytes: glib::Bytes,
) -> Result<(), glib::Error> {
    let dialog = gtk::FileDialog::builder()
        .title(title)
        .modal(true)
        .initial_name(initial_name)
        .build();
    let Ok(file) = dialog.save_future(parent).await else {
        return Ok(());
    };
    file.replace_contents_future(bytes, None, false, gio::FileCreateFlags::NONE)
        .await
        .map(|_| ())
        .map_err(|(_, error)| error)
}
