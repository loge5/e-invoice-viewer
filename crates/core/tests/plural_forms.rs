//! Spec 0030 AC5 — every catalogue's plural rule picks the form its
//! language needs, as evaluated by the gettext library the app uses.

use std::path::Path;
use std::process::Command;

use e_invoice_viewer_core::i18n::Language;

/// The plural form index a count needs in `language`, written out from the
/// rules in the notes of spec 0030 (the reference for this test).
fn expected_form(language: Language, n: u64) -> usize {
    use Language::{
        Croatian, Czech, French, Irish, Latvian, Lithuanian, Maltese, Polish, Romanian, Slovak,
        Slovenian,
    };
    let (n10, n100) = (n % 10, n % 100);
    match language {
        French => usize::from(n > 1),
        Romanian => match n {
            1 => 0,
            _ if n == 0 || (1..20).contains(&n100) => 1,
            _ => 2,
        },
        Latvian => match n {
            _ if n10 == 1 && n100 != 11 => 0,
            0 => 2,
            _ => 1,
        },
        Czech | Slovak => match n {
            1 => 0,
            2..=4 => 1,
            _ => 2,
        },
        Polish => match n {
            1 => 0,
            _ if (2..=4).contains(&n10) && !(10..20).contains(&n100) => 1,
            _ => 2,
        },
        Croatian => match n {
            _ if n10 == 1 && n100 != 11 => 0,
            _ if (2..=4).contains(&n10) && !(10..20).contains(&n100) => 1,
            _ => 2,
        },
        Lithuanian => match n {
            _ if n10 == 1 && n100 != 11 => 0,
            _ if n10 >= 2 && !(10..20).contains(&n100) => 1,
            _ => 2,
        },
        Slovenian => match n100 {
            1 => 0,
            2 => 1,
            3 | 4 => 2,
            _ => 3,
        },
        Maltese => match n {
            1 => 0,
            _ if n == 0 || (2..11).contains(&n100) => 1,
            _ if (11..20).contains(&n100) => 2,
            _ => 3,
        },
        Irish => match n {
            1 => 0,
            2 => 1,
            3..=6 => 2,
            7..=10 => 3,
            _ => 4,
        },
        _ => usize::from(n != 1),
    }
}

/// The number of plural forms of `language`: one more than the highest form.
fn expected_nplurals(language: Language) -> usize {
    (0..200)
        .map(|n| expected_form(language, n))
        .max()
        .unwrap_or(0)
        + 1
}

/// The `Plural-Forms` header of a catalogue.
fn plural_forms(po: &str) -> String {
    po.lines()
        .find_map(|line| line.strip_prefix("\"Plural-Forms: "))
        .and_then(|rest| rest.strip_suffix("\\n\""))
        .unwrap_or_else(|| panic!("no Plural-Forms header"))
        .to_string()
}

/// A catalogue with `plural_forms` and one entry whose forms are the
/// digits `0`, `1`, …, compiled with msgfmt and parsed by the gettext
/// library.
fn probe_catalogue(code: &str, plural_forms: &str, nplurals: usize) -> gettext::Catalog {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("plural-forms");
    std::fs::create_dir_all(&dir).expect("create dir");
    let mut po = format!(
        "msgid \"\"\nmsgstr \"\"\n\"Content-Type: text/plain; charset=UTF-8\\n\"\n\"Plural-Forms: {plural_forms}\\n\"\n\nmsgid \"one\"\nmsgid_plural \"many\"\n"
    );
    for form in 0..nplurals {
        po.push_str(&format!("msgstr[{form}] \"{form}\"\n"));
    }
    let (po_path, mo_path) = (
        dir.join(format!("{code}.po")),
        dir.join(format!("{code}.mo")),
    );
    std::fs::write(&po_path, po).expect("write po");
    let status = Command::new("msgfmt")
        .arg("--check")
        .arg("-o")
        .arg(&mo_path)
        .arg(&po_path)
        .status()
        .expect("msgfmt (GNU gettext) must be installed");
    assert!(status.success(), "{code}: msgfmt rejects {plural_forms}");
    gettext::Catalog::parse(std::fs::File::open(&mo_path).expect("open mo")).expect("parse mo")
}

#[test]
fn plural_forms_follow_the_language_rules() {
    let po_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../po");
    for language in Language::ALL {
        if language == Language::English {
            continue;
        }
        let code = language.code();
        let po = std::fs::read_to_string(po_dir.join(format!("{code}.po")))
            .unwrap_or_else(|error| panic!("{code}.po: {error}"));
        let header = plural_forms(&po);
        let nplurals = expected_nplurals(language);
        assert!(
            header.starts_with(&format!("nplurals={nplurals};")),
            "{code}: {header}, expected {nplurals} forms"
        );
        let catalogue = probe_catalogue(code, &header, nplurals);
        for n in [0, 1, 2, 3, 5, 11, 21, 22, 101] {
            assert_eq!(
                catalogue.ngettext("one", "many", n),
                expected_form(language, n).to_string(),
                "{code}: form for {n} ({header})"
            );
        }
    }
}
