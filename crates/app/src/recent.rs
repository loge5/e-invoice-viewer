//! The list of recently opened files (spec 0012).
//!
//! The list belongs to the user and is kept in the app's configuration
//! directory; it is shared by all windows of the application.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::config_store::{self, LOG_DOMAIN};
use crate::texts;

/// At most this many entries are kept.
pub const MAX_ENTRIES: usize = 10;

/// Recently opened files, most recent first, without duplicates.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecentFiles {
    entries: Vec<PathBuf>,
}

impl RecentFiles {
    pub fn entries(&self) -> &[PathBuf] {
        &self.entries
    }

    /// Puts `path` on top. Paths that cannot be stored (not absolute, not
    /// UTF-8) are ignored.
    pub fn add(&mut self, path: &Path) {
        if !path.is_absolute() || path.to_str().is_none() {
            return;
        }
        self.remove(path);
        self.entries.insert(0, path.to_path_buf());
        self.entries.truncate(MAX_ENTRIES);
    }

    pub fn remove(&mut self, path: &Path) {
        self.entries.retain(|entry| entry != path);
    }
}

/// The list shared by all windows of the application, with the menu that
/// shows it. The store is read once and written after each change.
pub struct RecentStore {
    path: PathBuf,
    files: RefCell<RecentFiles>,
    menu: gio::Menu,
}

impl RecentStore {
    pub fn new() -> Rc<Self> {
        Self::with_path(store_path())
    }

    fn with_path(path: PathBuf) -> Rc<Self> {
        let (files, warning) = load(&path);
        if let Some(warning) = warning {
            glib::g_warning!(LOG_DOMAIN, "{warning}");
        }
        let store = Rc::new(Self {
            path,
            files: RefCell::new(files),
            menu: gio::Menu::new(),
        });
        store.update_menu();
        store
    }

    /// The menu "Zuletzt geöffnet"; its entries activate `win.open-recent`
    /// with the path as target.
    pub fn menu(&self) -> &gio::Menu {
        &self.menu
    }

    pub fn add(&self, path: &Path) {
        self.change(|files| files.add(path));
    }

    pub fn remove(&self, path: &Path) {
        self.change(|files| files.remove(path));
    }

    fn change(&self, change: impl FnOnce(&mut RecentFiles)) {
        let changed = {
            let mut files = self.files.borrow_mut();
            let before = files.clone();
            change(&mut files);
            (*files != before).then(|| files.clone())
        };
        if let Some(files) = changed {
            if let Err(error) = save(&self.path, &files) {
                glib::g_warning!(
                    LOG_DOMAIN,
                    "recent files list {} not saved: {error}",
                    self.path.display()
                );
            }
            self.update_menu();
        }
    }

    fn update_menu(&self) {
        let files = self.files.borrow();
        let entries = files.entries();
        self.menu.remove_all();
        if entries.is_empty() {
            // An item without action is shown insensitive.
            self.menu.append(Some(texts::t(texts::RECENT_EMPTY)), None);
        }
        let home = glib::home_dir();
        for path in entries {
            let Some(target) = path.to_str() else {
                continue;
            };
            // Underscores would be taken as mnemonics.
            let label = texts::recent_entry_label(path, &home).replace('_', "__");
            let item = gio::MenuItem::new(Some(&label), None);
            item.set_action_and_target_value(Some("win.open-recent"), Some(&target.to_variant()));
            self.menu.append_item(&item);
        }
    }
}

const GROUP: &str = "Recent";

/// Key of the n-th entry (0 = most recent).
fn key(n: usize) -> String {
    format!("file{n}")
}

/// Where the list is stored.
pub fn store_path() -> PathBuf {
    config_store::path("recent")
}

/// Reads the stored list. Returns the list and, if the store exists but
/// cannot be used, a warning to log.
pub fn load(path: &Path) -> (RecentFiles, Option<String>) {
    let key_file = match config_store::read(path) {
        Ok(Some(key_file)) if key_file.has_group(GROUP) => key_file,
        Ok(Some(_)) => return unusable(path, "no list"),
        Ok(None) => return (RecentFiles::default(), None),
        Err(reason) => return unusable(path, &reason),
    };
    let mut files = RecentFiles::default();
    for n in (0..MAX_ENTRIES).rev() {
        if let Ok(entry) = key_file.string(GROUP, &key(n)) {
            files.add(Path::new(entry.as_str()));
        }
    }
    (files, None)
}

fn unusable(path: &Path, reason: &str) -> (RecentFiles, Option<String>) {
    (
        RecentFiles::default(),
        Some(format!(
            "recent files list {} ignored: {reason}",
            path.display()
        )),
    )
}

