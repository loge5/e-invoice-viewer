//! Spec 0017 — invoice content in German and English.

mod common;

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use common::model::{field, section};
use common::validation::variant;
use common::{fixture, replace_once, sample_l_cii, sample_l_ubl};
use e_invoice_viewer_core::attribution::attribute;
use e_invoice_viewer_core::fields::{FieldModel, map_invoice};
use e_invoice_viewer_core::i18n::Language;
use e_invoice_viewer_core::validate::{Validation, Validator};

fn model_in(xml: &str, language: Language) -> FieldModel {
    map_invoice(xml.as_bytes(), language).expect("field model")
}

fn validate_in(xml: &str, language: Language) -> Validation {
    Validator::new()
        .validate(xml.as_bytes(), language)
        .expect("validation")
}

fn titles(model: &FieldModel) -> Vec<&str> {
    let mut titles: Vec<&str> = Vec::new();
    for section in &model.sections {
        if titles.last() != Some(&section.title.as_str()) {
            titles.push(&section.title);
        }
    }
    titles
}

// AC1
#[test]
fn section_titles_in_both_languages() {
    let xml = fixture("cii/sample-a.xml");
    let german_model = model_in(&xml, Language::German);
    let german: Vec<&str> = titles(&german_model)
        .into_iter()
        .filter(|title| !title.starts_with("Position 2"))
        .collect();
    let english_model = model_in(&xml, Language::English);
    let english: Vec<&str> = titles(&english_model)
        .into_iter()
        .filter(|title| !title.starts_with("Invoice line 2"))
        .collect();

    assert_eq!(
        german,
        [
            "Rechnung",
            "Verkäufer (BG-4)",
            "Käufer (BG-7)",
            "Position 1",
            "Zahlung (BG-16)",
            "Umsatzsteuer (BG-23)",
            "Summen (BG-22)",
        ]
    );
    assert_eq!(
        english,
        [
            "Invoice",
            "Seller (BG-4)",
            "Buyer (BG-7)",
            "Invoice line 1",
            "Payment instructions (BG-16)",
            "VAT breakdown (BG-23)",
            "Document totals (BG-22)",
        ]
    );
}

// AC2
#[test]
fn field_labels_and_displays_in_both_languages() {
    let xml = fixture("cii/sample-a.xml");
    let german = model_in(&xml, Language::German);
    let english = model_in(&xml, Language::English);
    for (german_section, english_section, id, labels, displays) in [
        (
            "Rechnung",
            "Invoice",
            "BT-1",
            ("Rechnungsnummer", "Invoice number"),
            ("RE-2026-0042", "RE-2026-0042"),
        ),
        (
            "Rechnung",
            "Invoice",
            "BT-2",
            ("Rechnungsdatum", "Invoice issue date"),
            ("17.11.2026", "2026-11-17"),
        ),
        (
            "Rechnung",
            "Invoice",
            "BT-3",
            ("Rechnungsart", "Invoice type code"),
            ("Rechnung (380)", "Commercial invoice (380)"),
        ),
        (
            "Zahlung (BG-16)",
            "Payment instructions (BG-16)",
            "BT-81",
            ("Zahlungsart", "Payment means type code"),
            ("SEPA-Überweisung (58)", "SEPA credit transfer (58)"),
        ),
        (
            "Position 1",
            "Invoice line 1",
            "BT-129",
            ("Menge", "Invoiced quantity"),
            ("8 Stunden (HUR)", "8 hours (HUR)"),
        ),
        (
            "Summen (BG-22)",
            "Document totals (BG-22)",
            "BT-106",
            (
                "Summe der Positionen netto",
                "Sum of Invoice line net amount",
            ),
            ("1.050,00 EUR", "1,050.00 EUR"),
        ),
        (
            "Summen (BG-22)",
            "Document totals (BG-22)",
            "BT-112",
            ("Rechnungsbetrag inkl. USt", "Invoice total amount with VAT"),
            ("1.249,50 EUR", "1,249.50 EUR"),
        ),
    ] {
        let de = field(section(&german, german_section), id);
        let en = field(section(&english, english_section), id);
        assert_eq!((de.label.as_str(), en.label.as_str()), labels, "{id}");
        assert_eq!((de.display.as_str(), en.display.as_str()), displays, "{id}");
    }
}

/// CII sample A declaring the Factur-X profile Minimum.
fn a_minimum() -> String {
    variant("a-minimum.xml")
}

/// CII sample B declaring XRechnung 2.3.
fn b_version_2_3() -> String {
    variant("b-version-2-3.xml")
}

// AC6
#[test]
fn validation_notes_in_both_languages() {
    let reason = |language| validate_in(&a_minimum(), language).not_applicable_reason;
    assert_eq!(
        reason(Language::German).as_deref(),
        Some("Profil Minimum: keine vollständige Rechnung nach EN 16931")
    );
    assert_eq!(
        reason(Language::English).as_deref(),
        Some("Profile Minimum: not a complete invoice according to EN 16931")
    );

    let note = |language| validate_in(&b_version_2_3(), language).version_note;
    assert_eq!(
        note(Language::German).as_deref(),
        Some("Geprüft nach XRechnung 3.0.2, Rechnung erklärt Version 2.3")
    );
    assert_eq!(
        note(Language::English).as_deref(),
        Some("Checked against XRechnung 3.0.2, invoice declares version 2.3")
    );
}

