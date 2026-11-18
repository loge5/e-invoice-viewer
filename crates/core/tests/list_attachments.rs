//! Spec 0013 — list the attachments of an invoice.

mod common;

use common::{Attachment, fixture, pdf_document, pdf_with, pdf_with_text, replace_once};
use e_invoice_viewer_core::attachments::{self, Content, Source};
use e_invoice_viewer_core::document;
use lopdf::Object;

const LIEFERSCHEIN_SIZE: usize = 24_576;

/// CII sample A with the given BG-24 entries (XML of
/// `ram:AdditionalReferencedDocument` elements).
fn cii_a_with(documents: &str) -> String {
    replace_once(
        &fixture("cii/sample-a.xml"),
        "    </ram:ApplicableHeaderTradeAgreement>",
        &format!("{documents}    </ram:ApplicableHeaderTradeAgreement>"),
    )
}

/// `stunden.csv` and its base64 encoding.
const STUNDEN_CSV: &[u8] = b"Datum;Stunden\n17.11.2026;8\n";
const STUNDEN_CSV_BASE64: &str = "RGF0dW07U3R1bmRlbgoxNy4xMS4yMDI2OzgK";

fn stunden_entry(binary: &str) -> String {
    format!(
        "      <ram:AdditionalReferencedDocument>\n\
         \x20       <ram:IssuerAssignedID>Stundennachweis</ram:IssuerAssignedID>\n\
         \x20       <ram:TypeCode>916</ram:TypeCode>\n\
         \x20       {binary}\n\
         \x20     </ram:AdditionalReferencedDocument>\n"
    )
}

fn lieferschein() -> Vec<u8> {
    vec![b'%'; LIEFERSCHEIN_SIZE]
}

// AC1
#[test]
fn pdf_attachment_listed_without_invoice_xml() {
    let xml = fixture("cii/sample-a.xml");
    let content = lieferschein();
    let pdf = pdf_with(&[
        Attachment::plain("factur-x.xml", xml.as_bytes()),
        Attachment::plain("lieferschein.pdf", &content)
            .with_mime_type("application/pdf")
            .with_description("Lieferschein 4711"),
    ]);

    let list = attachments::list(&pdf);

    assert_eq!(list.len(), 1, "{list:#?}");
    let entry = &list[0];
    assert_eq!(entry.name.as_deref(), Some("lieferschein.pdf"));
    assert_eq!(entry.source, Source::Pdf);
    assert_eq!(entry.mime_type.as_deref(), Some("application/pdf"));
    assert_eq!(entry.description.as_deref(), Some("Lieferschein 4711"));
    assert_eq!(entry.content, Content::Embedded(content));
}

// AC1
#[test]
fn pdf_without_invoice_lists_attachments() {
    let content = lieferschein();
    let pdf = pdf_with(&[Attachment::compressed("lieferschein.pdf", &content)]);
    assert!(matches!(
        document::open(&pdf),
        Ok(document::Document::Pdf { invoice: None })
    ));

    let list = attachments::list(&pdf);

    assert_eq!(list.len(), 1);
    assert_eq!(list[0].content, Content::Embedded(content));
}

// AC1
#[test]
fn second_xml_is_listed() {
    let xml = fixture("cii/sample-a.xml");
    let pdf = pdf_with(&[
        Attachment::plain("factur-x.xml", xml.as_bytes()),
        Attachment::plain("xrechnung.xml", xml.as_bytes()),
    ]);

    let names: Vec<_> = attachments::list(&pdf)
        .into_iter()
        .map(|entry| entry.name)
        .collect();

    assert_eq!(names, [Some("xrechnung.xml".to_string())]);
}

// AC2
#[test]
fn bg24_attachment_listed() {
    let xml = cii_a_with(&stunden_entry(&format!(
        "<ram:AttachmentBinaryObject mimeCode=\"text/csv\" filename=\"stunden.csv\">{STUNDEN_CSV_BASE64}</ram:AttachmentBinaryObject>"
    )));

    let list = attachments::list(xml.as_bytes());

    assert_eq!(list.len(), 1, "{list:#?}");
    let entry = &list[0];
    assert_eq!(entry.name.as_deref(), Some("stunden.csv"));
    assert_eq!(entry.source, Source::Invoice);
    assert_eq!(entry.mime_type.as_deref(), Some("text/csv"));
    assert_eq!(entry.description.as_deref(), Some("Stundennachweis"));
}

