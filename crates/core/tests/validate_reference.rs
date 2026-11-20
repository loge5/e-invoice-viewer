//! Specs 0009 and 0010 — comparison with the official KoSIT validator.
//!
//! All reference invoices are validated once, in parallel, and shared by the
//! tests of this file.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use common::validation::{is_xrechnung_rule, severity_name, variant, variants};
use common::{fixture, select};
use e_invoice_viewer_core::i18n::Language;
use e_invoice_viewer_core::link::XmlLayout;
use e_invoice_viewer_core::validate::{Validation, Validator};

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

fn reference_xml(key: &str) -> String {
    match key.strip_prefix("variants/") {
        Some(name) => variant(name),
        None => fixture(key),
    }
}

fn comma_separated(list: &str) -> impl Iterator<Item = &str> {
    list.split(',').filter(|item| !item.is_empty())
}

/// The EN 16931 reference set (spec 0009): invoice key and the rule IDs the
/// KoSIT validator fired with the EN 16931 rules only. Invoices that fail
/// the XML schema are left out.
fn en16931_expected() -> BTreeMap<String, BTreeSet<String>> {
    let expected: BTreeMap<_, _> = fixture("kosit/expected-en16931.tsv")
        .lines()
        .filter_map(|line| {
            let (key, ids) = line.split_once('\t').expect("key<TAB>ids");
            (ids != "!xsd").then(|| {
                (
                    key.to_string(),
                    comma_separated(ids).map(str::to_string).collect(),
                )
            })
        })
        .collect();
    assert!(
        expected.len() > 300,
        "reference set has only {} invoices",
        expected.len()
    );
    expected
}

/// The XRechnung reference set (spec 0010): the invoices the KoSIT validator
/// assigns to an XRechnung scenario, with the `(rule ID, severity)` pairs of
/// all its findings.
fn xrechnung_expected() -> BTreeMap<String, BTreeSet<(String, String)>> {
    let expected: BTreeMap<_, _> = fixture("kosit/expected-xrechnung.tsv")
        .lines()
        .map(|line| {
            let mut columns = line.split('\t');
            let key = columns.next().expect("key");
            let _scenario = columns.next().expect("scenario");
            let findings = comma_separated(columns.next().unwrap_or_default())
                .map(|pair| {
                    let (id, level) = pair.split_once(':').expect("id:level");
                    (id.to_string(), level.to_string())
                })
                .collect();
            (key.to_string(), findings)
        })
        .collect();
    assert!(
        expected.len() > 200,
        "XRechnung reference set has only {} invoices",
        expected.len()
    );
    expected
}

/// Whether an invoice was checked against a Factur-X profile (spec 0027),
/// which KoSIT does not do: its schema and profile findings differ by design.
fn is_facturx(validation: &Validation) -> bool {
    validation
        .applied_rule_sets
        .iter()
        .any(|set| set.contains("Factur-X"))
}

struct Checked {
    key: String,
    xml: String,
    validation: Validation,
}

