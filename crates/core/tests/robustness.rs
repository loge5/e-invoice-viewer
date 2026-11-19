//! Spec 0025 — no crash and no hang on hostile input. Inputs found by the
//! fuzzer (see `fuzz/README.md`) are added here as regression tests.

use std::time::{Duration, Instant};

use e_invoice_viewer_core::document;
use e_invoice_viewer_core::i18n::Language;
use e_invoice_viewer_core::link::XmlLayout;
use e_invoice_viewer_core::validate::Validator;

/// Opening, validating and laying out `input` each end within 5 s, with a
/// result or an error, and without a panic.
fn survives(name: &str, input: &[u8]) {
    for (step, run) in [
        (
            "open",
            &(|| drop(document::open(input, Language::English))) as &dyn Fn(),
        ),
        ("validate", &|| {
            drop(Validator::new().validate(input, Language::English));
        }),
        ("layout", &|| drop(XmlLayout::new(input))),
    ] {
        let started = Instant::now();
        run();
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "{name}: {step} took {:?}",
            started.elapsed()
        );
    }
}

// AC3
#[test]
fn deep_nesting_is_refused_quickly() {
    let depth = 100_000;
    let mut xml = String::from("<?xml version=\"1.0\"?>");
    xml.push_str(&"<a>".repeat(depth));
    xml.push_str(&"</a>".repeat(depth));
    survives("deep nesting", xml.as_bytes());
    assert!(document::open(xml.as_bytes(), Language::English).is_err());
}

#[test]
fn deep_nesting_inside_an_invoice_is_refused_quickly() {
    let depth = 100_000;
    let xml = format!(
        "<rsm:CrossIndustryInvoice xmlns:rsm=\"urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100\">{}{}</rsm:CrossIndustryInvoice>",
        "<rsm:x>".repeat(depth),
        "</rsm:x>".repeat(depth)
    );
    survives("deep invoice", xml.as_bytes());
    let error = document::open(xml.as_bytes(), Language::English).expect_err("too deep");
    assert!(
        error.to_string().contains("nested more than 256 levels"),
        "{error}"
    );
}

#[test]
fn empty_and_garbage_input() {
    for (name, input) in [
        ("empty", &b""[..]),
        ("nul bytes", &[0u8; 64][..]),
        ("pdf header only", &b"%PDF-1.7\n"[..]),
        ("xml declaration only", &b"<?xml version=\"1.0\"?>"[..]),
    ] {
        survives(name, input);
    }
}

// AC5: found by the `validate` target (timeout under the fuzzer's
// instrumentation): sample A (UBL) with broken amounts. The first validation
// with schema errors took 10 s instrumented (350 ms natively), because the
// error paths were only warmed up then; it is a valid result, not a hang.
#[test]
fn fuzz_validate_timeout_5c0933ae() {
    let input = include_bytes!("fixtures/fuzz/validate-timeout-5c0933ae.xml");
    survives("validate-timeout-5c0933ae", input);
    let validator = Validator::new();
    let started = Instant::now();
    let validation = validator
        .validate(input, Language::English)
        .expect("validation");
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
    assert!(
        validation.findings.iter().any(|f| f.rule_id == "XSD"),
        "{:#?}",
        validation.findings
    );
}
