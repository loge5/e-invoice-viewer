//! The language chosen in the app (spec 0019): the system language, or a
//! fixed language. It is stored in the app's configuration directory, apart
//! from the other settings.

use std::path::{Path, PathBuf};

use e_invoice_viewer_core::i18n::Language;
use gtk::glib;

use crate::config_store;
use crate::texts;

/// What the user chose in the language menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// Follow the system's preferred languages (0018).
    System,
    Fixed(Language),
}

impl Choice {
    /// All choices in menu order.
    pub const ALL: [Self; 3] = [
        Self::System,
        Self::Fixed(Language::German),
        Self::Fixed(Language::English),
    ];

    /// The stored form, also the target of the action `app.language`.
    pub fn code(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Fixed(language) => language.code(),
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|choice| choice.code() == code)
    }

    /// The language to use, given the system's preferred languages.
    pub fn effective<S: AsRef<str>>(self, preferences: &[S]) -> Language {
        match self {
            Self::System => Language::from_preferences(preferences),
            Self::Fixed(language) => language,
        }
    }

    /// The label in the language menu: each language is named in itself, so
    /// that it can be found without reading the current language.
    pub fn label(self) -> &'static str {
        match self {
            Self::System => texts::t(texts::SYSTEM_LANGUAGE),
            Self::Fixed(Language::German) => "Deutsch",
            Self::Fixed(Language::English) => "English",
        }
    }
}

const GROUP: &str = "Language";
const KEY: &str = "choice";

/// Where the choice is stored.
pub fn store_path() -> PathBuf {
    config_store::path("language")
}

/// Reads the stored choice: the system language if nothing is stored, and
/// also, with a warning to log, if the store cannot be used.
pub fn load(path: &Path) -> (Choice, Option<String>) {
    let unusable = |reason: &str| {
        (
            Choice::System,
            Some(format!(
                "language setting {} ignored: {reason}",
                path.display()
            )),
        )
    };
    match config_store::read(path) {
        Ok(None) => (Choice::System, None),
        Ok(Some(key_file)) => match key_file.string(GROUP, KEY) {
            Ok(code) => match Choice::from_code(code.as_str()) {
                Some(choice) => (choice, None),
                None => unusable(&format!("unknown language {code:?}")),
            },
            Err(error) => unusable(error.message()),
        },
        Err(reason) => unusable(&reason),
    }
}

/// Stores the choice; the previous file is replaced atomically.
pub fn save(path: &Path, choice: Choice) -> Result<(), String> {
    let key_file = glib::KeyFile::new();
    key_file.set_string(GROUP, KEY, choice.code());
    config_store::write(path, &key_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("e-invoice-viewer-tests")
            .join(format!("language-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        dir
    }

    // AC3
    #[test]
    fn choice_round_trip() {
        let path = scratch_dir("round-trip").join("nested").join("language");
        for choice in Choice::ALL {
            save(&path, choice).expect("save");
            assert_eq!(load(&path), (choice, None), "{choice:?}");
        }
    }

    // AC4
    #[test]
    fn damaged_or_unknown_choice_falls_back_with_one_warning() {
        let dir = scratch_dir("damaged");
        assert_eq!(load(&dir.join("missing")), (Choice::System, None));
        for (name, content) in [
            ("unknown", "[Language]\nchoice=xx\n"),
            ("empty", ""),
            ("garbage", "no key file ["),
            ("nul", "[Language]\nchoice=de\u{0}\n"),
        ] {
            let path = dir.join(name);
            std::fs::write(&path, content).expect("write");

            let (choice, warning) = load(&path);

            assert_eq!(choice, Choice::System, "{name}");
            assert!(warning.is_some(), "{name}");
        }
    }

    // AC1, AC3
    #[test]
    fn effective_language() {
        let english_system = ["en_US.UTF-8", "en", "C"];
        assert_eq!(Choice::System.effective(&english_system), Language::English);
        assert_eq!(
            Choice::Fixed(Language::German).effective(&english_system),
            Language::German
        );
        assert_eq!(
            Choice::Fixed(Language::English).effective(&["de_DE.UTF-8", "de"]),
            Language::English
        );
    }

    // AC4, AC5
    #[test]
    fn stored_apart_from_other_settings() {
        let dir = glib::user_config_dir().join("e-invoice-viewer");
        assert_eq!(store_path(), dir.join("language"));
        assert_ne!(store_path(), crate::recent::store_path());
        assert_ne!(store_path(), crate::window_size::store_path());

        let path = scratch_dir("content").join("language");
        save(&path, Choice::Fixed(Language::German)).expect("save");
        let content = std::fs::read_to_string(&path).expect("read");
        assert_eq!(content.trim(), "[Language]\nchoice=de");
    }

    // AC1
    #[test]
    fn language_choice_labels() {
        for language in Language::ALL {
            texts::set_language(language);
            let labels: Vec<&str> = Choice::ALL.into_iter().map(Choice::label).collect();
            let system = match language {
                Language::German => "Systemsprache",
                Language::English => "System language",
            };
            assert_eq!(labels, [system, "Deutsch", "English"]);
            assert_eq!(
                texts::t(texts::LANGUAGE_MENU),
                match language {
                    Language::German => "Sprache",
                    Language::English => "Language",
                }
            );
        }
        texts::set_language(Language::German);
    }
}