/// Stores the list; the previous file is replaced atomically, so that no
/// half-written list can remain.
pub fn save(path: &Path, files: &RecentFiles) -> Result<(), String> {
    let key_file = glib::KeyFile::new();
    // The group is written even for an empty list, so that an empty list
    // can be told apart from a damaged store.
    key_file.set_integer(GROUP, "version", 1);
    for (n, path) in files
        .entries
        .iter()
        .filter_map(|path| path.to_str())
        .enumerate()
    {
        key_file.set_string(GROUP, &key(n), path);
    }
    config_store::write(path, &key_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(paths: &[&str]) -> RecentFiles {
        let mut files = RecentFiles::default();
        for path in paths {
            files.add(Path::new(path));
        }
        files
    }

    fn entries(files: &RecentFiles) -> Vec<&str> {
        files
            .entries()
            .iter()
            .map(|path| path.to_str().expect("UTF-8"))
            .collect()
    }

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("e-invoice-viewer-tests")
            .join(format!("{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        dir
    }

    // AC5
    #[test]
    fn reopening_moves_entry_to_top_without_duplicates() {
        let files = list(&["/r/a.pdf", "/r/b.xml", "/r/a.pdf"]);

        assert_eq!(entries(&files), ["/r/a.pdf", "/r/b.xml"]);
    }

    // AC5
    #[test]
    fn eleventh_entry_drops_oldest() {
        let paths: Vec<String> = (1..=11).map(|n| format!("/r/{n}.pdf")).collect();
        let paths: Vec<&str> = paths.iter().map(String::as_str).collect();

        let files = list(&paths);

        assert_eq!(files.entries().len(), MAX_ENTRIES);
        assert_eq!(entries(&files)[0], "/r/11.pdf");
        assert_eq!(entries(&files)[9], "/r/2.pdf");
    }

    // AC5
    #[test]
    fn same_name_in_different_folders_both_listed() {
        let files = list(&["/tmp/x/rechnung.pdf", "/tmp/y/rechnung.pdf"]);

        assert_eq!(
            entries(&files),
            ["/tmp/y/rechnung.pdf", "/tmp/x/rechnung.pdf"]
        );
    }

    // AC8
    #[test]
    fn remove_drops_entry() {
        let mut files = list(&["/r/alt.pdf", "/r/b.xml"]);

        files.remove(Path::new("/r/alt.pdf"));

        assert_eq!(entries(&files), ["/r/b.xml"]);
    }

    // AC9: first start
    #[test]
    fn missing_store_is_empty_without_warning() {
        let path = scratch_dir("missing").join("recent");

        assert_eq!(load(&path), (RecentFiles::default(), None));
    }

    // AC9
    #[test]
    fn empty_truncated_or_garbage_store_is_empty_with_one_warning() {
        let dir = scratch_dir("broken");
        for (name, content) in [
            ("empty", &b""[..]),
            ("truncated", b"[Rec"),
            ("garbage", b"\xff\xfe not a key file \x00"),
            ("nul", b"[Recent]\nversion=1\x00\n"),
        ] {
            let path = dir.join(name);
            std::fs::write(&path, content).expect("write store");

            let (files, warning) = load(&path);

            assert_eq!(files, RecentFiles::default(), "{name}");
            assert!(warning.is_some(), "{name}");
        }
    }

    // AC9
    #[test]
    fn save_then_load_round_trip() {
        let path = scratch_dir("round-trip").join("nested").join("recent");
        let files = list(&["/r/b.xml", "/r/Rechnung; März.pdf", "/r/a.pdf"]);

        save(&path, &files).expect("save");

        assert_eq!(load(&path), (files, None));
    }

    // AC10
    #[test]
    fn store_lives_in_app_config_dir() {
        assert_eq!(
            store_path(),
            gtk::glib::user_config_dir()
                .join("e-invoice-viewer")
                .join("recent")
        );
    }

    /// Label and `win.open-recent` target of each menu item.
    fn menu_items(menu: &gio::Menu) -> Vec<(String, Option<String>)> {
        (0..menu.n_items())
            .map(|n| {
                let label = menu
                    .item_attribute_value(n, "label", None)
                    .and_then(|value| value.get::<String>())
                    .expect("label");
                let target = menu
                    .item_attribute_value(n, "target", None)
                    .and_then(|value| value.get::<String>());
                (label, target)
            })
            .collect()
    }

    // AC5, AC6, AC8: all windows show the same menu model
    #[test]
    fn menu_follows_the_list() {
        let dir = scratch_dir("menu");
        let store = RecentStore::with_path(dir.join("recent"));
        assert_eq!(
            menu_items(store.menu()),
            [("Keine Einträge".to_string(), None)]
        );

        store.add(Path::new("/tmp/x/a_1.pdf"));
        store.add(Path::new("/tmp/x/b.xml"));
        store.add(Path::new("/tmp/x/a_1.pdf"));

        assert_eq!(
            menu_items(store.menu()),
            [
                (
                    "a__1.pdf — /tmp/x".to_string(),
                    Some("/tmp/x/a_1.pdf".to_string())
                ),
                (
                    "b.xml — /tmp/x".to_string(),
                    Some("/tmp/x/b.xml".to_string())
                ),
            ]
        );
        assert_eq!(
            load(&dir.join("recent")).0.entries(),
            [
                PathBuf::from("/tmp/x/a_1.pdf"),
                PathBuf::from("/tmp/x/b.xml")
            ]
        );

        store.remove(Path::new("/tmp/x/a_1.pdf"));

        assert_eq!(menu_items(store.menu()).len(), 1);
    }
}