// AC3
#[test]
fn no_attachments() {
    assert!(attachments::list(fixture("cii/sample-a.xml").as_bytes()).is_empty());
    let xml = fixture("cii/sample-a.xml");
    assert!(
        attachments::list(&pdf_with(&[Attachment::plain(
            "factur-x.xml",
            xml.as_bytes()
        )]))
        .is_empty()
    );
}

// AC4
#[test]
fn decoded_content_is_byte_identical() {
    // Line breaks inside base64 are allowed.
    let wrapped = format!(
        "{}\n          {}",
        &STUNDEN_CSV_BASE64[..16],
        &STUNDEN_CSV_BASE64[16..]
    );
    let xml = cii_a_with(&stunden_entry(&format!(
        "<ram:AttachmentBinaryObject mimeCode=\"text/csv\" filename=\"stunden.csv\">{wrapped}</ram:AttachmentBinaryObject>"
    )));

    let list = attachments::list(xml.as_bytes());

    assert_eq!(list[0].content, Content::Embedded(STUNDEN_CSV.to_vec()));
    assert_eq!(list[0].proposed_file_name(1), "stunden.csv");
}

// AC5
#[test]
fn proposed_save_names() {
    let entry = |name: Option<&str>| attachments::Attachment {
        name: name.map(str::to_string),
        source: Source::Pdf,
        mime_type: None,
        description: None,
        content: Content::Embedded(Vec::new()),
    };
    let spoofed = "rechnung\u{202E}fdp.exe";

    assert_eq!(
        entry(Some("../../.bashrc")).proposed_file_name(1),
        ".bashrc"
    );
    assert_eq!(
        entry(Some("C:\\Temp\\stunden.csv")).proposed_file_name(1),
        "stunden.csv"
    );
    assert_eq!(
        entry(Some(spoofed)).proposed_file_name(1),
        "rechnungfdp.exe"
    );
    assert_eq!(entry(Some("..")).proposed_file_name(2), "Anhang-2");
    assert_eq!(entry(None).proposed_file_name(2), "Anhang-2");
    assert_eq!(
        entry(Some("a\u{0}b\nc.txt")).proposed_file_name(1),
        "abc.txt"
    );

    assert_eq!(entry(Some(spoofed)).display_name(1), "rechnungfdp.exe");
    assert_eq!(entry(None).display_name(3), "Anhang-3");
}

// AC6
#[test]
fn external_reference_has_no_content() {
    let xml = cii_a_with(
        "      <ram:AdditionalReferencedDocument>\n\
         \x20       <ram:IssuerAssignedID>Stundennachweis</ram:IssuerAssignedID>\n\
         \x20       <ram:URIID>https://example.com/stunden.csv</ram:URIID>\n\
         \x20       <ram:TypeCode>916</ram:TypeCode>\n\
         \x20     </ram:AdditionalReferencedDocument>\n",
    );

    let list = attachments::list(xml.as_bytes());

    assert_eq!(list.len(), 1);
    assert_eq!(
        list[0].content,
        Content::External("https://example.com/stunden.csv".to_string())
    );
}

// AC7
#[test]
fn invalid_base64_and_broken_stream_are_unreadable() {
    let xml = cii_a_with(&stunden_entry(
        "<ram:AttachmentBinaryObject mimeCode=\"text/csv\" filename=\"stunden.csv\">not*base64!</ram:AttachmentBinaryObject>",
    ));
    assert_eq!(
        attachments::list(xml.as_bytes())[0].content,
        Content::Unreadable
    );
    assert!(document::open(xml.as_bytes()).is_ok());

    let invoice = fixture("cii/sample-a.xml");
    let mut doc = pdf_document(&[
        Attachment::plain("factur-x.xml", invoice.as_bytes()),
        Attachment::plain("lieferschein.pdf", b"placeholder"),
    ]);
    for object in doc.objects.values_mut() {
        if let Object::Stream(stream) = object
            && stream.content == b"placeholder"
        {
            stream
                .dict
                .set("Filter", Object::Name(b"FlateDecode".to_vec()));
            stream.set_content(b"definitely not deflate data".to_vec());
        }
    }
    let mut pdf = Vec::new();
    doc.save_to(&mut pdf).expect("save PDF");

    assert_eq!(attachments::list(&pdf)[0].content, Content::Unreadable);
    assert!(matches!(
        document::open(&pdf),
        Ok(document::Document::Pdf { invoice: Some(_) })
    ));
}

