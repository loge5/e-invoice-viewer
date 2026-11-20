//! Spec 0027 AC1, AC2 — the sample invoices of the FeRD package against the
//! profile schemas and rules. The samples are not committed (their licence
//! is not stated): set `FERD_PACKAGE` to the unpacked package
//! (`ZUGFeRD_2.5.2_EN`) to run this test; without it, it passes empty.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use e_invoice_viewer_core::document::{Document, open};
use e_invoice_viewer_core::i18n::Language;
use e_invoice_viewer_core::validate::{Severity, Validation, Validator};

fn package() -> Option<PathBuf> {
    std::env::var_os("FERD_PACKAGE").map(PathBuf::from)
}

/// The files with `extension` below `dir`, sorted.
fn files(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("read dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
            {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// The invoice XML of a sample: the file itself, or the XML embedded in the
/// PDF (`None` for a PDF without invoice).
fn invoice_xml(path: &Path) -> Option<Vec<u8>> {
    let bytes = std::fs::read(path).expect("read sample");
    if !path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
    {
        return Some(bytes);
    }
    match open(&bytes, Language::English).expect("sample opens") {
        Document::Pdf { invoice, .. } => invoice.map(|invoice| invoice.xml),
        Document::Xml { invoice } => Some(invoice.xml),
    }
}

fn errors(validation: &Validation) -> Vec<String> {
    validation
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .map(|finding| {
            format!(
                "{} {} ({})",
                finding.rule_id, finding.message, finding.location
            )
        })
        .collect()
}

// AC1
#[test]
fn profile_samples_have_no_errors() {
    let Some(package) = package() else { return };
    let validator = Validator::new();
    let mut failures = Vec::new();
    let mut checked = 0;
    for (dir, profile) in [
        ("0. MINIMUM", "Minimum"),
        ("1. BASIC WL", "Basic WL"),
        ("2. BASIC", "Basic"),
        ("3. EN16931", "EN 16931"),
        ("4. EXTENDED", "Extended"),
    ] {
        let dir = package.join("Examples").join(dir);
        let samples = files(&dir, "xml").into_iter().chain(files(&dir, "pdf"));
        // Supporting documents of the samples lie in `attachments/`.
        for path in
            samples.filter(|path| !path.components().any(|c| c.as_os_str() == "attachments"))
        {
            let Some(xml) = invoice_xml(&path) else {
                failures.push(format!("{}: no invoice", path.display()));
                continue;
            };
            let validation = validator
                .validate_embedded(&xml, Language::English)
                .expect("sample is an invoice");
            checked += 1;
            let rules = format!("Factur-X 1.09.2 (ZUGFeRD 2.5.2) {profile}");
            let schema = format!("XML schema Factur-X 1.09.2 {profile}");
            if !validation.applied_rule_sets.contains(&rules)
                || !validation.applied_rule_sets.contains(&schema)
            {
                failures.push(format!(
                    "{}: applied {:?}",
                    path.display(),
                    validation.applied_rule_sets
                ));
            }
            for error in errors(&validation) {
                failures.push(format!("{}: {error}", path.display()));
            }
        }
    }
    assert!(checked > 100, "only {checked} samples found");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// AC2
#[test]
fn xrechnung_samples_get_no_profile_rules() {
    let Some(package) = package() else { return };
    let validator = Validator::new();
    let dir = package.join("Examples").join("5. XRECHNUNG");
    let samples = files(&dir, "xml");
    assert!(!samples.is_empty());
    for path in samples {
        let xml = std::fs::read(&path).expect("read sample");
        for validation in [
            validator.validate(&xml, Language::English),
            validator.validate_embedded(&xml, Language::English),
        ] {
            let validation = validation.expect("sample is an invoice");
            assert!(
                validation
                    .applied_rule_sets
                    .iter()
                    .all(|set| !set.contains("Factur-X")),
                "{}: {:?}",
                path.display(),
                validation.applied_rule_sets
            );
            assert!(
                validation
                    .applied_rule_sets
                    .iter()
                    .any(|set| set.starts_with("XRechnung")),
                "{}",
                path.display()
            );
        }
    }
}

// AC9: the largest Extended sample (by XML size), release build, with the
// rules compiled on first use; opening the PDF included.
#[test]
#[cfg_attr(debug_assertions, ignore = "timing applies to release builds")]
fn largest_extended_sample_within_two_seconds() {
    let Some(package) = package() else { return };
    let dir = package.join("Examples").join("4. EXTENDED");
    let largest_xml = files(&dir, "xml")
        .into_iter()
        .max_by_key(|path| std::fs::metadata(path).map(|m| m.len()).unwrap_or(0))
        .expect("Extended samples");
    let pdf = std::fs::read_dir(largest_xml.parent().expect("dir"))
        .expect("sample dir")
        .map(|entry| entry.expect("entry").path())
        .find(|path| {
            path.extension().is_some_and(|ext| ext == "pdf")
                && path
                    .file_stem()
                    .zip(largest_xml.file_stem())
                    .is_some_and(|(pdf, xml)| {
                        pdf.to_string_lossy().starts_with(&*xml.to_string_lossy())
                    })
        })
        .unwrap_or_else(|| largest_xml.clone());
    let start = Instant::now();
    let xml = invoice_xml(&pdf).expect("invoice");
    let validation = Validator::new()
        .validate_embedded(&xml, Language::English)
        .expect("validation");
    let elapsed = start.elapsed();
    println!(
        "{}: {elapsed:?}, {} findings",
        pdf.display(),
        validation.findings.len()
    );
    assert!(elapsed < Duration::from_secs(2), "took {elapsed:?}");
}
