//! Builders for synthetic test PDFs (no real invoices in the repository).

#![allow(dead_code)]

use lopdf::encryption::{EncryptionState, EncryptionVersion, Permissions};
use lopdf::{Document, Object, Stream, dictionary};

/// A file to embed into a test PDF.
pub struct Attachment<'a> {
    pub name: &'a str,
    pub content: &'a [u8],
    /// Store the stream Flate-encoded instead of uncompressed.
    pub compressed: bool,
}

impl<'a> Attachment<'a> {
    pub fn plain(name: &'a str, content: &'a [u8]) -> Self {
        Self {
            name,
            content,
            compressed: false,
        }
    }

    pub fn compressed(name: &'a str, content: &'a [u8]) -> Self {
        Self {
            name,
            content,
            compressed: true,
        }
    }
}

/// Sample invoice XML (start of CII sample A, enough for byte comparisons).
pub const SAMPLE_XML: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><rsm:CrossIndustryInvoice xmlns:rsm=\"urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100\"><!-- RE-2026-0042 --></rsm:CrossIndustryInvoice>";

/// A one-page PDF document with the given embedded files.
pub fn pdf_document(attachments: &[Attachment]) -> Document {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
    });
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        }),
    );

    let mut name_tree = Vec::new();
    for attachment in attachments {
        let mut stream = Stream::new(
            dictionary! { "Type" => "EmbeddedFile" },
            attachment.content.to_vec(),
        );
        if attachment.compressed {
            stream.compress().expect("compress test stream");
            assert!(
                stream.dict.get(b"Filter").is_ok(),
                "test stream must be Flate-encoded"
            );
        }
        let stream_id = doc.add_object(stream);
        let filespec_id = doc.add_object(dictionary! {
            "Type" => "Filespec",
            "F" => Object::string_literal(attachment.name),
            "UF" => Object::string_literal(attachment.name),
            "EF" => dictionary! { "F" => stream_id },
        });
        name_tree.push(Object::string_literal(attachment.name));
        name_tree.push(filespec_id.into());
    }

    let mut catalog = dictionary! { "Type" => "Catalog", "Pages" => pages_id };
    if !attachments.is_empty() {
        catalog.set(
            "Names",
            dictionary! { "EmbeddedFiles" => dictionary! { "Names" => name_tree } },
        );
    }
    let catalog_id = doc.add_object(catalog);
    doc.trailer.set("Root", catalog_id);
    doc
}

/// Serialised one-page PDF with the given embedded files.
pub fn pdf_with(attachments: &[Attachment]) -> Vec<u8> {
    save(&mut pdf_document(attachments))
}

/// Serialised PDF that requires the user password `geheim` to open.
pub fn password_protected_pdf() -> Vec<u8> {
    let mut doc = pdf_document(&[Attachment::plain("factur-x.xml", SAMPLE_XML)]);
    let file_id = Object::string_literal("e-invoice-viewer-test-id");
    doc.trailer.set("ID", vec![file_id.clone(), file_id]);
    let version = EncryptionVersion::V2 {
        document: &doc,
        owner_password: "owner",
        user_password: "geheim",
        key_length: 128,
        permissions: Permissions::all(),
    };
    let state = EncryptionState::try_from(version).expect("encryption state");
    doc.encrypt(&state).expect("encrypt test PDF");
    save(&mut doc)
}

/// Serialised PDF of at least 4 KB: the invoice is preceded by a 4 KB
/// attachment, so that truncating it cuts through the file body.
pub fn large_pdf() -> Vec<u8> {
    let filler = vec![b'x'; 4096];
    pdf_with(&[
        Attachment::plain("lieferschein.pdf", &filler),
        Attachment::plain("factur-x.xml", SAMPLE_XML),
    ])
}

/// Bytes of a minimal PNG image (signature and IHDR chunk header).
pub fn png_bytes() -> Vec<u8> {
    let mut bytes = vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];
    bytes.extend_from_slice(&[0, 0, 0, 13, b'I', b'H', b'D', b'R']);
    bytes.extend_from_slice(&[0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0]);
    bytes
}

fn save(doc: &mut Document) -> Vec<u8> {
    let mut out = Vec::new();
    doc.save_to(&mut out).expect("save test PDF");
    out
}
