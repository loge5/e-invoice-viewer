//! Validating arbitrary bytes (schema and business rules) must return a
//! result or an error, never panic.
#![no_main]

use std::cell::RefCell;

use e_invoice_viewer_core::i18n::Language;
use e_invoice_viewer_core::validate::Validator;
use libfuzzer_sys::fuzz_target;

thread_local! {
    static VALIDATOR: RefCell<Validator> = RefCell::new(Validator::new());
}

const SAMPLES: [&str; 3] = [
    include_str!("../../crates/core/tests/fixtures/cii/sample-b.xml"),
    include_str!("../../crates/core/tests/fixtures/ubl/sample-b.xml"),
    // Schema and rule errors: warms up the paths that build findings.
    include_str!("../../crates/core/tests/fixtures/fuzz/validate-timeout-5c0933ae.xml"),
];

fuzz_target!(
    init: {
        // Compile the rule sets (CEN and XRechnung, CII and UBL) and warm up
        // the error paths once before the first input: that takes seconds and
        // is not what the per-input time limit is about.
        VALIDATOR.with(|validator| {
            for sample in SAMPLES {
                let _ = validator.borrow().validate(sample.as_bytes(), Language::English);
            }
        });
    },
    |data: &[u8]| {
        VALIDATOR.with(|validator| {
            let _ = validator.borrow().validate(data, Language::English);
        });
    }
);
