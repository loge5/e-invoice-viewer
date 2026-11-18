//! Translations (spec 0017, ADR 0003).
//!
//! Texts are written in English and marked for translation: [`tr`] and
//! [`tr_n`] translate them where they are used, [`n_`] marks texts in tables
//! that are translated later. The German translation (`po/de.po`) is embedded
//! into the program. The language is always passed explicitly, so that texts
//! can be produced in either language at any time.

use std::sync::OnceLock;

use gettext::Catalog;

/// A language of the user interface and the invoice content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    German,
    English,
}

impl Language {
    /// All supported languages.
    pub const ALL: [Self; 2] = [Self::German, Self::English];

    /// The language code, e.g. `de`.
    pub fn code(self) -> &'static str {
        match self {
            Self::German => "de",
            Self::English => "en",
        }
    }

    /// The language with this code, e.g. `de`.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|language| language.code() == code)
    }
}

/// Arguments for `xgettext` that extract the texts marked for translation
/// from the Rust sources.
pub const XGETTEXT_ARGS: &[&str] = &[
    "--language=Rust",
    "--from-code=UTF-8",
    "--keyword=tr:2",
    "--keyword=tr_n:2,3",
    "--keyword=n_",
    "--keyword=term:2",
];

const GERMAN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/de.mo"));

fn catalogue(language: Language) -> &'static Catalog {
    static GERMAN_CATALOGUE: OnceLock<Catalog> = OnceLock::new();
    static ENGLISH_CATALOGUE: OnceLock<Catalog> = OnceLock::new();
    match language {
        Language::German => GERMAN_CATALOGUE
            .get_or_init(|| Catalog::parse(GERMAN).expect("the embedded catalogue is valid")),
        Language::English => ENGLISH_CATALOGUE.get_or_init(Catalog::empty),
    }
}

/// `text` in `language`; the English text itself if it has no translation.
pub fn tr(language: Language, text: &'static str) -> &'static str {
    catalogue(language).gettext(text)
}

/// The singular or plural form for `count` in `language`.
pub fn tr_n(
    language: Language,
    singular: &'static str,
    plural: &'static str,
    count: u64,
) -> &'static str {
    catalogue(language).ngettext(singular, plural, count)
}

/// Marks a text for translation without translating it, for tables; it is
/// translated where it is shown.
pub const fn n_(text: &'static str) -> &'static str {
    text
}
