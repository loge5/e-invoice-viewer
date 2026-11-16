//! Spec 0002 — Extract the invoice XML from a ZUGFeRD / Factur-X PDF.

mod common;

use std::time::{Duration, Instant};

use common::{Attachment, SAMPLE_XML, large_pdf, password_protected_pdf, pdf_with, png_bytes};
use e_invoice_viewer_core::pdf::{ExtractError, extract_invoice_xml};

fn assert_extracts(embedded_name: &str) {
    let pdf = pdf_with(&[Attachment::plain(embedded_name, SAMPLE_XML)]);

    let invoice = extract_invoice_xml(&pdf).expect("invoice XML");

    assert_eq!(invoice.xml, SAMPLE_XML);
    assert_eq!(invoice.source_name, embedded_name);
}

// AC1
#[test]
fn returns_factur_x_xml_byte_identical() {
    assert_extracts("factur-x.xml");
}

// AC1
#[test]
fn matches_name_case_insensitively_and_keeps_stored_name() {
    assert_extracts("Factur-X.xml");
}

// AC1
#[test]
fn returns_xrechnung_xml() {
    assert_extracts("xrechnung.xml");
}

// AC1
#[test]
fn returns_zugferd_invoice_xml_with_stored_case() {
    assert_extracts("ZUGFeRD-invoice.xml");
}

// AC2
#[test]
fn prefers_factur_x_over_xrechnung_and_ignores_other_attachments() {
    let xrechnung = b"<xrechnung/>";
    let pdf = pdf_with(&[
        Attachment::plain("xrechnung.xml", xrechnung),
        Attachment::plain("factur-x.xml", SAMPLE_XML),
        Attachment::plain("lieferschein.pdf", b"%PDF-1.7 delivery note"),
    ]);

    let invoice = extract_invoice_xml(&pdf).expect("invoice XML");

    assert_eq!(invoice.xml, SAMPLE_XML);
    assert_eq!(invoice.source_name, "factur-x.xml");
}

// AC3
#[test]
fn decompresses_flate_encoded_embedded_invoice() {
    let pdf = pdf_with(&[Attachment::compressed("factur-x.xml", SAMPLE_XML)]);

    let invoice = extract_invoice_xml(&pdf).expect("invoice XML");

    assert_eq!(invoice.xml, SAMPLE_XML);
}

// AC4
#[test]
fn pdf_without_embedded_files_reports_no_invoice_and_empty_list() {
    let pdf = pdf_with(&[]);

    let error = extract_invoice_xml(&pdf).unwrap_err();

    assert_eq!(
        error,
        ExtractError::NoEmbeddedInvoice {
            embedded_files: vec![]
        }
    );
}

// AC5
#[test]
fn non_standard_xml_name_is_not_the_invoice_but_listed() {
    let pdf = pdf_with(&[Attachment::plain("rechnung.xml", SAMPLE_XML)]);

    let error = extract_invoice_xml(&pdf).unwrap_err();

    assert_eq!(
        error,
        ExtractError::NoEmbeddedInvoice {
            embedded_files: vec!["rechnung.xml".to_string()]
        }
    );
}

// AC6
#[test]
fn empty_embedded_invoice_is_reported_with_source_name() {
    let pdf = pdf_with(&[Attachment::plain("factur-x.xml", b"")]);

    let error = extract_invoice_xml(&pdf).unwrap_err();

    assert_eq!(
        error,
        ExtractError::EmptyInvoice {
            source_name: "factur-x.xml".to_string()
        }
    );
}

// AC7
#[test]
fn png_bytes_are_not_a_readable_pdf() {
    assert_eq!(
        extract_invoice_xml(&png_bytes()).unwrap_err(),
        ExtractError::NotReadablePdf
    );
}

// AC7
#[test]
fn truncated_pdf_is_not_a_readable_pdf() {
    let pdf = large_pdf();
    assert!(pdf.len() > 4096, "test PDF must be longer than 4 KB");

    assert_eq!(
        extract_invoice_xml(&pdf[..1024]).unwrap_err(),
        ExtractError::NotReadablePdf
    );
}

// AC7
#[test]
fn password_protected_pdf_is_not_a_readable_pdf() {
    assert_eq!(
        extract_invoice_xml(&password_protected_pdf()).unwrap_err(),
        ExtractError::NotReadablePdf
    );
}

// AC7 (time limit)
#[test]
fn unreadable_inputs_return_within_two_seconds() {
    let inputs = [
        png_bytes(),
        large_pdf()[..1024].to_vec(),
        password_protected_pdf(),
    ];

    for input in inputs {
        let start = Instant::now();
        let result = extract_invoice_xml(&input);
        let elapsed = start.elapsed();

        assert!(result.is_err());
        assert!(
            elapsed < Duration::from_secs(2),
            "took {elapsed:?}, limit is 2 s"
        );
    }
}
