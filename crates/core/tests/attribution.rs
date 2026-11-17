//! Spec 0011 — attributing validation findings to fields, sections or the
//! document.

mod common;

use common::validation::{validate, variant};
use common::{fixture, replace_once, sample_l_cii, select};
use e_invoice_viewer_core::attribution::{Target, attribute};
use e_invoice_viewer_core::fields::{FieldModel, map_invoice};
use e_invoice_viewer_core::validate::Finding;

/// CII sample A with BT-131 of line 2 removed.
fn a_line_2_without_bt131() -> String {
    let a = fixture("cii/sample-a.xml");
    let start = a
        .match_indices("<ram:LineTotalAmount>")
        .nth(1)
        .expect("line 2 net amount")
        .0;
    let line_start = a[..start].rfind('\n').expect("line start") + 1;
    let line_end = start + a[start..].find('\n').expect("line end") + 1;
    format!("{}{}", &a[..line_start], &a[line_end..])
}

/// CII sample B declaring XRechnung 2.3, without BT-10.
fn b_version_2_3_without_bt10() -> String {
    replace_once(
        &variant("b-version-2-3.xml"),
        "      <ram:BuyerReference>04011000-12345-34</ram:BuyerReference>\n",
        "",
    )
}

/// The samples for the manual checks of this spec, by file name.
fn manual_samples() -> Vec<(&'static str, String)> {
    vec![
        ("a.xml", fixture("cii/sample-a.xml")),
        ("b.xml", fixture("cii/sample-b.xml")),
        ("a-grand-total-1300.xml", variant("a-grand-total-1300.xml")),
        ("a-line-2-without-bt131.xml", a_line_2_without_bt131()),
        (
            "b-version-2-3-without-bt10.xml",
            b_version_2_3_without_bt10(),
        ),
        ("a-without-bt24.xml", variant("a-without-bt24.xml")),
        ("a-minimum.xml", variant("a-minimum.xml")),
        ("l.xml", sample_l_cii()),
    ]
}

/// Writes the samples of the manual checks to `target/manual-0011/`.
#[test]
#[ignore = "writes the samples for the manual checks of spec 0011"]
fn write_manual_samples() {
    let dir = format!("{}/../../target/manual-0011", env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(&dir).expect("create dir");
    for (name, xml) in manual_samples() {
        std::fs::write(format!("{dir}/{name}"), xml).expect("write sample");
    }
}

/// Where each finding of an invoice goes, as `(rule ID, target)` with the
/// target described as `field <section> <BT>`, `section <title>` or
/// `document`.
fn targets(xml: &str) -> Vec<(String, String)> {
    let model = map_invoice(xml.as_bytes()).expect("field model");
    validate(xml)
        .findings
        .iter()
        .map(|finding| (finding.rule_id.clone(), describe(&model, finding)))
        .collect()
}

fn describe(model: &FieldModel, finding: &Finding) -> String {
    match attribute(model, finding) {
        Target::Field { section, field } => format!(
            "field {} {}",
            model.sections[section].title, model.sections[section].fields[field].id
        ),
        Target::Section(section) => format!("section {}", model.sections[section].title),
        Target::Document => "document".to_string(),
    }
}

fn target_of<'a>(targets: &'a [(String, String)], rule_id: &str) -> &'a str {
    let found: Vec<&str> = targets
        .iter()
        .filter(|(id, _)| id == rule_id)
        .map(|(_, target)| target.as_str())
        .collect();
    assert_eq!(found.len(), 1, "{rule_id} in {targets:#?}");
    found[0]
}

// AC1
#[test]
fn grand_total_findings_go_to_bt112_and_bt115() {
    let targets = targets(&variant("a-grand-total-1300.xml"));

    assert_eq!(
        target_of(&targets, "BR-CO-15"),
        "field Summen (BG-22) BT-112"
    );
    assert_eq!(
        target_of(&targets, "BR-CO-16"),
        "field Summen (BG-22) BT-115"
    );
}

// AC2
#[test]
fn line_finding_without_field_goes_to_its_line() {
    let targets = targets(&a_line_2_without_bt131());

    assert_eq!(target_of(&targets, "BR-24"), "section Position 2");
}

// AC3
#[test]
fn xrechnung_findings() {
    let targets = targets(&b_version_2_3_without_bt10());

    assert_eq!(target_of(&targets, "BR-DE-15"), "document");
    assert_eq!(target_of(&targets, "BR-DE-21"), "field Rechnung BT-24");
    assert_eq!(target_of(&targets, "BR-DE-TMP-32"), "document");
}

// AC5
#[test]
fn findings_without_field_go_to_document() {
    let targets = targets(&variant("a-without-bt24.xml"));

    assert_eq!(target_of(&targets, "BR-01"), "document");
    assert_eq!(target_of(&targets, "CII-SR-010"), "document");
}

// Requirement: a field with the finding's location comes first
#[test]
fn equal_location_comes_first() {
    let targets = targets(&a_line_2_without_bt131());

    // BR-S-08 is checked on the VAT category code (BT-118); its message
    // names BT-119 first.
    assert_eq!(
        target_of(&targets, "BR-S-08"),
        "field Umsatzsteuer (BG-23) BT-118"
    );
}

// Contract: the location of every section selects its element
#[test]
fn every_section_location_selects_its_element() {
    for xml in [fixture("cii/sample-a.xml"), fixture("cii/sample-b.xml")] {
        let model = map_invoice(xml.as_bytes()).expect("field model");
        for section in &model.sections {
            assert_eq!(
                select(&xml, &section.location).len(),
                1,
                "{} at {}",
                section.title,
                section.location
            );
            for field in &section.fields {
                assert!(
                    field
                        .location
                        .starts_with(&format!("{}/", section.location)),
                    "{} {} not below {}",
                    section.title,
                    field.id,
                    section.location
                );
            }
        }
    }
}
