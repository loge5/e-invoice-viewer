//! Spec 0025 — the public sample collections open and give a complete field
//! model. Their validation is compared with the KoSIT validator in
//! `validate_reference.rs` (keys `public/…` of the reference files).

mod common;

use std::path::Path;

use common::{fixture, select};
use e_invoice_viewer_core::document::{self, Document};
use e_invoice_viewer_core::i18n::Language;
use e_invoice_viewer_core::validate::Validator;

/// Every sample file below `fixtures/public`, as a path relative to
/// `fixtures`.
fn public_samples() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut pending = vec![root.join("public")];
    let mut files = Vec::new();
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("read dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "xml") {
                files.push(
                    path.strip_prefix(&root)
                        .expect("below fixtures")
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }
    files.sort();
    files
}

// AC1
#[test]
fn collections_are_present_with_licence_and_source() {
    let samples = public_samples();
    let count = |prefix: &str| samples.iter().filter(|s| s.starts_with(prefix)).count();
    assert_eq!(count("public/xrechnung-testsuite/standard/"), 66);
    assert_eq!(count("public/xrechnung-testsuite/extension/"), 6);
    assert_eq!(count("public/cen/cii/"), 15);
    assert_eq!(count("public/cen/ubl/"), 19);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/public");
    for file in [
        "README.md",
        "xrechnung-testsuite/LICENSE",
        "cen/LICENSE.txt",
    ] {
        assert!(root.join(file).is_file(), "{file}");
    }
    // Every sample is in the KoSIT reference.
    let reference = fixture("kosit/expected-en16931.tsv");
    for sample in &samples {
        assert!(
            reference
                .lines()
                .any(|line| line.starts_with(&format!("{sample}\t"))),
            "{sample} is not in expected-en16931.tsv"
        );
    }
}

// AC2
#[test]
fn every_public_sample_opens_with_a_complete_field_model() {
    let validator = Validator::new();
    let mut problems = Vec::new();
    for sample in public_samples() {
        let xml = fixture(&sample);
        let document = match document::open(xml.as_bytes(), Language::English) {
            Ok(document) => document,
            Err(error) => {
                problems.push(format!("{sample}: does not open: {error}"));
                continue;
            }
        };
        let Document::Xml { invoice, .. } = &document else {
            problems.push(format!("{sample}: not opened as an XML invoice"));
            continue;
        };
        let model = &invoice.fields;
        if let Some(empty) = model.sections.iter().find(|s| s.fields.is_empty()) {
            problems.push(format!("{sample}: empty section {:?}", empty.title));
        }
        let has = |id: &str| model.fields().any(|(_, _, field)| field.id == id);
        for id in ["BT-1", "BT-2", "BT-5"] {
            if !has(id) {
                problems.push(format!("{sample}: no {id}"));
            }
        }
        if !has("BT-112") && !has("BT-115") {
            problems.push(format!("{sample}: neither BT-112 nor BT-115"));
        }
        let validation = validator
            .validate(xml.as_bytes(), Language::English)
            .expect("validation");
        for finding in &validation.findings {
            if select(&xml, &finding.location).is_empty() {
                problems.push(format!(
                    "{sample}: {} at {} selects nothing",
                    finding.rule_id, finding.location
                ));
            }
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}
