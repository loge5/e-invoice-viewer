//! Translations (spec 0017, ADR 0003).
//!
//! Texts are written in English and marked for translation: [`tr`] and
//! [`tr_n`] translate them where they are used, [`n_`] marks texts in tables
//! that are translated later. The translations (`po/<code>.po`) are embedded
//! into the program. The language is always passed explicitly, so that texts
//! can be produced in any language at any time.

use std::sync::OnceLock;

use gettext::Catalog;

/// A language of the user interface and the invoice content: the official
/// languages of the EU (spec 0030).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Bulgarian,
    Spanish,
    Czech,
    Danish,
    German,
    Estonian,
    Greek,
    English,
    French,
    Irish,
    Croatian,
    Italian,
    Latvian,
    Lithuanian,
    Hungarian,
    Maltese,
    Dutch,
    Polish,
    Portuguese,
    Romanian,
    Slovak,
    Slovenian,
    Finnish,
    Swedish,
}

/// How a language writes dates (0017, 0030).
#[derive(Debug, Clone, Copy)]
enum DateForm {
    /// Day, month, year with this separator, e.g. `17.11.2026`.
    DayMonthYear(char),
    /// As `DayMonthYear`, with a closing separator: `17.11.2026.`.
    DayMonthYearClosed(char),
    /// Year, month, day with this separator, e.g. `2026-11-17`.
    YearMonthDay(char),
    /// As `YearMonthDay`, with a closing separator: `2026.11.17.`.
    YearMonthDayClosed(char),
}

/// What differs between languages apart from the texts (0017, 0030).
struct Conventions {
    code: &'static str,
    /// The language's name in itself, as shown in the language menu.
    name: &'static str,
    decimal_separator: char,
    thousands_separator: char,
    date: DateForm,
}

/// No-break space, the thousands separator of many languages (CLDR).
const NBSP: char = '\u{A0}';

impl Language {
    /// Every language, in the EU's protocol order, which is the order of the
    /// language menu (0019, 0030).
    pub const ALL: [Self; 24] = [
        Self::Bulgarian,
        Self::Spanish,
        Self::Czech,
        Self::Danish,
        Self::German,
        Self::Estonian,
        Self::Greek,
        Self::English,
        Self::French,
        Self::Irish,
        Self::Croatian,
        Self::Italian,
        Self::Latvian,
        Self::Lithuanian,
        Self::Hungarian,
        Self::Maltese,
        Self::Dutch,
        Self::Polish,
        Self::Portuguese,
        Self::Romanian,
        Self::Slovak,
        Self::Slovenian,
        Self::Finnish,
        Self::Swedish,
    ];

    fn conventions(self) -> Conventions {
        use DateForm::{DayMonthYear, DayMonthYearClosed, YearMonthDay, YearMonthDayClosed};
        let c = |code, name, (decimal_separator, thousands_separator), date| Conventions {
            code,
            name,
            decimal_separator,
            thousands_separator,
            date,
        };
        let comma_dot = (',', '.');
        let comma_space = (',', NBSP);
        let point_comma = ('.', ',');
        match self {
            Self::Bulgarian => c("bg", "Български", comma_space, DayMonthYear('.')),
            Self::Spanish => c("es", "Español", comma_dot, DayMonthYear('/')),
            Self::Czech => c("cs", "Čeština", comma_space, DayMonthYear('.')),
            Self::Danish => c("da", "Dansk", comma_dot, DayMonthYear('.')),
            Self::German => c("de", "Deutsch", comma_dot, DayMonthYear('.')),
            Self::Estonian => c("et", "Eesti", comma_space, DayMonthYear('.')),
            Self::Greek => c("el", "Ελληνικά", comma_dot, DayMonthYear('/')),
            Self::English => c("en", "English", point_comma, YearMonthDay('-')),
            Self::French => c("fr", "Français", (',', '\u{202F}'), DayMonthYear('/')),
            Self::Irish => c("ga", "Gaeilge", point_comma, DayMonthYear('/')),
            Self::Croatian => c("hr", "Hrvatski", comma_dot, DayMonthYearClosed('.')),
            Self::Italian => c("it", "Italiano", comma_dot, DayMonthYear('/')),
            Self::Latvian => c("lv", "Latviešu", comma_space, DayMonthYearClosed('.')),
            Self::Lithuanian => c("lt", "Lietuvių", comma_space, YearMonthDay('-')),
            Self::Hungarian => c("hu", "Magyar", comma_space, YearMonthDayClosed('.')),
            Self::Maltese => c("mt", "Malti", point_comma, DayMonthYear('/')),
            Self::Dutch => c("nl", "Nederlands", comma_dot, DayMonthYear('-')),
            Self::Polish => c("pl", "Polski", comma_space, DayMonthYear('.')),
            Self::Portuguese => c("pt", "Português", comma_space, DayMonthYear('/')),
            Self::Romanian => c("ro", "Română", comma_dot, DayMonthYear('.')),
            Self::Slovak => c("sk", "Slovenčina", comma_space, DayMonthYear('.')),
            Self::Slovenian => c("sl", "Slovenščina", comma_dot, DayMonthYear('.')),
            Self::Finnish => c("fi", "Suomi", comma_space, DayMonthYear('.')),
            Self::Swedish => c("sv", "Svenska", comma_space, YearMonthDay('-')),
        }
    }

