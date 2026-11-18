//! Attachments of an invoice (spec 0013): files embedded in the PDF and
//! additional supporting documents of the invoice XML (BG-24).

use roxmltree::{Document, Node};

pub use crate::pdf::ContentError;
use crate::pdf::{EmbeddedFile, MAX_EMBEDDED_FILE_SIZE};
use crate::xml_view::decode;

const RSM: &str = "urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100";
const RAM: &str =
    "urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100";
const CAC: &str = "urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2";
const CBC: &str = "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2";

/// Document type code of an additional supporting document in CII (BG-24).
const SUPPORTING_DOCUMENT: &str = "916";
/// Document type code of the invoiced object identifier (BT-18), which UBL
/// stores in the same element as BG-24.
const INVOICED_OBJECT: &str = "130";

/// Where an attachment comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A further file embedded in the PDF.
    Pdf,
    /// An additional supporting document of the invoice XML (BG-24).
    Invoice,
}

/// The content of an attachment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    /// The decoded content; it can be saved.
    Embedded(Vec<u8>),
    /// Only an external address (BT-124); it is never opened or fetched.
    External(String),
    /// The content cannot be read.
    Failed(ContentError),
}

/// One attachment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    /// The file name as stored, if any.
    pub name: Option<String>,
    pub source: Source,
    pub mime_type: Option<String>,
    pub description: Option<String>,
    pub content: Content,
}

impl Attachment {
    pub(crate) fn from_pdf(file: EmbeddedFile) -> Self {
        Self {
            name: Some(file.name),
            source: Source::Pdf,
            mime_type: file.mime_type,
            description: file.description,
            content: file.content.map_or_else(Content::Failed, Content::Embedded),
        }
    }

    /// The name shown in the list, without control and text-direction
    /// characters; `Anhang-<position>` if there is none. `position` is
    /// 1-based.
    pub fn display_name(&self, position: usize) -> String {
        let name: String = self
            .name
            .as_deref()
            .unwrap_or_default()
            .chars()
            .filter(|c| is_visible(*c))
            .collect();
        if name.trim().is_empty() {
            fallback_name(position)
        } else {
            name
        }
    }

    /// The file name proposed when saving: without any directory part,
    /// control or text-direction characters; `Anhang-<position>` if nothing
    /// is left.
    pub fn proposed_file_name(&self, position: usize) -> String {
        let name = self.name.as_deref().unwrap_or_default();
        let last = name.rsplit(['/', '\\']).next().unwrap_or_default();
        let clean: String = last.chars().filter(|c| is_visible(*c)).collect();
        match clean.trim() {
            "" | "." | ".." => fallback_name(position),
            _ => clean,
        }
    }
}

fn fallback_name(position: usize) -> String {
    format!("Anhang-{position}")
}

/// Whether a character may appear in a shown or proposed name: no control
/// characters and no characters that change the text direction (which could
/// disguise `…exe` as `…pdf`).
fn is_visible(c: char) -> bool {
    !c.is_control()
        && !matches!(
            c,
            '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
        )
}

/// The BG-24 entries of an invoice XML (CII or UBL) with embedded content or
/// an external address.
pub(crate) fn of_invoice(xml: &[u8]) -> Vec<Attachment> {
    let Ok(text) = decode(xml) else {
        return Vec::new();
    };
    let Ok(doc) = Document::parse(&text) else {
        return Vec::new();
    };
    let root = doc.root_element();
    if root.tag_name().namespace() == Some(RSM) {
        children(root, RSM, "SupplyChainTradeTransaction")
            .flat_map(|node| children(node, RAM, "ApplicableHeaderTradeAgreement"))
            .flat_map(|node| children(node, RAM, "AdditionalReferencedDocument"))
            .filter(|node| text_of(*node, RAM, "TypeCode").as_deref() == Some(SUPPORTING_DOCUMENT))
            .filter_map(|node| {
                entry(
                    child(node, RAM, "AttachmentBinaryObject"),
                    text_of(node, RAM, "URIID"),
                    text_of(node, RAM, "Name").or_else(|| text_of(node, RAM, "IssuerAssignedID")),
                )
            })
            .collect()
    } else {
        children(root, CAC, "AdditionalDocumentReference")
            .filter(|node| {
                text_of(*node, CBC, "DocumentTypeCode").as_deref() != Some(INVOICED_OBJECT)
            })
            .filter_map(|node| {
                let attachment = child(node, CAC, "Attachment");
                entry(
                    attachment.and_then(|a| child(a, CBC, "EmbeddedDocumentBinaryObject")),
                    attachment
                        .and_then(|a| child(a, CAC, "ExternalReference"))
                        .and_then(|reference| text_of(reference, CBC, "URI")),
                    text_of(node, CBC, "DocumentDescription").or_else(|| text_of(node, CBC, "ID")),
                )
            })
            .collect()
    }
}

