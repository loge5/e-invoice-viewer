//! Settings files in the app's configuration directory (specs 0012, 0015):
//! GLib key files, written atomically.

use std::path::{Path, PathBuf};

use gtk::glib;

/// Log domain of warnings.
pub const LOG_DOMAIN: &str = "e-invoice-viewer";

/// Where the settings file `name` is stored.
pub fn path(name: &str) -> PathBuf {
    glib::user_config_dir().join("e-invoice-viewer").join(name)
}

/// Reads a settings file: `None` if there is none, an error with the reason
/// if it exists but cannot be used.
pub fn read(path: &Path) -> Result<Option<glib::KeyFile>, String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let text = String::from_utf8(bytes).map_err(|_| "not UTF-8".to_string())?;
    // GLib cannot take text with NUL characters.
    if text.contains('\0') {
        return Err("contains NUL characters".to_string());
    }
    let key_file = glib::KeyFile::new();
    key_file
        .load_from_data(&text, glib::KeyFileFlags::NONE)
        .map_err(|error| error.message().to_string())?;
    Ok(Some(key_file))
}

/// Writes a settings file. It is written to a temporary file and renamed,
/// so that an interrupted save keeps the previous file.
pub fn write(path: &Path, key_file: &glib::KeyFile) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    }
    glib::file_set_contents(path, key_file.to_data().as_bytes())
        .map_err(|error| error.message().to_string())
}
