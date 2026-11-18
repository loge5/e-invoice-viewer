//! Opening a file: deciding what can be shown (spec 0007).

use thiserror::Error;

use crate::fields::{FieldModel, MapError, map_invoice};
use crate::pdf::{ExtractError, extract_invoice_xml};

/// PDF readers accept the header anywhere in the first 1024 bytes.
const PDF_HEADER_WINDOW: usize = 1024;

/// What an opened file contains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Document {
    /// A PDF; `invoice` is `None` if it embeds no electronic invoice.
    Pdf { invoice: Option<Invoice> },
    /// A plain XML invoice without PDF.
    Xml { invoice: Invoice },
}

/// An electronic invoice: its original XML and the field model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invoice {
    pub xml: Vec<u8>,
    pub fields: FieldModel,
}

/// Why a file cannot be shown as an invoice.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OpenError {
    #[error(transparent)]
    Pdf(ExtractError),
    #[error(transparent)]
    Xml(#[from] MapError),
}

/// Opens the bytes of a PDF or XML file.
pub fn open(bytes: &[u8]) -> Result<Document, OpenError> {
    if !is_pdf(bytes) {
        return Ok(Document::Xml {
            invoice: invoice(bytes.to_vec())?,
        });
    }
    match extract_invoice_xml(bytes) {
        Ok(embedded) => Ok(Document::Pdf {
            invoice: Some(invoice(embedded.xml)?),
        }),
        Err(ExtractError::NoEmbeddedInvoice { .. }) => Ok(Document::Pdf { invoice: None }),
        Err(error) => Err(OpenError::Pdf(error)),
    }
}

fn invoice(xml: Vec<u8>) -> Result<Invoice, OpenError> {
    let fields = map_invoice(&xml)?;
    Ok(Invoice { xml, fields })
}

pub(crate) fn is_pdf(bytes: &[u8]) -> bool {
    bytes[..bytes.len().min(PDF_HEADER_WINDOW)]
        .windows(5)
        .any(|window| window == b"%PDF-")
}