// AC8
#[test]
fn oversized_attachment() {
    let huge = vec![0_u8; 64 * 1024 * 1024 + 1];
    let pdf = pdf_with(&[Attachment::compressed("gross.bin", &huge)]);

    assert_eq!(attachments::list(&pdf)[0].content, Content::TooLarge);
}

// AC9
#[test]
fn missing_mime_type_is_unknown() {
    let pdf = pdf_with(&[Attachment::plain("notiz.txt", b"Hallo")]);
    let xml = cii_a_with(&stunden_entry(&format!(
        "<ram:AttachmentBinaryObject filename=\"stunden.csv\">{STUNDEN_CSV_BASE64}</ram:AttachmentBinaryObject>"
    )));

    assert_eq!(attachments::list(&pdf)[0].mime_type, None);
    assert_eq!(attachments::list(xml.as_bytes())[0].mime_type, None);
}

// Spec 0013 + 0006: BG-24 of UBL invoices is listed like CII
#[test]
fn ubl_bg24_attachment_listed() {
    let xml = replace_once(
        &fixture("ubl/sample-a.xml"),
        "  <cac:AccountingSupplierParty>",
        &format!(
            "  <cac:AdditionalDocumentReference>\n\
             \x20   <cbc:ID>Stundennachweis</cbc:ID>\n\
             \x20   <cac:Attachment>\n\
             \x20     <cbc:EmbeddedDocumentBinaryObject mimeCode=\"text/csv\" filename=\"stunden.csv\">{STUNDEN_CSV_BASE64}</cbc:EmbeddedDocumentBinaryObject>\n\
             \x20   </cac:Attachment>\n\
             \x20 </cac:AdditionalDocumentReference>\n\
             \x20 <cac:AccountingSupplierParty>"
        ),
    );

    let list = attachments::list(xml.as_bytes());

    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name.as_deref(), Some("stunden.csv"));
    assert_eq!(list[0].description.as_deref(), Some("Stundennachweis"));
    assert_eq!(list[0].content, Content::Embedded(STUNDEN_CSV.to_vec()));
}

/// Writes the samples of the manual checks of this spec to
/// `target/manual-0013/`.
#[test]
#[ignore = "writes the samples for the manual checks of spec 0013"]
fn write_manual_samples() {
    let dir = format!("{}/../../target/manual-0013", env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(&dir).expect("create dir");
    let invoice = fixture("cii/sample-a.xml");
    // A real PDF, so that the saved attachment can be opened.
    let content = pdf_with_text("Lieferschein 4711", &[]);
    let with_attachment = pdf_with_text(
        "Rechnung 471102 (Beispiel A)",
        &[
            Attachment::plain("factur-x.xml", invoice.as_bytes()),
            Attachment::plain("lieferschein.pdf", &content)
                .with_mime_type("application/pdf")
                .with_description("Lieferschein 4711"),
        ],
    );
    let without_invoice = pdf_with_text(
        "PDF ohne Rechnung",
        &[Attachment::compressed("lieferschein.pdf", &content).with_mime_type("application/pdf")],
    );
    let bg24 = cii_a_with(&format!(
        "{}{}{}",
        stunden_entry(&format!(
            "<ram:AttachmentBinaryObject mimeCode=\"text/csv\" filename=\"../../stunden.csv\">{STUNDEN_CSV_BASE64}</ram:AttachmentBinaryObject>"
        )),
        "      <ram:AdditionalReferencedDocument>\n\
         \x20       <ram:IssuerAssignedID>Online-Nachweis</ram:IssuerAssignedID>\n\
         \x20       <ram:URIID>https://example.com/stunden.csv</ram:URIID>\n\
         \x20       <ram:TypeCode>916</ram:TypeCode>\n\
         \x20     </ram:AdditionalReferencedDocument>\n",
        stunden_entry(
            "<ram:AttachmentBinaryObject mimeCode=\"text/csv\" filename=\"kaputt.csv\">not*base64!</ram:AttachmentBinaryObject>"
        ),
    ));
    for (name, bytes) in [
        ("a-mit-lieferschein.pdf", with_attachment),
        ("pdf-ohne-rechnung.pdf", without_invoice),
        ("a-mit-bg24.xml", bg24.into_bytes()),
        ("a.xml", invoice.into_bytes()),
    ] {
        std::fs::write(format!("{dir}/{name}"), bytes).expect("write sample");
    }
}
