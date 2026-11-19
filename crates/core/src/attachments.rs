//! Attachments of an invoice (spec 0013): files embedded in the PDF and
//! additional supporting documents of the invoice XML (BG-24).

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use roxmltree::Node;

pub use crate::pdf::ContentError;
use crate::pdf::{EmbeddedFile, MAX_EMBEDDED_FILE_SIZE};

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
    /// control or text-direction characters, and valid on Windows too
    /// (spec 0022); `Anhang-<position>` if nothing is left.
    pub fn proposed_file_name(&self, position: usize) -> String {
        let name = self.name.as_deref().unwrap_or_default();
        let last = name.rsplit(['/', '\\']).next().unwrap_or_default();
        let clean: String = last
            .chars()
            .filter(|c| is_visible(*c))
            .map(|c| if is_forbidden_on_windows(c) { '_' } else { c })
            .collect();
        // Windows drops trailing dots and spaces.
        let clean = clean.trim_end_matches(['.', ' ']);
        if clean.trim().is_empty() {
            fallback_name(position)
        } else if is_windows_device_name(clean) {
            format!("_{clean}")
        } else {
            clean.to_string()
        }
    }
}

/// Characters Windows does not allow in file names (besides the path
/// separators, which never reach here).
fn is_forbidden_on_windows(c: char) -> bool {
    matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*')
}

/// Whether `name` is a Windows device name (`CON`, `NUL`, `COM1`, …), also
/// with an extension; such files cannot be created on Windows.
fn is_windows_device_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or_default().trim_end();
    let upper = stem.to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| {
            upper.strip_prefix(prefix).is_some_and(|digit| {
                matches!(digit, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
        })
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

/// A supporting document (BG-24) from its binary object (BT-125), external
/// address (BT-124) and description; `None` if it has neither content nor
/// address.
pub(crate) fn supporting_document(
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

/// Decodes base64 (standard alphabet, padded; whitespace is ignored).
/// Content larger than `limit` bytes is not decoded.
fn decode_base64(text: &str, limit: usize) -> Result<Vec<u8>, ContentError> {
    let digits: Vec<u8> = text
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    // Checked before decoding, so that a huge text is never decoded.
    if digits.len() / 4 * 3 > limit + 2 {
        return Err(ContentError::TooLarge);
    }
    let content = STANDARD
        .decode(&digits)
        .map_err(|_| ContentError::Unreadable)?;
    if content.len() > limit {
        return Err(ContentError::TooLarge);
    }
    Ok(content)
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
