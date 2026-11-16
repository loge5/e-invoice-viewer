//! Domain logic of E-Invoice Viewer.
//!
//! Everything here is GUI-free and unit-testable: reading the invoice source
//! (PDF with embedded XML, or plain XML), detecting syntax and profile, mapping
//! to the field model and validation. See `ai-harness/guidelines/architecture.md`.

pub mod detect;
pub mod pdf;
