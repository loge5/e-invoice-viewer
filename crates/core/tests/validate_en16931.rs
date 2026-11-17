//! Spec 0009 — validating an invoice against the EN 16931 business rules.

mod common;

use std::collections::BTreeSet;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use common::{fixture, replace_once, sample_l_cii, select};
use e_invoice_viewer_core::detect::DetectError;
use e_invoice_viewer_core::validate::{Finding, Severity, Validation, Validator};

const CII_RULES: &str = "EN 16931 CII 1.3.17";
const UBL_RULES: &str = "EN 16931 UBL 1.3.17";

fn validate(xml: &str) -> Validation {
    Validator::new()
        .validate(xml.as_bytes())
        .expect("validation result")
}

fn findings_with<'a>(validation: &'a Validation, rule_id: &str) -> Vec<&'a Finding> {
    validation
        .findings
        .iter()
        .filter(|finding| finding.rule_id == rule_id)
        .collect()
}

fn errors(validation: &Validation) -> Vec<&Finding> {
    validation
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .collect()
}

fn with_specification_identifier(id: &str) -> String {
    replace_once(
        &fixture("cii/sample-a.xml"),
        "<ram:ID>urn:cen.eu:en16931:2017</ram:ID>",
        &format!("<ram:ID>{id}</ram:ID>"),
    )
}

/// The variants of the samples this spec describes, by file name. They are
/// part of the KoSIT reference set (see `write_variants`).
fn variants() -> Vec<(&'static str, String)> {
    let a = fixture("cii/sample-a.xml");
    vec![
        (
            "a-without-bt24.xml",
            replace_once(&a, "      <ram:ID>urn:cen.eu:en16931:2017</ram:ID>\n", ""),
        ),
        (
            "a-grand-total-1300.xml",
            replace_once(
                &a,
                "<ram:GrandTotalAmount>1249.50</ram:GrandTotalAmount>",
                "<ram:GrandTotalAmount>1300.00</ram:GrandTotalAmount>",
            ),
        ),
        (
            "a-ubl-without-bt1.xml",
            replace_once(
                &fixture("ubl/sample-a.xml"),
                "  <cbc:ID>RE-2026-0042</cbc:ID>\n",
                "",
            ),
        ),
        (
            "a-minimum.xml",
            with_specification_identifier("urn:factur-x.eu:1p0:minimum"),
        ),
        (
            "a-basicwl.xml",
            with_specification_identifier("urn:factur-x.eu:1p0:basicwl"),
        ),
        (
            "a-extended.xml",
            with_specification_identifier(
                "urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended",
            ),
        ),
        (
            "a-unknown-profile.xml",
            with_specification_identifier("urn:example:unknown-profile"),
        ),
    ]
}

fn variant(name: &str) -> String {
    variants()
        .into_iter()
        .find(|(file, _)| *file == name)
        .map(|(_, xml)| xml)
        .unwrap_or_else(|| panic!("no variant {name}"))
}

