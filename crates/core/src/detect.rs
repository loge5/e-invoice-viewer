//! Detecting syntax, document kind and profile of an invoice XML (spec 0003).

use roxmltree::{Document, Node};
use thiserror::Error;

use crate::xml_view::decode;

const CII_NS: &str = "urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100";
const RAM_NS: &str =
    "urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100";
const UBL_INVOICE_NS: &str = "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2";
const UBL_CREDIT_NOTE_NS: &str = "urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2";
const CBC_NS: &str = "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2";
const ZUGFERD_1_NS: &str = "urn:ferd:CrossIndustryDocument:invoice:1p0";

/// Invoice type codes (BT-3) that denote a credit note.
const CREDIT_NOTE_TYPE_CODES: [&str; 2] = ["381", "261"];

/// Fixed specification identifiers (BT-24) and their profiles.
const FIXED_PROFILES: [(&str, Profile); 5] = [
    ("urn:factur-x.eu:1p0:minimum", Profile::Minimum),
    ("urn:factur-x.eu:1p0:basicwl", Profile::BasicWl),
    (
        "urn:cen.eu:en16931:2017#compliant#urn:factur-x.eu:1p0:basic",
        Profile::Basic,
    ),
    ("urn:cen.eu:en16931:2017", Profile::En16931),
    (
        "urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended",
        Profile::Extended,
    ),
];

/// Prefixes of XRechnung identifiers, followed by the version; the flag marks
/// the XRechnung Extension.
const XRECHNUNG_PREFIXES: [(&str, bool); 3] = [
    (
        "urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_",
        false,
    ),
    (
        "urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_",
        false,
    ),
    (
        "urn:cen.eu:en16931:2017#conformant#urn:xeinkauf.de:kosit:extension:xrechnung_",
        true,
    ),
];

/// XML syntax of the invoice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syntax {
    Cii,
    Ubl,
}

/// Whether the document is an invoice or a credit note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    Invoice,
    CreditNote,
}

/// Profile declared in the specification identifier (BT-24).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Minimum,
    BasicWl,
    Basic,
    En16931,
    Extended,
    XRechnung,
    Unknown,
}

/// Result of a successful detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detection {
    pub syntax: Syntax,
    pub kind: DocumentKind,
    pub profile: Profile,
    /// XRechnung version from the identifier (e.g. `3.0`).
    pub profile_version: Option<String>,
    /// `true` for the XRechnung Extension.
    pub extension: bool,
    /// Raw value of BT-24 (untrimmed), empty if missing.
    pub specification_identifier: String,
}

/// Why the document is not a supported invoice.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DetectError {
    #[error("XML is not well-formed (line {line})")]
    NotWellFormed { line: u32 },
    #[error("XML contains a DOCTYPE declaration – not supported")]
    DoctypeNotSupported,
    #[error("not an electronic invoice")]
    NotAnInvoice,
    #[error("unsupported format: ZUGFeRD 1.0")]
    UnsupportedZugferd1,
    #[error("unsupported character encoding: {encoding}")]
    UnsupportedEncoding { encoding: String },
}

/// Detects syntax, document kind and profile of an invoice XML.
pub fn detect(xml: &[u8]) -> Result<Detection, DetectError> {
    let text = decode(xml)?;
    let text = text.as_ref();

    // Default options reject any DOCTYPE, so no entity is ever resolved.
    let doc = Document::parse(text).map_err(|error| match error {
        roxmltree::Error::DtdDetected => DetectError::DoctypeNotSupported,
        other => DetectError::NotWellFormed {
            line: other.pos().row,
        },
    })?;
    let root = doc.root_element();
    let tag = root.tag_name();

    let (syntax, root_kind, specification_identifier, type_code) =
        match (tag.namespace(), tag.name()) {
            (Some(CII_NS), "CrossIndustryInvoice") => (
                Syntax::Cii,
                DocumentKind::Invoice,
                text_at(
                    root,
                    &[
                        (CII_NS, "ExchangedDocumentContext"),
                        (RAM_NS, "GuidelineSpecifiedDocumentContextParameter"),
                        (RAM_NS, "ID"),
                    ],
                ),
                text_at(root, &[(CII_NS, "ExchangedDocument"), (RAM_NS, "TypeCode")]),
            ),
            (Some(UBL_INVOICE_NS), "Invoice") => (
                Syntax::Ubl,
                DocumentKind::Invoice,
                text_at(root, &[(CBC_NS, "CustomizationID")]),
                text_at(root, &[(CBC_NS, "InvoiceTypeCode")]),
            ),
            (Some(UBL_CREDIT_NOTE_NS), "CreditNote") => (
                Syntax::Ubl,
                DocumentKind::CreditNote,
                text_at(root, &[(CBC_NS, "CustomizationID")]),
                text_at(root, &[(CBC_NS, "CreditNoteTypeCode")]),
            ),
            (Some(ZUGFERD_1_NS), "CrossIndustryDocument") => {
                return Err(DetectError::UnsupportedZugferd1);
            }
            _ => return Err(DetectError::NotAnInvoice),
        };

    let kind = match type_code.as_deref().map(str::trim) {
        Some(code) if CREDIT_NOTE_TYPE_CODES.contains(&code) => DocumentKind::CreditNote,
        _ => root_kind,
    };
    let specification_identifier = specification_identifier.unwrap_or_default();
    let (profile, profile_version, extension) = profile_of(specification_identifier.trim());

    Ok(Detection {
        syntax,
        kind,
        profile,
        profile_version,
        extension,
        specification_identifier,
    })
}

/// Maps a trimmed specification identifier to (profile, version, extension).
fn profile_of(identifier: &str) -> (Profile, Option<String>, bool) {
    if let Some((_, profile)) = FIXED_PROFILES.iter().find(|(id, _)| *id == identifier) {
        return (*profile, None, false);
    }
    for (prefix, extension) in XRECHNUNG_PREFIXES {
        if let Some(version) = identifier.strip_prefix(prefix)
            && is_version(version)
        {
            return (Profile::XRechnung, Some(version.to_string()), extension);
        }
    }
    (Profile::Unknown, None, false)
}

/// A version like `3.0` or `2.3.1`: digit groups separated by dots.
fn is_version(text: &str) -> bool {
    !text.is_empty()
        && text
            .split('.')
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

/// Text of the element reached by following `path` (namespace, local name)
/// from `node`, taking the first match at each step.
fn text_at(node: Node, path: &[(&str, &str)]) -> Option<String> {
    let mut current = node;
    for (namespace, name) in path {
        current = current.children().find(|child| {
            child.is_element()
                && child.tag_name().namespace() == Some(*namespace)
                && child.tag_name().name() == *name
        })?;
    }
    Some(current.text().unwrap_or_default().to_string())
}
