//! Reading ZUGFeRD / Factur-X PDFs (spec 0002).

use std::panic::{self, AssertUnwindSafe};

use lopdf::{Dictionary, Document, Object};
use thiserror::Error;

/// Names of the embedded invoice XML, in order of precedence (compared
/// case-insensitively).
const INVOICE_FILE_NAMES: [&str; 3] = ["factur-x.xml", "xrechnung.xml", "zugferd-invoice.xml"];

/// Upper bound for a decompressed embedded file, protecting against
/// decompression bombs.
const MAX_EMBEDDED_FILE_SIZE: usize = 64 * 1024 * 1024;

/// Upper bound for the nesting depth of the embedded-files name tree,
/// protecting against malicious or cyclic trees.
const MAX_NAME_TREE_DEPTH: usize = 32;

/// The invoice XML embedded in a PDF.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedInvoice {
    /// File name of the embedded file, as stored in the PDF.
    pub source_name: String,
    /// The XML, byte-identical to the embedded file content.
    pub xml: Vec<u8>,
}

/// Why no invoice XML could be extracted.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ExtractError {
    /// The bytes are not a PDF, are damaged, or need a password to open.
    #[error("not a readable PDF")]
    NotReadablePdf,
    /// The PDF contains no file with one of the invoice file names.
    #[error("no embedded invoice")]
    NoEmbeddedInvoice {
        /// Names of the embedded files that are present, in document order.
        embedded_files: Vec<String>,
    },
    /// The invoice file is present but has no content.
    #[error("embedded invoice is empty: {source_name}")]
    EmptyInvoice { source_name: String },
}

/// Extracts the embedded invoice XML from the bytes of a PDF.
pub fn extract_invoice_xml(pdf: &[u8]) -> Result<EmbeddedInvoice, ExtractError> {
    // The PDF parser must never take the application down, whatever the input.
    let embedded = panic::catch_unwind(AssertUnwindSafe(|| embedded_files(pdf)))
        .map_err(|_| ExtractError::NotReadablePdf)??;

    let invoice = INVOICE_FILE_NAMES.iter().find_map(|wanted| {
        embedded
            .iter()
            .find(|file| file.name.eq_ignore_ascii_case(wanted))
    });

    match invoice {
        Some(file) => match &file.content {
            Some(content) if !content.is_empty() => Ok(EmbeddedInvoice {
                source_name: file.name.clone(),
                xml: content.clone(),
            }),
            Some(_) => Err(ExtractError::EmptyInvoice {
                source_name: file.name.clone(),
            }),
            None => Err(ExtractError::NotReadablePdf),
        },
        None => Err(ExtractError::NoEmbeddedInvoice {
            embedded_files: embedded.into_iter().map(|file| file.name).collect(),
        }),
    }
}

/// An embedded file of the PDF. `content` is `None` if it cannot be read.
struct EmbeddedFile {
    name: String,
    content: Option<Vec<u8>>,
}

fn embedded_files(pdf: &[u8]) -> Result<Vec<EmbeddedFile>, ExtractError> {
    let doc = Document::load_mem(pdf).map_err(|_| ExtractError::NotReadablePdf)?;
    if doc.is_encrypted() {
        // Still encrypted after loading: a user password would be required.
        return Err(ExtractError::NotReadablePdf);
    }

    let catalog = doc.catalog().map_err(|_| ExtractError::NotReadablePdf)?;
    let mut entries = Vec::new();
    if let Some(tree) = resolve_dict(&doc, catalog.get(b"Names").ok())
        .and_then(|names| resolve_dict(&doc, names.get(b"EmbeddedFiles").ok()))
    {
        collect_name_tree(&doc, tree, 0, &mut entries);
    }

    Ok(entries
        .into_iter()
        .map(|(key, filespec)| read_filespec(&doc, &key, filespec))
        .collect())
}

/// Collects the (key, value) pairs of a PDF name tree in document order.
fn collect_name_tree<'a>(
    doc: &'a Document,
    node: &'a Dictionary,
    depth: usize,
    entries: &mut Vec<(String, &'a Object)>,
) {
    if depth > MAX_NAME_TREE_DEPTH {
        return;
    }
    if let Some(names) = resolve_array(doc, node.get(b"Names").ok()) {
        for pair in names.chunks_exact(2) {
            if let Ok(key) = pair[0].as_str() {
                entries.push((pdf_text(key), &pair[1]));
            }
        }
    }
    if let Some(kids) = resolve_array(doc, node.get(b"Kids").ok()) {
        for kid in kids {
            if let Some(kid) = resolve_dict(doc, Some(kid)) {
                collect_name_tree(doc, kid, depth + 1, entries);
            }
        }
    }
}

fn read_filespec(doc: &Document, key: &str, filespec: &Object) -> EmbeddedFile {
    let filespec = resolve_dict(doc, Some(filespec));
    let name = filespec
        .and_then(|spec| {
            [b"UF".as_slice(), b"F".as_slice()]
                .iter()
                .find_map(|field| spec.get(field).ok()?.as_str().ok())
        })
        .map(pdf_text)
        .unwrap_or_else(|| key.to_string());

    let content = filespec
        .and_then(|spec| resolve_dict(doc, spec.get(b"EF").ok()))
        .and_then(|ef| ef.get(b"F").or_else(|_| ef.get(b"UF")).ok())
        .and_then(|stream| doc.dereference(stream).ok())
        .and_then(|(_, stream)| stream.as_stream().ok())
        .and_then(|stream| {
            if stream.dict.has(b"Filter") {
                stream
                    .decompressed_content_with_limit(MAX_EMBEDDED_FILE_SIZE)
                    .ok()
            } else {
                Some(stream.content.clone())
            }
        });

    EmbeddedFile { name, content }
}

fn resolve_dict<'a>(doc: &'a Document, object: Option<&'a Object>) -> Option<&'a Dictionary> {
    doc.dereference(object?).ok()?.1.as_dict().ok()
}

fn resolve_array<'a>(doc: &'a Document, object: Option<&'a Object>) -> Option<&'a Vec<Object>> {
    doc.dereference(object?).ok()?.1.as_array().ok()
}

/// Decodes a PDF text string (UTF-16BE with byte order mark, or
/// PDFDocEncoding, which matches Latin-1 for all printable ASCII characters).
fn pdf_text(bytes: &[u8]) -> String {
    match bytes {
        [0xFE, 0xFF, rest @ ..] => {
            let units: Vec<u16> = rest
                .chunks_exact(2)
                .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
                .collect();
            String::from_utf16_lossy(&units)
        }
        _ => bytes.iter().map(|&byte| char::from(byte)).collect(),
    }
}
