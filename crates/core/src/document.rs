//! Opening a file: deciding what can be shown (spec 0007).

use thiserror::Error;

use crate::attachments::Attachment;
use crate::fields::{FieldModel, MapError, read_invoice};
use crate::i18n::Language;
use crate::pdf::{ExtractError, read_embedded};

/// PDF readers accept the header anywhere in the first 1024 bytes.
const PDF_HEADER_WINDOW: usize = 1024;

/// What an opened file contains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Document {
    /// A PDF; `invoice` is `None` if it embeds no electronic invoice.
    /// `files` are the further embedded files.
    Pdf {
        invoice: Option<Invoice>,
        files: Vec<Attachment>,
    },
    /// A plain XML invoice without PDF.
    Xml { invoice: Invoice },
}

impl Document {
    /// All attachments (0013): the further files of the PDF, then the
    /// supporting documents of the invoice (BG-24), each in document order.
    pub fn attachments(&self) -> impl Iterator<Item = &Attachment> {
        let (files, invoice) = match self {
            Self::Pdf { invoice, files } => (files.as_slice(), invoice.as_ref()),
            Self::Xml { invoice } => (&[][..], Some(invoice)),
        };
        files
            .iter()
            .chain(invoice.into_iter().flat_map(|invoice| &invoice.attachments))
    }
}

/// An electronic invoice: its original XML, the field model and its
/// supporting documents (BG-24).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invoice {
    pub xml: Vec<u8>,
    pub fields: FieldModel,
    pub attachments: Vec<Attachment>,
}

/// Why a file cannot be shown as an invoice.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OpenError {
    #[error(transparent)]
    Pdf(ExtractError),
    #[error(transparent)]
    Xml(#[from] MapError),
}

/// Opens the bytes of a PDF or XML file; texts of the field model are in
/// `language`.
pub fn open(bytes: &[u8], language: Language) -> Result<Document, OpenError> {
    if !is_pdf(bytes) {
        return Ok(Document::Xml {
            invoice: invoice(bytes.to_vec(), language)?,
        });
    }
    let embedded = read_embedded(bytes).map_err(OpenError::Pdf)?;
    let invoice = match embedded.invoice {
        Ok(embedded) => Some(invoice(embedded.xml, language)?),
        Err(ExtractError::NoEmbeddedInvoice { .. }) => None,
        Err(error) => return Err(OpenError::Pdf(error)),
    };
    Ok(Document::Pdf {
        invoice,
        files: embedded
            .others
            .into_iter()
            .map(Attachment::from_pdf)
            .collect(),
    })
}

fn invoice(xml: Vec<u8>, language: Language) -> Result<Invoice, OpenError> {
    let (fields, attachments) = read_invoice(&xml, language)?;
    Ok(Invoice {
        xml,
        fields,
        attachments,
    })
}

fn is_pdf(bytes: &[u8]) -> bool {
    bytes[..bytes.len().min(PDF_HEADER_WINDOW)]
        .windows(5)
        .any(|window| window == b"%PDF-")
}