// AC7
#[test]
fn findings_are_language_independent() {
    let b_without_bt10 = replace_once(
        &b_version_2_3(),
        "      <ram:BuyerReference>04011000-12345-34</ram:BuyerReference>\n",
        "",
    );
    for (xml, rule_id) in [
        (b_without_bt10, "BR-DE-15"),
        (variant("a-grand-total-1300.xml"), "BR-CO-15"),
    ] {
        let german = validate_in(&xml, Language::German);
        let english = validate_in(&xml, Language::English);
        assert_eq!(german.findings, english.findings, "{rule_id}");
        let finding = english
            .findings
            .iter()
            .find(|finding| finding.rule_id == rule_id)
            .unwrap_or_else(|| panic!("{rule_id}"));
        assert_eq!(
            attribute(&model_in(&xml, Language::German), finding),
            attribute(&model_in(&xml, Language::English), finding),
            "{rule_id}"
        );
    }
    let cen = validate_in(&variant("a-grand-total-1300.xml"), Language::German);
    let message = &cen
        .findings
        .iter()
        .find(|finding| finding.rule_id == "BR-CO-15")
        .expect("BR-CO-15")
        .message;
    assert!(message.starts_with("[BR-CO-15]-Invoice total amount with VAT"));
}

// AC8
#[test]
fn models_differ_only_in_texts() {
    for (name, xml) in [
        ("CII A", fixture("cii/sample-a.xml")),
        ("CII B", fixture("cii/sample-b.xml")),
        ("CII L", sample_l_cii()),
        ("UBL A", fixture("ubl/sample-a.xml")),
        ("UBL B", fixture("ubl/sample-b.xml")),
        ("UBL L", sample_l_ubl()),
    ] {
        let german = model_in(&xml, Language::German);
        let english = model_in(&xml, Language::English);
        let neutral = |model: &FieldModel| -> Vec<_> {
            model
                .sections
                .iter()
                .map(|section| {
                    (
                        section.location.clone(),
                        section.repeated,
                        section
                            .fields
                            .iter()
                            .map(|field| {
                                (
                                    field.id.clone(),
                                    field.raw.clone(),
                                    field.location.clone(),
                                    field.attribute.clone(),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect()
        };
        assert_eq!(neutral(&german), neutral(&english), "{name}");
    }
}

/// The `msgid`s of a PO or POT file, with whether each is fuzzy or has an
/// empty translation; obsolete entries (`#~`) are reported as such.
struct Catalogue {
    translated: BTreeSet<String>,
    untranslated: BTreeSet<String>,
    fuzzy: BTreeSet<String>,
    obsolete: usize,
}

fn unquote(line: &str) -> String {
    let inner = line.trim();
    let inner = inner
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or_default();
    inner.replace("\\\"", "\"").replace("\\n", "\n")
}

fn read_catalogue(text: &str) -> Catalogue {
    let mut catalogue = Catalogue {
        translated: BTreeSet::new(),
        untranslated: BTreeSet::new(),
        fuzzy: BTreeSet::new(),
        obsolete: 0,
    };
    for entry in text.split("\n\n") {
        if entry.lines().any(|line| line.starts_with("#~")) {
            catalogue.obsolete += 1;
            continue;
        }
        let fuzzy = entry
            .lines()
            .any(|line| line.starts_with("#,") && line.contains("fuzzy"));
        // (keyword, text) of msgid, msgid_plural, msgstr and msgstr[n].
        let mut parts: Vec<(String, String)> = Vec::new();
        for line in entry.lines().filter(|line| !line.starts_with('#')) {
            if line.starts_with('"') {
                if let Some((_, text)) = parts.last_mut() {
                    text.push_str(&unquote(line));
                }
            } else if let Some((keyword, rest)) = line.split_once(' ') {
                parts.push((keyword.to_string(), unquote(rest)));
            }
        }
        let Some((_, msgid)) = parts.iter().find(|(keyword, _)| keyword == "msgid") else {
            continue;
        };
        if msgid.is_empty() {
            continue; // the header
        }
        let empty = parts
            .iter()
            .filter(|(keyword, _)| keyword.starts_with("msgstr"))
            .any(|(_, text)| text.is_empty());
        if fuzzy {
            catalogue.fuzzy.insert(msgid.clone());
        } else if empty {
            catalogue.untranslated.insert(msgid.clone());
        } else {
            catalogue.translated.insert(msgid.clone());
        }
    }
    catalogue
}

/// Every source file of the workspace.
fn source_files(dir: &Path, files: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("read dir") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            source_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path.to_string_lossy().into_owned());
        }
    }
}

// AC9
#[test]
fn german_catalogue_is_complete() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for crate_dir in ["crates/core/src", "crates/app/src"] {
        source_files(&root.join(crate_dir), &mut files);
    }
    files.sort();
    let output = Command::new("xgettext")
        .args(e_invoice_viewer_core::i18n::XGETTEXT_ARGS)
        .args(["-o", "-"])
        .args(&files)
        .output()
        .expect("xgettext (GNU gettext) must be installed");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let template = read_catalogue(&String::from_utf8(output.stdout).expect("UTF-8"));
    let wanted: BTreeSet<String> = template
        .translated
        .union(&template.untranslated)
        .cloned()
        .collect();
    assert!(wanted.len() > 100, "only {} texts found", wanted.len());

    let german = read_catalogue(&std::fs::read_to_string(root.join("po/de.po")).expect("po/de.po"));
    let missing: Vec<_> = wanted.difference(&german.translated).collect();
    let stale: Vec<_> = german
        .translated
        .union(&german.untranslated)
        .filter(|msgid| !wanted.contains(*msgid))
        .collect();
    assert!(
        missing.is_empty(),
        "not translated in po/de.po: {missing:#?}"
    );
    assert!(stale.is_empty(), "stale in po/de.po: {stale:#?}");
    assert!(german.fuzzy.is_empty(), "fuzzy: {:#?}", german.fuzzy);
    assert_eq!(german.obsolete, 0, "obsolete entries in po/de.po");
}
