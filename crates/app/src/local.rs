//! Only local files are read (spec 0012): the app never fetches content
//! from the network, whichever way an address reaches it.

use std::path::PathBuf;

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::texts;

/// The local path of `file`, or the message shown when it is not a local
/// file.
pub fn local_path(file: &gio::File) -> Result<PathBuf, String> {
    // Web addresses are refused before anything else is asked about them:
    // even resolving them could reach the network.
    let refused = WEB_SCHEMES.iter().any(|scheme| file.has_uri_scheme(scheme));
    let path = if refused { None } else { file.path() };
    path.ok_or_else(|| texts::not_local_message(&file.uri()))
}

/// The file a dropped text names, if it is an address (e.g. a link dragged
/// from a browser, which offers it as text only).
pub fn dropped_address(text: &str) -> Option<gio::File> {
    let first = text.lines().next()?.trim();
    glib::Uri::peek_scheme(first).map(|_| gio::File::for_uri(first))
}

/// Schemes whose content would be fetched from the network. Files on network
/// shares the user mounted (smb, sftp, …) have a local path and are allowed.
const WEB_SCHEMES: [&str; 6] = ["http", "https", "ftp", "dav", "davs", "webdav"];

#[cfg(test)]
mod tests {
    use super::*;

    // AC4
    #[test]
    fn web_addresses_are_refused() {
        for uri in [
            "https://example.com/RE-2026-0042.pdf",
            "http://example.com/RE-2026-0042.pdf",
            "ftp://example.com/RE-2026-0042.pdf",
        ] {
            assert_eq!(
                local_path(&gio::File::for_uri(uri)),
                Err(format!("Die Adresse {uri} ist keine lokale Datei")),
            );
        }
    }

    // AC4
    #[test]
    fn local_files_are_accepted() {
        let file = gio::File::for_path("/tmp/x/RE-2026-0042.pdf");
        assert_eq!(
            local_path(&file),
            Ok(PathBuf::from("/tmp/x/RE-2026-0042.pdf"))
        );
        let file = gio::File::for_uri("file:///tmp/x/RE%202026.pdf");
        assert_eq!(local_path(&file), Ok(PathBuf::from("/tmp/x/RE 2026.pdf")));
        assert!(file.is_native());
    }

    // AC4: browsers offer a dragged link as text
    #[test]
    fn dropped_links_are_addresses() {
        let file = dropped_address("https://example.com/RE-2026-0042.pdf\r\n").expect("address");
        assert_eq!(file.uri(), "https://example.com/RE-2026-0042.pdf");
        assert_eq!(
            local_path(&file),
            Err("Die Adresse https://example.com/RE-2026-0042.pdf ist keine lokale Datei".into())
        );

        let file = dropped_address("file:///tmp/x/RE-2026-0042.pdf").expect("address");
        assert_eq!(
            local_path(&file),
            Ok(PathBuf::from("/tmp/x/RE-2026-0042.pdf"))
        );

        assert!(dropped_address("Rechnung vom 17.11.").is_none());
        assert!(dropped_address("").is_none());
    }
}
