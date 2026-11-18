//! Spec 0007 — deciding what to show for an opened file (core part).

mod common;

use common::{
    Attachment, fixture, pdf_with, pdf_with_text, pdf_without_page_tree, png_bytes,
    sample_a_latin1, sample_cii_with_lines, sample_l_cii, single_line,
};
use e_invoice_viewer_core::detect::DetectError;
use e_invoice_viewer_core::document::{Document, OpenError, open};
use e_invoice_viewer_core::pdf::ExtractError;

fn sample_a_pdf() -> Vec<u8> {
    let xml = fixture("cii/sample-a.xml");
    pdf_with_text(
        "Rechnung RE-2026-0042",
        &[Attachment::compressed("factur-x.xml", xml.as_bytes())],
    )
}

// AC2
#[test]
fn pdf_with_invoice_yields_pdf_and_fields() {
    let Document::Pdf {
        invoice: Some(invoice),
        ..
    } = open(&sample_a_pdf()).expect("document")
    else {
        panic!("expected a PDF with invoice");
    };

    assert_eq!(invoice.xml, fixture("cii/sample-a.xml").as_bytes());
    assert_eq!(invoice.fields.sections[0].fields[0].display, "RE-2026-0042");
}

// AC3
#[test]
fn plain_xml_yields_fields_without_pdf() {
    let xml = fixture("cii/sample-a.xml");

    let Document::Xml { invoice } = open(xml.as_bytes()).expect("document") else {
        panic!("expected a plain XML invoice");
    };

    assert_eq!(invoice.xml, xml.as_bytes());
    assert_eq!(invoice.fields.sections[0].fields[0].display, "RE-2026-0042");
}

// AC6, row 1
#[test]
fn pdf_without_invoice_yields_pdf_without_fields() {
    assert_eq!(
        open(&pdf_with(&[])).expect("document"),
        Document::Pdf {
            invoice: None,
            files: Vec::new()
        }
    );
}

// AC6, row 3
#[test]
fn unreadable_inputs_yield_a_reason() {
    assert_eq!(
        open(&png_bytes()).unwrap_err(),
        OpenError::Xml(DetectError::NotWellFormed { line: 1 }.into())
    );
    assert_eq!(
        open(b"<a><b></a>").unwrap_err(),
        OpenError::Xml(DetectError::NotWellFormed { line: 1 }.into())
    );
    assert_eq!(
        open(b"<order><id>1</id></order>").unwrap_err(),
        OpenError::Xml(DetectError::NotAnInvoice.into())
    );
    assert_eq!(
        open(b"%PDF-1.7\nnot really a PDF").unwrap_err(),
        OpenError::Pdf(ExtractError::NotReadablePdf)
    );
}

/// Writes the files for the manual checks of specs 0007 and 0008 to
/// `target/manual-fixtures/`. Run with
/// `cargo test -p e-invoice-viewer-core --test open_document -- --ignored`.
#[test]
#[ignore = "generates files for manual checks"]
fn generate_manual_fixtures() {
    let dir = format!(
        "{}/../../target/manual-fixtures",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::create_dir_all(&dir).expect("create directory");
    let xml = fixture("cii/sample-a.xml");
    let files: [(&str, Vec<u8>); 8] = [
        (
            "RE-2026-0500-einzeilig.xml",
            single_line(&sample_l_cii()).into_bytes(),
        ),
        ("RE-2026-0042-latin1.xml", sample_a_latin1()),
        (
            "RE-2026-2mb.xml",
            single_line(&sample_cii_with_lines(3000)).into_bytes(),
        ),
        ("RE-2026-0042.pdf", sample_a_pdf()),
        ("RE-2026-0042.xml", xml.clone().into_bytes()),
        (
            "keine-rechnung.pdf",
            pdf_with_text("Gescannte Rechnung ohne XML", &[]),
        ),
        (
            "kaputte-seiten.pdf",
            pdf_without_page_tree(&[Attachment::plain("factur-x.xml", xml.as_bytes())]),
        ),
        ("bild.png", png_bytes()),
    ];
    for (name, content) in files {
        std::fs::write(format!("{dir}/{name}"), content).expect("write fixture");
    }
}

/// Writes the multi-page PDFs of the manual checks of spec 0016 to
/// `target/manual-0016/`.
#[test]
#[ignore = "writes the samples for the manual checks of spec 0016"]
fn write_manual_samples_0016() {
    let dir = format!("{}/../../target/manual-0016", env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(&dir).expect("create dir");
    let invoice = common::fixture("cii/sample-a.xml");
    for count in [3, 50] {
        let pdf = common::multi_page_pdf(
            count,
            &[common::Attachment::plain(
                "factur-x.xml",
                invoice.as_bytes(),
            )],
        );
        std::fs::write(format!("{dir}/{count}-seiten.pdf"), pdf).expect("write sample");
    }
}