    /// The language code, e.g. `de`.
    pub fn code(self) -> &'static str {
        self.conventions().code
    }

    /// The language's name in itself, e.g. `Français` (0019, 0030).
    pub fn name(self) -> &'static str {
        self.conventions().name
    }

    /// The decimal separator of numbers (0017, 0030).
    pub fn decimal_separator(self) -> char {
        self.conventions().decimal_separator
    }

    /// The thousands separator of numbers (0017, 0030).
    pub fn thousands_separator(self) -> char {
        self.conventions().thousands_separator
    }

    /// A date in the notation of the language (0017, 0030).
    pub fn date(self, year: &str, month: &str, day: &str) -> String {
        match self.conventions().date {
            DateForm::DayMonthYear(s) => format!("{day}{s}{month}{s}{year}"),
            DateForm::DayMonthYearClosed(s) => format!("{day}{s}{month}{s}{year}{s}"),
            DateForm::YearMonthDay(s) => format!("{year}{s}{month}{s}{day}"),
            DateForm::YearMonthDayClosed(s) => format!("{year}{s}{month}{s}{day}{s}"),
        }
    }

    /// The language with this code, e.g. `de`.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|language| language.code() == code)
    }

    /// The first supported language among the user's preferred languages,
    /// most preferred first, as locale names (`de_DE.UTF-8`, `de`, `C`);
    /// English if none is supported (spec 0018). `C` and `POSIX` stand for
    /// English.
    pub fn from_preferences<S: AsRef<str>>(names: &[S]) -> Self {
        names
            .iter()
            .find_map(|name| {
                let code = name
                    .as_ref()
                    .split(['_', '.', '@'])
                    .next()
                    .unwrap_or_default();
                match code {
                    "C" | "POSIX" => Some(Self::English),
                    code => Self::from_code(code),
                }
            })
            .unwrap_or(Self::English)
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
    // Helpers of the app's texts module.
    "--keyword=t",
    "--keyword=tr_fill:1",
    "--keyword=tr_count:1,2",
];

// `CATALOGUES`: the compiled catalogue of every language except English,
// by language code, generated by `build.rs` from `po/`.
include!(concat!(env!("OUT_DIR"), "/catalogues.rs"));

fn catalogue(language: Language) -> &'static Catalog {
    static PARSED: [OnceLock<Catalog>; Language::ALL.len()] =
        [const { OnceLock::new() }; Language::ALL.len()];
    let index = Language::ALL
        .iter()
        .position(|candidate| *candidate == language)
        .expect("every language is in ALL");
    PARSED[index].get_or_init(|| {
        match CATALOGUES.iter().find(|(code, _)| *code == language.code()) {
            Some((_, mo)) => Catalog::parse(*mo).expect("the embedded catalogue is valid"),
            None => Catalog::empty(),
        }
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    // Spec 0018 AC1, AC2
    #[test]
    fn language_from_system_preferences() {
        let cases: [(&[&str], Language); 14] = [
            (
                &["de_DE.UTF-8", "de_DE", "de.UTF-8", "de", "C"],
                Language::German,
            ),
            (&["en_US.UTF-8", "en_US", "en", "C"], Language::English),
            (&["ru", "de", "C"], Language::German),
            // Spec 0030 AC8
            (&["fr_FR.UTF-8", "fr_FR", "fr", "C"], Language::French),
            (&["es_ES"], Language::Spanish),
            (&["nl_BE"], Language::Dutch),
            (&["pl_PL"], Language::Polish),
            (&["ga_IE"], Language::Irish),
            (&["el_GR"], Language::Greek),
            (&["pt_PT", "pt_BR"], Language::Portuguese),
            (&["ru_RU.UTF-8", "ru", "C"], Language::English),
            (&["tr_TR"], Language::English),
            (&["POSIX"], Language::English),
            (&[], Language::English),
        ];
        for (names, expected) in cases {
            assert_eq!(Language::from_preferences(names), expected, "{names:?}");
        }
    }

    // Spec 0030: every language but English has an embedded catalogue, and
    // every catalogue belongs to a language.
    #[test]
    fn every_language_has_a_catalogue() {
        let mut expected: Vec<&str> = Language::ALL
            .iter()
            .filter(|language| **language != Language::English)
            .map(|language| language.code())
            .collect();
        expected.sort_unstable();
        let embedded: Vec<&str> = CATALOGUES.iter().map(|(code, _)| *code).collect();
        assert_eq!(embedded, expected);
    }

    // Spec 0030 AC7: the menu names, in the EU protocol order.
    #[test]
    fn names_in_protocol_order() {
        let names: Vec<&str> = Language::ALL.iter().map(|l| l.name()).collect();
        assert_eq!(
            names,
            [
                "Български",
                "Español",
                "Čeština",
                "Dansk",
                "Deutsch",
                "Eesti",
                "Ελληνικά",
                "English",
                "Français",
                "Gaeilge",
                "Hrvatski",
                "Italiano",
                "Latviešu",
                "Lietuvių",
                "Magyar",
                "Malti",
                "Nederlands",
                "Polski",
                "Português",
                "Română",
                "Slovenčina",
                "Slovenščina",
                "Suomi",
                "Svenska",
            ]
        );
    }
}