/// An attachment from its binary object (BT-125), external address (BT-124)
/// and description; `None` if it has neither content nor address.
fn entry(
    binary: Option<Node>,
    address: Option<String>,
    description: Option<String>,
) -> Option<Attachment> {
    let attribute = |name| {
        binary
            .and_then(|node| node.attribute(name))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    };
    let content = match (binary, address) {
        (Some(node), _) => {
            match decode_base64(node.text().unwrap_or_default(), MAX_EMBEDDED_FILE_SIZE) {
                Ok(content) => Content::Embedded(content),
                Err(error) => Content::Failed(error),
            }
        }
        (None, Some(address)) => Content::External(address),
        (None, None) => return None,
    };
    Some(Attachment {
        name: attribute("filename"),
        source: Source::Invoice,
        mime_type: attribute("mimeCode"),
        description,
        content,
    })
}

fn children<'a, 'input>(
    node: Node<'a, 'input>,
    namespace: &'static str,
    name: &'static str,
) -> impl Iterator<Item = Node<'a, 'input>> {
    node.children().filter(move |child| {
        child.is_element()
            && child.tag_name().namespace() == Some(namespace)
            && child.tag_name().name() == name
    })
}

fn child<'a, 'input>(
    node: Node<'a, 'input>,
    namespace: &'static str,
    name: &'static str,
) -> Option<Node<'a, 'input>> {
    children(node, namespace, name).next()
}

fn text_of(node: Node, namespace: &'static str, name: &'static str) -> Option<String> {
    child(node, namespace, name)
        .and_then(|child| child.text())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

/// Decodes base64 (standard alphabet, padded; whitespace is ignored).
/// Content larger than `limit` bytes is not decoded.
fn decode_base64(text: &str, limit: usize) -> Result<Vec<u8>, ContentError> {
    let digits: Vec<u8> = text
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    if !digits.len().is_multiple_of(4) {
        return Err(ContentError::Unreadable);
    }
    if digits.len() / 4 * 3 > limit + 2 {
        return Err(ContentError::TooLarge);
    }
    let value = |byte: u8| match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    };
    let mut out = Vec::with_capacity(digits.len() / 4 * 3);
    let chunks = digits.chunks_exact(4);
    let last = chunks.len().saturating_sub(1);
    for (index, chunk) in chunks.enumerate() {
        let padding = chunk.iter().rev().take_while(|byte| **byte == b'=').count();
        if padding > 2 || (padding > 0 && index != last) {
            return Err(ContentError::Unreadable);
        }
        let mut bits = 0_u32;
        for &byte in &chunk[..4 - padding] {
            bits = (bits << 6) | u32::from(value(byte).ok_or(ContentError::Unreadable)?);
        }
        bits <<= 6 * padding;
        let bytes = bits.to_be_bytes();
        out.extend_from_slice(&bytes[1..4 - padding]);
    }
    if out.len() > limit {
        return Err(ContentError::TooLarge);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_decoding() {
        assert_eq!(decode_base64("SGFsbG8=", 100), Ok(b"Hallo".to_vec()));
        assert_eq!(decode_base64("SGFs\n bG8h", 100), Ok(b"Hallo!".to_vec()));
        assert_eq!(decode_base64("SA==", 100), Ok(b"H".to_vec()));
        assert_eq!(decode_base64("", 100), Ok(Vec::new()));
        assert_eq!(decode_base64("SGFsbG8", 100), Err(ContentError::Unreadable));
        assert_eq!(
            decode_base64("SA==SA==", 100),
            Err(ContentError::Unreadable)
        );
        assert_eq!(decode_base64("S*==", 100), Err(ContentError::Unreadable));
        // The limit of 0002, scaled down: 6 decoded bytes, limit 5.
        assert_eq!(decode_base64("SGFsbG8h", 5), Err(ContentError::TooLarge));
        assert_eq!(decode_base64("SGFsbG8=", 5), Ok(b"Hallo".to_vec()));
    }
}
