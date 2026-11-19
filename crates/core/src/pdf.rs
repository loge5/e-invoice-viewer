//! Reading ZUGFeRD / Factur-X PDFs (spec 0002).

use std::panic::{self, AssertUnwindSafe};

use lopdf::{Dictionary, Document, Object};
use thiserror::Error;

/// Names of the embedded invoice XML, in order of precedence (compared
/// case-insensitively).
const INVOICE_FILE_NAMES: [&str; 3] = ["factur-x.xml", "xrechnung.xml", "zugferd-invoice.xml"];

/// Upper bound for a decompressed embedded file or attachment, protecting
/// against decompression bombs.
pub const MAX_EMBEDDED_FILE_SIZE: usize = 64 * 1024 * 1024;

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
    read_embedded(pdf)?.invoice
}

/// The embedded files of a PDF: the invoice XML, and the others in document
/// order.
pub(crate) struct Embedded {
    pub invoice: Result<EmbeddedInvoice, ExtractError>,
    pub others: Vec<EmbeddedFile>,
}

/// Reads the embedded files of a PDF once; never panics. Fails only if the
/// PDF itself cannot be read.
pub(crate) fn read_embedded(pdf: &[u8]) -> Result<Embedded, ExtractError> {
    // The PDF parser must never take the application down, whatever the input.
    let mut files = panic::catch_unwind(AssertUnwindSafe(|| embedded_files(pdf)))
        .map_err(|_| ExtractError::NotReadablePdf)??;
    let position = INVOICE_FILE_NAMES.iter().find_map(|wanted| {
        files
            .iter()
            .position(|file| file.name.eq_ignore_ascii_case(wanted))
    });
    let Some(position) = position else {
        let embedded_files = files.iter().map(|file| file.name.clone()).collect();
        return Ok(Embedded {
            invoice: Err(ExtractError::NoEmbeddedInvoice { embedded_files }),
            others: files,
        });
    };
    let file = files.remove(position);
    let invoice = match file.content {
        Ok(xml) if !xml.is_empty() => Ok(EmbeddedInvoice {
            source_name: file.name,
            xml,
        }),
        Ok(_) => Err(ExtractError::EmptyInvoice {
            source_name: file.name,
        }),
        Err(_) => Err(ExtractError::NotReadablePdf),
    };
    Ok(Embedded {
        invoice,
        others: files,
    })
}

/// An embedded file of the PDF.
pub(crate) struct EmbeddedFile {
    pub name: String,
    /// File description (`/Desc`), if any.
    pub description: Option<String>,
    /// MIME type (`/Subtype` of the stream), if any.
    pub mime_type: Option<String>,
    pub content: Result<Vec<u8>, ContentError>,
}

/// Why the content of an attachment cannot be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentError {
    Unreadable,
    TooLarge,
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
        for [key, value] in names.as_chunks::<2>().0 {
            if let Ok(key) = key.as_str() {
                entries.push((pdf_text(key), value));
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

    let description = filespec
        .and_then(|spec| spec.get(b"Desc").ok()?.as_str().ok())
        .map(pdf_text)
        .filter(|text| !text.trim().is_empty());

    let stream = filespec
        .and_then(|spec| resolve_dict(doc, spec.get(b"EF").ok()))
        .and_then(|ef| ef.get(b"F").or_else(|_| ef.get(b"UF")).ok())
        .and_then(|stream| doc.dereference(stream).ok())
        .and_then(|(_, stream)| stream.as_stream().ok());
    let mime_type = stream
        .and_then(|stream| stream.dict.get(b"Subtype").ok()?.as_name().ok())
        .map(|name| String::from_utf8_lossy(name).into_owned())
        .filter(|name| !name.is_empty());
    let content = stream.map_or(Err(ContentError::Unreadable), |stream| {
        stream_content(doc, stream)
    });

    EmbeddedFile {
        name,
        description,
        mime_type,
        content,
    }
}

/// The decoded content of an embedded file stream, within the size limit.
///
/// lopdf decodes leniently: a damaged stream gives a truncated or empty
/// result instead of an error. So a result that differs from the declared
/// size (`/Params /Size`) is unreadable; without a declared size, an empty
/// result is accepted only if the raw stream is a complete zlib stream of
/// empty data.
fn stream_content(doc: &Document, stream: &lopdf::Stream) -> Result<Vec<u8>, ContentError> {
    let content = if stream.dict.has(b"Filter") {
        stream
            .decompressed_content_with_limit(MAX_EMBEDDED_FILE_SIZE)
            .map_err(|error| match error {
                lopdf::Error::Decompress(lopdf::DecompressError::MemoryLimitExceeded {
                    ..
                }) => ContentError::TooLarge,
                _ => ContentError::Unreadable,
            })?
    } else {
        stream.content.clone()
    };
    if content.len() > MAX_EMBEDDED_FILE_SIZE {
        return Err(ContentError::TooLarge);
    }
    let declared = stream
        .dict
        .get(b"Params")
        .ok()
        .and_then(|params| doc.dereference(params).ok())
        .and_then(|(_, params)| params.as_dict().ok())
        .and_then(|params| params.get(b"Size").ok()?.as_i64().ok());
    let intact = match declared {
        Some(size) => usize::try_from(size) == Ok(content.len()),
        None => !content.is_empty() || stream.content == content || is_empty_zlib(&stream.content),
    };
    if intact {
        Ok(content)
    } else {
        Err(ContentError::Unreadable)
    }
}

/// Whether `raw` is a complete zlib stream of empty data: it ends with the
/// Adler-32 checksum of no bytes (1), which a damaged stream almost never
/// does by chance.
fn is_empty_zlib(raw: &[u8]) -> bool {
    raw.len() >= 8 && raw.ends_with(&[0, 0, 0, 1])
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
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u16::from_be_bytes(*pair))
                .collect();
            String::from_utf16_lossy(&units)
        }
        _ => bytes.iter().map(|&byte| char::from(byte)).collect(),
    }
}