/// Writes the variants to `target/validation-variants/` for
/// `scripts/kosit-reference.sh`.
#[test]
#[ignore = "writes the inputs of scripts/kosit-reference.sh"]
fn write_variants() {
    let dir = format!(
        "{}/../../target/validation-variants",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::create_dir_all(&dir).expect("create dir");
    for (name, xml) in variants() {
        std::fs::write(format!("{dir}/{name}"), xml).expect("write variant");
    }
}

/// The reference set: invoice key, its XML and the rule IDs the KoSIT
/// validator fired. Invoices that fail the XML schema are left out.
fn reference_set() -> Vec<(String, String, BTreeSet<String>)> {
    let expected = fixture("kosit/expected-en16931.tsv");
    let set: Vec<_> = expected
        .lines()
        .filter_map(|line| {
            let (key, ids) = line.split_once('\t').expect("key<TAB>ids");
            if ids == "!xsd" {
                return None;
            }
            let xml = match key.strip_prefix("variants/") {
                Some(name) => variant(name),
                None => fixture(key),
            };
            let ids = ids
                .split(',')
                .filter(|id| !id.is_empty())
                .map(str::to_string)
                .collect();
            Some((key.to_string(), xml, ids))
        })
        .collect();
    assert!(
        set.len() > 300,
        "reference set has only {} invoices",
        set.len()
    );
    set
}

/// A reference invoice with its expected rule IDs and the validation result.
struct Checked {
    key: String,
    xml: String,
    expected: BTreeSet<String>,
    validation: Validation,
}

/// The reference set, validated once and shared by the tests that need it.
/// Validators are not `Send`, so each worker thread has its own.
fn checked_reference_set() -> &'static [Checked] {
    static CHECKED: OnceLock<Vec<Checked>> = OnceLock::new();
    CHECKED.get_or_init(|| {
        let set = reference_set();
        let threads = std::thread::available_parallelism().map_or(4, usize::from);
        let set = &set;
        std::thread::scope(|scope| {
            // Interleaved, so that slow (UBL) and fast invoices are spread.
            let workers: Vec<_> = (0..threads)
                .map(|worker| {
                    scope.spawn(move || {
                        let validator = Validator::new();
                        set.iter()
                            .skip(worker)
                            .step_by(threads)
                            .map(|(key, xml, expected)| Checked {
                                key: key.clone(),
                                xml: xml.clone(),
                                expected: expected.clone(),
                                validation: validator.validate(xml.as_bytes()).expect(key),
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            workers
                .into_iter()
                .flat_map(|worker| worker.join().expect("worker"))
                .collect()
        })
    })
}

// AC1
#[test]
fn sample_a_cii_has_no_errors() {
    let validation = validate(&fixture("cii/sample-a.xml"));

    assert_eq!(errors(&validation), Vec::<&Finding>::new());
    assert_eq!(validation.applied_rule_sets, [CII_RULES]);
    assert_eq!(validation.not_applicable_reason, None);
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
    assert_eq!(validation.applied_rule_sets, [UBL_RULES]);
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
    assert_eq!(validation.applied_rule_sets, Vec::<String>::new());
    assert!(validation.not_applicable_reason.is_some());
}

// Requirement: no EN 16931 findings for BASIC WL either
#[test]
fn basic_wl_profile_is_not_applicable() {
    let validation = validate(&variant("a-basicwl.xml"));

    assert_eq!(validation.findings, []);
    assert_eq!(validation.applied_rule_sets, Vec::<String>::new());
    assert!(validation.not_applicable_reason.is_some());
}

// Requirement: rules apply for extended and unknown profiles
#[test]
fn rules_apply_for_extended_and_unknown_profiles() {
    for name in ["a-extended.xml", "a-unknown-profile.xml"] {
        let validation = validate(&variant(name));

        assert_eq!(validation.applied_rule_sets, [CII_RULES], "{name}");
        assert_eq!(validation.not_applicable_reason, None, "{name}");
    }
}

// AC7
#[test]
fn unevaluable_rule_counts_as_violated() {
    let validation = validate(&fixture("kosit/ubl-inv/ubl-inv-peppol-en16931-r053-2.xml"));

    assert_eq!(findings_with(&validation, "BR-CO-15").len(), 1);
}

// AC8
#[test]
fn matches_kosit_reference() {
    let mut differences = Vec::new();
    for checked in checked_reference_set() {
        let fired: BTreeSet<String> = checked
            .validation
            .findings
            .iter()
            .map(|finding| finding.rule_id.clone())
            .collect();
        if fired != checked.expected {
            differences.push(format!(
                "{}: missing {:?}, extra {:?}",
                checked.key,
                checked.expected.difference(&fired).collect::<Vec<_>>(),
                fired.difference(&checked.expected).collect::<Vec<_>>()
            ));
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

// Contract: the location always selects an existing element
#[test]
fn every_location_selects_an_existing_element() {
    let mut checked_findings = 0;
    for checked in checked_reference_set() {
        for finding in &checked.validation.findings {
            assert_eq!(
                select(&checked.xml, &finding.location).len(),
                1,
                "{}: {} at {}",
                checked.key,
                finding.rule_id,
                finding.location
            );
            checked_findings += 1;
        }
    }
    assert!(
        checked_findings > 100,
        "only {checked_findings} findings checked"
    );
}

// Contract: severities follow the official flags
#[test]
fn severities_follow_official_flags() {
    let validation = validate(&fixture("kosit/cii/cii-peppol-en16931-r040.xml"));
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
        validator.validate(b"<x/>").unwrap_err(),
        DetectError::NotAnInvoice
    );
    assert_eq!(
        validator
            .validate(b"<rsm:CrossIndustryInvoice")
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
        .validate(xml.as_bytes())
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
        .validate(xml.as_bytes())
        .expect("validation");
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(5), "took {elapsed:?}");
}
