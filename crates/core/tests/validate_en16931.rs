//! Spec 0009 — validating an invoice against the EN 16931 business rules.

mod common;

use std::time::{Duration, Instant};

use common::validation::{
    CII_RULES, CII_SCHEMA, UBL_RULES, UBL_SCHEMA, errors, findings_with, validate, variant,
};
use common::{fixture, sample_l_cii, select};
use e_invoice_viewer_core::detect::DetectError;
use e_invoice_viewer_core::i18n::Language;
use e_invoice_viewer_core::validate::{Finding, Severity, Validator};

// AC1
#[test]
fn sample_a_cii_has_no_errors() {
    let validation = validate(&fixture("cii/sample-a.xml"));

    assert_eq!(errors(&validation), Vec::<&Finding>::new());
    assert_eq!(validation.applied_rule_sets, [CII_SCHEMA, CII_RULES]);
    assert_eq!(validation.not_applicable_reason, None);
}

// Spec 0021: sample C (references, delivery, direct debit, allowances and
// charges) is a correct invoice.
#[test]
fn sample_c_has_no_errors() {
    for path in ["cii/sample-c.xml", "ubl/sample-c.xml"] {
        let validation = validate(&fixture(path));
        assert_eq!(errors(&validation), Vec::<&Finding>::new(), "{path}");
    }
}

// AC2
#[test]
fn missing_bt24_reports_br01_at_root() {
    let xml = variant("a-without-bt24.xml");
    let validation = validate(&xml);

    let br01 = findings_with(&validation, "BR-01");
    assert_eq!(br01.len(), 1, "{:#?}", validation.findings);
    assert_eq!(br01[0].severity, Severity::Error);
    assert!(br01[0].message.contains("BT-24"), "{}", br01[0].message);
    assert_eq!(select(&xml, &br01[0].location).len(), 1);
    assert!(
        !br01[0].location.trim_start_matches('/').contains('/'),
        "location {} is not the root element",
        br01[0].location
    );
    assert!(br01[0].location.starts_with("/rsm:CrossIndustryInvoice"));
}

// AC3
#[test]
fn wrong_grand_total_reports_br_co_15() {
    let validation = validate(&variant("a-grand-total-1300.xml"));

    let br_co_15 = findings_with(&validation, "BR-CO-15");
    assert_eq!(br_co_15.len(), 1, "{:#?}", validation.findings);
    assert_eq!(br_co_15[0].severity, Severity::Error);
    assert_eq!(br_co_15[0].business_terms, ["BT-112", "BT-109", "BT-110"]);
}

// AC4
#[test]
fn sample_a_ubl_has_no_errors() {
    let validation = validate(&fixture("ubl/sample-a.xml"));

    assert_eq!(errors(&validation), Vec::<&Finding>::new());
    assert_eq!(validation.applied_rule_sets, [UBL_SCHEMA, UBL_RULES]);
}

// AC5
#[test]
fn ubl_missing_bt1_reports_br02() {
    let validation = validate(&variant("a-ubl-without-bt1.xml"));

    let br02 = findings_with(&validation, "BR-02");
    assert_eq!(br02.len(), 1, "{:#?}", validation.findings);
    assert_eq!(br02[0].severity, Severity::Error);
}

// AC6
#[test]
fn minimum_profile_is_not_applicable() {
    let validation = validate(&variant("a-minimum.xml"));

    assert_eq!(validation.findings, []);
    assert_eq!(validation.applied_rule_sets, [CII_SCHEMA]);
    assert!(validation.not_applicable_reason.is_some());
}

// Requirement: no EN 16931 findings for BASIC WL either
#[test]
fn basic_wl_profile_is_not_applicable() {
    let validation = validate(&variant("a-basicwl.xml"));

    assert_eq!(validation.findings, []);
    assert_eq!(validation.applied_rule_sets, [CII_SCHEMA]);
    assert!(validation.not_applicable_reason.is_some());
}

// Requirement: rules apply for extended and unknown profiles
#[test]
fn rules_apply_for_extended_and_unknown_profiles() {
    for name in ["a-extended.xml", "a-unknown-profile.xml"] {
        let validation = validate(&variant(name));

        assert_eq!(
            validation.applied_rule_sets,
            [CII_SCHEMA, CII_RULES],
            "{name}"
        );
        assert_eq!(validation.not_applicable_reason, None, "{name}");
    }
}

// AC7
#[test]
fn unevaluable_rule_counts_as_violated() {
    let validation = validate(&fixture("kosit/ubl-inv/ubl-inv-peppol-en16931-r053-2.xml"));

    assert_eq!(findings_with(&validation, "BR-CO-15").len(), 1);
}

// AC8: see tests/validate_reference.rs

// Contract: severities follow the official flags
#[test]
fn severities_follow_official_flags() {
    // r040 declared as plain EN 16931, so no XRechnung severity changes
    // apply (spec 0010).
    let validation = validate(&variant("r040-en16931.xml"));
    let sr465 = findings_with(&validation, "CII-SR-465");
    assert_eq!(sr465.len(), 1, "{:#?}", validation.findings);
    assert_eq!(sr465[0].severity, Severity::Warning);

    let validation = validate(&variant("a-grand-total-1300.xml"));
    assert_eq!(
        findings_with(&validation, "BR-CO-15")[0].severity,
        Severity::Error
    );
}

// Requirement: validation never crashes
#[test]
fn non_invoice_input_is_an_error_not_a_panic() {
    let validator = Validator::new();

    assert_eq!(
        validator.validate(b"<x/>", Language::German).unwrap_err(),
        DetectError::NotAnInvoice
    );
    assert_eq!(
        validator
            .validate(b"<rsm:CrossIndustryInvoice", Language::German)
            .unwrap_err(),
        DetectError::NotWellFormed { line: 1 }
    );
}

// AC9 (release build only, as the spec states)
#[test]
#[cfg_attr(debug_assertions, ignore = "timing applies to release builds")]
fn validates_sample_a_within_one_second() {
    let xml = fixture("cii/sample-a.xml");
    let start = Instant::now();
    Validator::new()
        .validate(xml.as_bytes(), Language::German)
        .expect("validation");
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(1), "took {elapsed:?}");
}

// AC9 (release build only, as the spec states)
#[test]
#[cfg_attr(debug_assertions, ignore = "timing applies to release builds")]
fn validates_sample_l_within_five_seconds() {
    let xml = sample_l_cii();
    let start = Instant::now();
    Validator::new()
        .validate(xml.as_bytes(), Language::German)
        .expect("validation");
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(5), "took {elapsed:?}");
}
