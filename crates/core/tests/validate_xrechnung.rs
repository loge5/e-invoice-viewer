//! Spec 0010 — validating XRechnung invoices against the XRechnung CIUS rules.

mod common;

use common::fixture;
use common::validation::{
    CII_RULES, CII_SCHEMA, UBL_RULES, UBL_SCHEMA, XRECHNUNG_RULES, errors, findings_with,
    is_xrechnung_rule, validate, variant,
};
use e_invoice_viewer_core::validate::{Finding, Severity};

// AC1
#[test]
fn sample_b_cii_has_no_errors() {
    let validation = validate(&fixture("cii/sample-b.xml"));

    assert_eq!(errors(&validation), Vec::<&Finding>::new());
    assert_eq!(
        validation.applied_rule_sets,
        [CII_SCHEMA, CII_RULES, XRECHNUNG_RULES]
    );
    assert_eq!(validation.version_note, None);
}

// AC2
#[test]
fn missing_bt10_reports_br_de_15() {
    let validation = validate(&variant("b-without-bt10.xml"));

    let br_de_15 = findings_with(&validation, "BR-DE-15");
    assert_eq!(br_de_15.len(), 1, "{:#?}", validation.findings);
    assert_eq!(br_de_15[0].severity, Severity::Error);
    assert_eq!(
        br_de_15[0].message,
        "[BR-DE-15] Das Element \"Buyer reference\" (BT-10) muss übermittelt werden."
    );
    assert_eq!(br_de_15[0].business_terms, ["BT-10"]);
}

// AC3
#[test]
fn missing_payment_means_reports_br_de_1() {
    let validation = validate(&variant("b-without-payment-means.xml"));

    let br_de_1 = findings_with(&validation, "BR-DE-1");
    assert_eq!(br_de_1.len(), 1, "{:#?}", validation.findings);
    assert_eq!(br_de_1[0].severity, Severity::Error);
}

// AC4
#[test]
fn sample_a_gets_no_xrechnung_rules() {
    let validation = validate(&fixture("cii/sample-a.xml"));

    let xrechnung: Vec<&str> = validation
        .findings
        .iter()
        .map(|finding| finding.rule_id.as_str())
        .filter(|id| is_xrechnung_rule(id))
        .collect();
    assert_eq!(xrechnung, Vec::<&str>::new());
    assert_eq!(validation.applied_rule_sets, [CII_SCHEMA, CII_RULES]);
}

// AC5
#[test]
fn sample_b_ubl_has_no_errors() {
    let validation = validate(&fixture("ubl/sample-b.xml"));

    assert_eq!(errors(&validation), Vec::<&Finding>::new());
    assert_eq!(
        validation.applied_rule_sets,
        [UBL_SCHEMA, UBL_RULES, XRECHNUNG_RULES]
    );
}

// AC6
#[test]
fn declared_version_2_3_uses_bundled_rules() {
    let validation = validate(&variant("b-version-2-3.xml"));

    assert_eq!(
        validation.applied_rule_sets,
        [CII_SCHEMA, CII_RULES, XRECHNUNG_RULES]
    );
    assert_eq!(
        validation.version_note.as_deref(),
        Some("Geprüft nach XRechnung 3.0.2, Rechnung erklärt Version 2.3")
    );
    let br_de_21 = findings_with(&validation, "BR-DE-21");
    assert_eq!(br_de_21.len(), 1, "{:#?}", validation.findings);
    assert_eq!(br_de_21[0].severity, Severity::Warning);
}

// AC7
#[test]
fn cii_sr_465_severity_follows_scenario() {
    let xrechnung = validate(&fixture("kosit/cii/cii-peppol-en16931-r040.xml"));
    let sr465 = findings_with(&xrechnung, "CII-SR-465");
    assert_eq!(sr465.len(), 1, "{:#?}", xrechnung.findings);
    assert_eq!(sr465[0].severity, Severity::Error);

    let en16931 = validate(&variant("r040-en16931.xml"));
    let sr465 = findings_with(&en16931, "CII-SR-465");
    assert_eq!(sr465.len(), 1, "{:#?}", en16931.findings);
    assert_eq!(sr465[0].severity, Severity::Warning);
}

// Requirement: Extension and CVD rules are applied, decided by BT-24
#[test]
fn extension_and_cvd_get_xrechnung_rules() {
    let extension = validate(&variant("b-extension.xml"));
    assert_eq!(
        extension.applied_rule_sets,
        [CII_SCHEMA, CII_RULES, XRECHNUNG_RULES]
    );
    assert_eq!(extension.version_note, None);
    assert_eq!(
        findings_with(&extension, "BR-DE-21"),
        Vec::<&Finding>::new()
    );

    let cvd = validate(&variant("b-cvd.xml"));
    assert_eq!(
        cvd.applied_rule_sets,
        [CII_SCHEMA, CII_RULES, XRECHNUNG_RULES]
    );
    assert_eq!(cvd.version_note, None);
    assert_eq!(
        findings_with(&cvd, "BR-DE-CVD-01").len(),
        1,
        "{:#?}",
        cvd.findings
    );
}

// Contract: message texts with <name/> are filled in as by the KoSIT
// validator
#[test]
fn placeholders_in_messages_are_filled() {
    let validation = validate(&variant("b-cvd.xml"));

    let br_de_cvd_01 = findings_with(&validation, "BR-DE-CVD-01");
    assert_eq!(br_de_cvd_01.len(), 1, "{:#?}", validation.findings);
    assert_eq!(br_de_cvd_01[0].message, KOSIT_BR_DE_CVD_01);
}

/// Message of BR-DE-CVD-01 for `b-cvd.xml` in the KoSIT report.
const KOSIT_BR_DE_CVD_01: &str = "[BR-DE-CVD-01] Das Element ram:ApplicableHeaderTradeAgreement \"Contract reference\" (BT-12) muss übermittelt werden.";
