//! Spec 0030 — invoice content in every official EU language.

mod common;

use common::model::field;
use common::validation::variant;
use common::{fixture, replace_once};
use e_invoice_viewer_core::fields::{FieldModel, map_invoice};
use e_invoice_viewer_core::i18n::{Language, tr};
use e_invoice_viewer_core::validate::{Validation, Validator};

fn model_in(xml: &str, language: Language) -> FieldModel {
    map_invoice(xml.as_bytes(), language).expect("field model")
}

fn validate_in(xml: &str, language: Language) -> Validation {
    Validator::new()
        .validate(xml.as_bytes(), language)
        .expect("validation")
}

/// Every language except English, the source language.
fn translated_languages() -> impl Iterator<Item = Language> {
    Language::ALL
        .into_iter()
        .filter(|language| *language != Language::English)
}

/// BT-112 of sample A and BT-2 per language, as in the table of the
/// requirement.
fn expected_amount_and_date(language: Language) -> (&'static str, &'static str) {
    use Language::*;
    match language {
        English => ("1,249.50 EUR", "2026-11-17"),
        Irish | Maltese => ("1,249.50 EUR", "17/11/2026"),
        German | Danish | Romanian | Slovenian => ("1.249,50 EUR", "17.11.2026"),
        Croatian => ("1.249,50 EUR", "17.11.2026."),
        Spanish | Italian | Greek => ("1.249,50 EUR", "17/11/2026"),
        Dutch => ("1.249,50 EUR", "17-11-2026"),
        French => ("1\u{202F}249,50 EUR", "17/11/2026"),
        Portuguese => ("1\u{A0}249,50 EUR", "17/11/2026"),
        Bulgarian | Czech | Estonian | Finnish | Polish | Slovak => {
            ("1\u{A0}249,50 EUR", "17.11.2026")
        }
        Latvian => ("1\u{A0}249,50 EUR", "17.11.2026."),
        Hungarian => ("1\u{A0}249,50 EUR", "2026.11.17."),
        Lithuanian | Swedish => ("1\u{A0}249,50 EUR", "2026-11-17"),
    }
}

/// The seller section of sample A in `language`.
fn seller_title(model: &FieldModel) -> &str {
    model
        .sections
        .iter()
        .find(|section| section.title.ends_with("(BG-4)"))
        .map(|section| section.title.as_str())
        .expect("seller section")
}

// AC1, AC2
#[test]
fn sample_a_in_every_language() {
    for path in ["cii/sample-a.xml", "ubl/sample-a.xml"] {
        let xml = fixture(path);
        let english = model_in(&xml, Language::English);
        for language in Language::ALL {
            let model = model_in(&xml, language);
            let seller = seller_title(&model).to_string();
            let (amount, date) = expected_amount_and_date(language);
            let totals = model
                .sections
                .iter()
                .find(|section| section.title.ends_with("(BG-22)"))
                .expect("totals");
            assert_eq!(
                field(totals, "BT-112").display,
                amount,
                "{path} {language:?}"
            );
            let invoice = &model.sections[0];
            assert_eq!(field(invoice, "BT-2").display, date, "{path} {language:?}");
            if language != Language::English {
                assert_ne!(seller, seller_title(&english), "{language:?}");
                assert_ne!(tr(language, "No findings"), "No findings", "{language:?}");
            }
        }
        for (language, status, seller) in [
            (Language::French, "Aucun constat", "Vendeur (BG-4)"),
            (Language::Spanish, "Sin observaciones", "Vendedor (BG-4)"),
            (Language::Dutch, "Geen bevindingen", "Verkoper (BG-4)"),
        ] {
            assert_eq!(tr(language, "No findings"), status);
            assert_eq!(seller_title(&model_in(&xml, language)), seller);
        }
    }
}

/// Labels and titles that may be the same as in English on purpose.
const SAME_ON_PURPOSE: [&str; 10] = [
    "IBAN",
    "BIC",
    "XML",
    "E-mail",
    "Document",
    "Nota",
    "Contact",
    "Region",
    "Percentage",
    "Factuur",
];

// AC4
#[test]
fn sample_d_labels_are_translated() {
    let xml = fixture("cii/sample-d.xml");
    let english = model_in(&xml, Language::English);
    for language in translated_languages() {
        let model = model_in(&xml, language);
        assert_eq!(model.sections.len(), english.sections.len(), "{language:?}");
        let mut untranslated = Vec::new();
        for (theirs, ours) in model.sections.iter().zip(&english.sections) {
            if theirs.title == ours.title && !theirs.title.starts_with("BG-") {
                untranslated.push(theirs.title.clone());
            }
            for (field, english_field) in theirs.fields.iter().zip(&ours.fields) {
                // Item attributes are labelled with their name from the
                // invoice, other data with its element names (spec 0031).
                if field.id == "BT-161" || field.id.is_empty() {
                    continue;
                }
                if field.label == english_field.label
                    && !SAME_ON_PURPOSE.contains(&field.label.as_str())
                {
                    untranslated.push(format!("{} {}", field.id, field.label));
                }
            }
        }
        assert!(untranslated.is_empty(), "{language:?}: {untranslated:#?}");
    }
}

// AC6
#[test]
fn official_messages_unchanged_own_notes_translated() {
    let b_without_bt10 = replace_once(
        &variant("b-version-2-3.xml"),
        "      <ram:BuyerReference>04011000-12345-34</ram:BuyerReference>\n",
        "",
    );
    let cases = [
        (variant("a-grand-total-1300.xml"), "BR-CO-15"),
        (b_without_bt10, "BR-DE-15"),
    ];
    let english_reason = validate_in(&variant("a-minimum.xml"), Language::English)
        .not_applicable_reason
        .expect("note");
    for language in translated_languages() {
        for (xml, rule_id) in &cases {
            let theirs = validate_in(xml, language);
            let ours = validate_in(xml, Language::English);
            assert_eq!(theirs.findings, ours.findings, "{language:?} {rule_id}");
            assert!(
                theirs.findings.iter().any(|f| f.rule_id == *rule_id),
                "{rule_id}"
            );
        }
        let reason = validate_in(&variant("a-minimum.xml"), language)
            .not_applicable_reason
            .expect("note");
        assert_ne!(reason, english_reason, "{language:?}");
        assert!(reason.contains("Minimum"), "{language:?}: {reason}");
    }
}

/// Section titles of business groups keep their number in every language.
#[test]
fn section_numbers_are_kept() {
    let xml = fixture("cii/sample-d.xml");
    let english = model_in(&xml, Language::English);
    for language in translated_languages() {
        let model = model_in(&xml, language);
        for (theirs, ours) in model.sections.iter().zip(&english.sections) {
            if let Some(start) = ours.title.find("(BG-") {
                assert!(
                    theirs.title.ends_with(&ours.title[start..]),
                    "{language:?}: {} / {}",
                    theirs.title,
                    ours.title
                );
            }
        }
    }
}
