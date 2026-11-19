//! Spec 0023 — XML schema check (CII D16B, UBL 2.1).

mod common;

use common::validation::{validate, variant};
use common::{fixture, replace_once, select};
use e_invoice_viewer_core::attribution::{Target, attribute};
use e_invoice_viewer_core::fields::map_invoice;
use e_invoice_viewer_core::i18n::Language;
use e_invoice_viewer_core::link::XmlLayout;
use e_invoice_viewer_core::validate::{Finding, SCHEMA_RULE_ID, Severity, Validation, Validator};

fn schema_findings(validation: &Validation) -> Vec<&Finding> {
    validation
        .findings
        .iter()
        .filter(|finding| finding.rule_id == SCHEMA_RULE_ID)
        .collect()
}

fn english(xml: &str) -> Validation {
    Validator::new()
        .validate(xml.as_bytes(), Language::English)
        .expect("validation")
}

// AC1
#[test]
fn samples_are_schema_valid() {
    for path in [
        "cii/sample-a.xml",
        "cii/sample-b.xml",
        "cii/sample-c.xml",
        "ubl/sample-a.xml",
        "ubl/sample-b.xml",
        "ubl/sample-c.xml",
    ] {
        let validation = english(&fixture(path));
        assert_eq!(
            schema_findings(&validation),
            Vec::<&Finding>::new(),
            "{path}"
        );
        let schema = if path.starts_with("cii") {
            "XML schema CII D16B"
        } else {
            "XML schema UBL 2.1"
        };
        assert_eq!(validation.applied_rule_sets[0], schema, "{path}");
    }
}

fn a_with_unknown_element() -> String {
    replace_once(
        &fixture("cii/sample-a.xml"),
        "    <ram:TypeCode>380</ram:TypeCode>\n",
        "    <ram:Foo>1</ram:Foo>\n    <ram:TypeCode>380</ram:TypeCode>\n",
    )
}

// AC2, AC7
#[test]
fn unknown_element_is_reported_at_its_location() {
    let xml = a_with_unknown_element();
    let validation = validate(&xml);
    let findings = schema_findings(&validation);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let finding = findings[0];
    assert_eq!(finding.severity, Severity::Error);
    assert!(finding.message.contains("ram:Foo"), "{}", finding.message);
    assert!(
        finding.message.contains("ram:TypeCode"),
        "{}",
        finding.message
    );
    assert!(!finding.message.contains('{'), "{}", finding.message);
    assert_eq!(
        finding.location,
        "/rsm:CrossIndustryInvoice/rsm:ExchangedDocument/ram:Foo"
    );
    assert_eq!(select(&xml, &finding.location), ["1"]);
    // One error in all: the status line reads `1 error`.
    let errors = validation
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();
    assert_eq!(errors, 1, "{:#?}", validation.findings);
    // "Show in XML" selects the element (0014).
    let layout = XmlLayout::new(xml.as_bytes()).expect("layout");
    let range = layout.element_range(&finding.location).expect("range");
    assert_eq!(&layout.text()[range], "<ram:Foo>1</ram:Foo>");
}

// AC3
#[test]
fn wrong_datatype_is_reported_at_the_field() {
    let xml = replace_once(
        &fixture("cii/sample-a.xml"),
        "<ram:LineTotalAmount>1050.00</ram:LineTotalAmount>",
        "<ram:LineTotalAmount>abc</ram:LineTotalAmount>",
    );
    let validation = validate(&xml);
    let findings = schema_findings(&validation);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let model = map_invoice(xml.as_bytes(), Language::German).expect("model");
    match attribute(&model, findings[0]) {
        Target::Field { section, field } => {
            assert_eq!(model.sections[section].title, "Summen (BG-22)");
            assert_eq!(model.sections[section].fields[field].id, "BT-106");
        }
        other => panic!("{other:?}"),
    }
    assert!(
        validation
            .findings
            .iter()
            .any(|finding| finding.rule_id != SCHEMA_RULE_ID),
        "the business rules run as well: {:#?}",
        validation.findings
    );
}

// AC4
#[test]
fn missing_mandatory_element_in_ubl() {
    let validation = validate(&variant("a-ubl-without-bt1.xml"));
    let findings = schema_findings(&validation);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("cbc:IssueDate")
                || finding.message.contains("cbc:ID")),
        "{findings:#?}"
    );
}

// AC5: `schema_verdict_equals_kosit` in `validate_reference.rs`, which
// validates the reference invoices once for all comparisons.

// AC6
#[test]
fn minimum_profile_is_schema_checked() {
    let validation = english(&variant("a-minimum.xml"));
    assert_eq!(validation.applied_rule_sets, ["XML schema CII D16B"]);
    assert!(validation.not_applicable_reason.is_some());
    assert_eq!(schema_findings(&validation), Vec::<&Finding>::new());
}