/// The reference invoices, validated once. Validators are not `Send`, so
/// each worker thread has its own.
fn checked() -> &'static BTreeMap<String, Checked> {
    static CHECKED: OnceLock<BTreeMap<String, Checked>> = OnceLock::new();
    CHECKED.get_or_init(|| {
        let keys: Vec<String> = en16931_expected().into_keys().collect();
        let threads = std::thread::available_parallelism().map_or(4, usize::from);
        let keys = &keys;
        std::thread::scope(|scope| {
            // Interleaved, so that slow (UBL) and fast invoices are spread.
            let workers: Vec<_> = (0..threads)
                .map(|worker| {
                    scope.spawn(move || {
                        let validator = Validator::new();
                        keys.iter()
                            .skip(worker)
                            .step_by(threads)
                            .map(|key| {
                                let xml = reference_xml(key);
                                let validation = validator
                                    .validate(xml.as_bytes(), Language::German)
                                    .expect(key);
                                Checked {
                                    key: key.clone(),
                                    xml,
                                    validation,
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            workers
                .into_iter()
                .flat_map(|worker| worker.join().expect("worker"))
                .map(|checked| (checked.key.clone(), checked))
                .collect()
        })
    })
}

fn difference<T: Ord + std::fmt::Debug>(
    key: &str,
    expected: &BTreeSet<T>,
    actual: &BTreeSet<T>,
) -> Option<String> {
    (expected != actual).then(|| {
        format!(
            "{key}: missing {:?}, extra {:?}",
            expected.difference(actual).collect::<Vec<_>>(),
            actual.difference(expected).collect::<Vec<_>>()
        )
    })
}

// Spec 0009 AC8: the EN 16931 rule IDs equal the KoSIT result
#[test]
fn matches_kosit_en16931_reference() {
    let checked = checked();
    let differences: Vec<String> = en16931_expected()
        .iter()
        .filter_map(|(key, expected)| {
            let fired: BTreeSet<String> = checked[key]
                .validation
                .findings
                .iter()
                .filter(|finding| !is_xrechnung_rule(&finding.rule_id))
                .filter(|finding| {
                    !is_facturx(&checked[key].validation)
                        || !matches!(finding.rule_id.as_str(), "XSD" | "FX")
                })
                .map(|finding| finding.rule_id.clone())
                .collect();
            difference(key, expected, &fired)
        })
        .collect();
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

// Spec 0010 AC8: XRechnung rule IDs and all severities equal the KoSIT result
#[test]
fn matches_kosit_xrechnung_reference() {
    let checked = checked();
    let differences: Vec<String> = xrechnung_expected()
        .iter()
        .filter_map(|(key, expected)| {
            let Some(checked) = checked.get(key) else {
                return Some(format!("{key}: not in the EN 16931 reference set"));
            };
            let actual: BTreeSet<(String, String)> = checked
                .validation
                .findings
                .iter()
                .map(|finding| {
                    (
                        finding.rule_id.clone(),
                        severity_name(finding.severity).to_string(),
                    )
                })
                .collect();
            difference(key, expected, &actual)
        })
        .collect();
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

// Spec 0009 contract: the location always selects an existing element
#[test]
fn every_location_selects_an_existing_element() {
    let mut checked_findings = 0;
    let mut xrechnung_findings = 0;
    for checked in checked().values() {
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
            xrechnung_findings += usize::from(is_xrechnung_rule(&finding.rule_id));
        }
    }
    assert!(
        checked_findings > 100,
        "only {checked_findings} findings checked"
    );
    assert!(
        xrechnung_findings > 50,
        "only {xrechnung_findings} XRechnung findings checked"
    );
}

// Spec 0014 contract: finding locations are written like field and XML
// locations, so every finding can be shown in the XML view.
#[test]
fn every_location_is_found_in_the_xml_view() {
    for checked in checked().values() {
        let layout = XmlLayout::new(checked.xml.as_bytes()).expect(&checked.key);
        for finding in &checked.validation.findings {
            assert!(
                layout.element_range(&finding.location).is_some(),
                "{}: {} at {}",
                checked.key,
                finding.rule_id,
                finding.location
            );
        }
    }
}

// Spec 0023 AC5: the app reports a schema error exactly where KoSIT does.
#[test]
fn schema_verdict_equals_kosit() {
    use e_invoice_viewer_core::validate::SCHEMA_RULE_ID;
    let has_schema_error = |validation: &Validation| {
        validation
            .findings
            .iter()
            .any(|finding| finding.rule_id == SCHEMA_RULE_ID)
    };
    let mut deviations: Vec<String> = checked()
        .values()
        .filter(|checked| !is_facturx(&checked.validation))
        .filter(|checked| has_schema_error(&checked.validation))
        .map(|checked| format!("{}: schema error, KoSIT none", checked.key))
        .collect();
    let invalid: Vec<String> = fixture("kosit/expected-en16931.tsv")
        .lines()
        .filter_map(|line| line.strip_suffix("\t!xsd").map(str::to_string))
        .collect();
    assert!(
        !invalid.is_empty(),
        "the reference has schema-invalid invoices"
    );
    let validator = Validator::new();
    for key in &invalid {
        let xml = reference_xml(key);
        let validation = validator
            .validate(xml.as_bytes(), Language::German)
            .expect("validation");
        if !has_schema_error(&validation) {
            deviations.push(format!("{key}: no schema error, KoSIT has one"));
        }
    }
    assert!(deviations.is_empty(), "{deviations:#?}");
}
