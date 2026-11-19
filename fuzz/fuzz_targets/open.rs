//! Opening a document from arbitrary bytes (PDF and XML paths, attachments,
//! field model) must return a document or an error, never panic.
#![no_main]

use e_invoice_viewer_core::document;
use e_invoice_viewer_core::i18n::Language;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(document) = document::open(data, Language::English) {
        for attachment in document.attachments() {
            let _ = attachment.proposed_file_name(1);
        }
    }
});
