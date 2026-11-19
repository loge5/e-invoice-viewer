//! Spec 0023 — validating in several threads at once (the app validates in
//! glib's thread pool, one job per opened file). A test file of its own, so
//! that the schemas are compiled for the first time while the threads run.
//! With libxml2 2.9 (Ubuntu 24.04) threads compiling the schemas at the same
//! time hung in libxml2's catalog lock (CI, `attribution.rs`); the catalogs
//! are switched off since. The hang depends on timing, so this test may pass
//! without the fix; it guards the result of parallel validations.

mod common;

use std::sync::{Arc, Barrier, mpsc};
use std::time::Duration;

use common::{fixture, replace_once};
use e_invoice_viewer_core::i18n::Language;
use e_invoice_viewer_core::validate::Validator;

/// Valid invoices and invoices with schema errors, CII and UBL.
fn samples() -> Vec<String> {
    let cii_a = fixture("cii/sample-a.xml");
    let cii_broken = replace_once(
        &cii_a,
        "<rsm:ExchangedDocument>",
        "<rsm:ExchangedDocument><ram:Unexpected/>",
    );
    vec![
        cii_a,
        fixture("ubl/sample-a.xml"),
        cii_broken,
        fixture("fuzz/validate-timeout-5c0933ae.xml"),
        fixture("fuzz/validate-timeout-a782dc5b.xml"),
    ]
}

/// The rule ids of the findings, in order.
fn rule_ids(xml: &str) -> Vec<String> {
    Validator::new()
        .validate(xml.as_bytes(), Language::German)
        .expect("validation")
        .findings
        .into_iter()
        .map(|finding| finding.rule_id)
        .collect()
}

#[test]
fn threads_validate_at_the_same_time() {
    let samples = Arc::new(samples());
    let threads = 8;
    let mut results = vec![Vec::new(); samples.len()];
    // Fresh threads every round: each compiles the schemas again.
    for _ in 0..4 {
        let start = Arc::new(Barrier::new(threads));
        let (done, finished) = mpsc::channel();
        for index in 0..threads {
            let (samples, start, done) = (samples.clone(), start.clone(), done.clone());
            std::thread::spawn(move || {
                let sample = index % samples.len();
                start.wait();
                done.send((sample, rule_ids(&samples[sample])))
                    .expect("send");
            });
        }
        for _ in 0..threads {
            let (sample, ids) = finished
                .recv_timeout(Duration::from_secs(120))
                .expect("every thread finishes within 2 minutes");
            results[sample] = ids;
        }
    }
    // The same findings as one validation after the other.
    for (sample, xml) in samples.iter().enumerate() {
        assert_eq!(results[sample], rule_ids(xml), "sample {sample}");
    }
    assert!(results[2].iter().any(|id| id == "XSD"), "{:?}", results[2]);
}
