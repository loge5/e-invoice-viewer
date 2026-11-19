//! Laying out arbitrary bytes for the XML view must return a layout or an
//! error, never panic.
#![no_main]

use e_invoice_viewer_core::link::XmlLayout;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(layout) = XmlLayout::new(data) {
        let _ = layout.text().len();
    }
});
