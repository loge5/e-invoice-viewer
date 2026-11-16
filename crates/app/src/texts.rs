//! User-facing texts (German), kept free of GTK so they can be unit-tested.

use e_invoice_viewer_core::detect::DetectError;
use e_invoice_viewer_core::document::OpenError;
use e_invoice_viewer_core::fields::{Field, MapError};
use e_invoice_viewer_core::pdf::ExtractError;

pub const APP_TITLE: &str = "E-Invoice Viewer";
pub const START_TITLE: &str = "Rechnung öffnen";
pub const OPEN_BUTTON: &str = "Öffnen…";
pub const CLOSE_BUTTON: &str = "Schließen";
pub const COPY_VALUE: &str = "Wert kopieren";
pub const NO_INVOICE_IN_PDF: &str = "Diese PDF enthält keine elektronische Rechnung";
pub const PDF_NOT_RENDERABLE: &str = "Die PDF-Seiten können nicht angezeigt werden";
pub const XML_ONLY_NOTE: &str = "Reine XML-Rechnung – kein PDF enthalten";
pub const VIEW_FIELDS: &str = "Felder";
pub const VIEW_XML: &str = "XML";
pub const COPY_XML: &str = "XML kopieren";
pub const SAVE_XML: &str = "XML speichern unter…";
pub const SEARCH_PLACEHOLDER: &str = "Im XML suchen";

/// Window title, with the file name while an invoice is shown.
pub fn window_title(file_name: Option<&str>) -> String {
    match file_name {
        Some(name) => format!("{name} – {APP_TITLE}"),
        None => APP_TITLE.to_string(),
    }
}

/// One field row: `BT-1 · Rechnungsnummer · RE-2026-0042`.
pub fn field_row_text(field: &Field) -> String {
    format!("{} · {} · {}", field.id, field.label, field.display)
}

/// The value "Wert kopieren" puts on the clipboard: the raw value.
pub fn copy_value(field: &Field) -> &str {
    &field.raw
}

/// File name proposed when saving the invoice XML of `source_file_name`.
pub fn suggested_xml_file_name(source_file_name: &str) -> String {
    let path = std::path::Path::new(source_file_name);
    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xml"))
    {
        return source_file_name.to_string();
    }
    let stem = path
        .file_stem()
        .map_or(source_file_name.into(), |stem| stem.to_string_lossy());
    format!("{stem}.xml")
}

/// Search status: `1 von 3`, or `Keine Treffer`. `current` is 1-based and
/// `None` while no match is selected.
pub fn search_status_text(current: Option<usize>, total: usize) -> String {
    match (current, total) {
        (_, 0) => "Keine Treffer".to_string(),
        (Some(current), total) => format!("{current} von {total}"),
        (None, total) => format!("{total} Treffer"),
    }
}

/// Message for a file that cannot be read as an invoice.
pub fn error_message(file_name: &str, error: &OpenError) -> String {
    format!(
        "Die Datei {file_name} kann nicht als Rechnung gelesen werden: {}",
        reason(error)
    )
}

fn reason(error: &OpenError) -> String {
    match error {
        OpenError::Pdf(ExtractError::NotReadablePdf) => "keine lesbare PDF-Datei".to_string(),
        OpenError::Pdf(ExtractError::NoEmbeddedInvoice { .. }) => NO_INVOICE_IN_PDF.to_string(),
        OpenError::Pdf(ExtractError::EmptyInvoice { source_name }) => {
            format!("die eingebettete Rechnung {source_name} ist leer")
        }
        OpenError::Xml(MapError::Detect(error)) => match error {
            DetectError::NotWellFormed { line } => {
                format!("XML ist nicht wohlgeformt (Zeile {line})")
            }
            DetectError::DoctypeNotSupported => {
                "XML enthält eine DOCTYPE-Deklaration – nicht unterstützt".to_string()
            }
            DetectError::NotAnInvoice => "keine elektronische Rechnung".to_string(),
            DetectError::UnsupportedZugferd1 => "ZUGFeRD 1.0 wird nicht unterstützt".to_string(),
            DetectError::UnsupportedEncoding { encoding } => {
                format!("die Zeichenkodierung {encoding} wird nicht unterstützt")
            }
        },
        OpenError::Xml(MapError::UnsupportedSyntax) => {
            "UBL-Rechnungen werden noch nicht unterstützt".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iban_field() -> Field {
        Field {
            id: "BT-84".into(),
            label: "IBAN".into(),
            raw: "DE02120300000000202051".into(),
            display: "DE02 1203 0000 0000 2020 51".into(),
            location: "/rsm:CrossIndustryInvoice".into(),
        }
    }

    // AC1, AC2, AC4
    #[test]
    fn window_title_with_and_without_file() {
        assert_eq!(window_title(None), "E-Invoice Viewer");
        assert_eq!(
            window_title(Some("RE-2026-0042.pdf")),
            "RE-2026-0042.pdf – E-Invoice Viewer"
        );
        assert!(!window_title(None).contains("RE-2026-0042"));
    }

    // AC2, AC5
    #[test]
    fn field_row_and_copy_value() {
        let invoice_number = Field {
            id: "BT-1".into(),
            label: "Rechnungsnummer".into(),
            raw: "RE-2026-0042".into(),
            display: "RE-2026-0042".into(),
            location: "/rsm:CrossIndustryInvoice/rsm:ExchangedDocument/ram:ID".into(),
        };

        assert_eq!(
            field_row_text(&invoice_number),
            "BT-1 · Rechnungsnummer · RE-2026-0042"
        );
        assert_eq!(copy_value(&iban_field()), "DE02120300000000202051");
    }

    // Spec 0008, AC6
    #[test]
    fn suggested_xml_file_name_replaces_extension() {
        assert_eq!(
            suggested_xml_file_name("RE-2026-0042.pdf"),
            "RE-2026-0042.xml"
        );
        assert_eq!(suggested_xml_file_name("rechnung.xml"), "rechnung.xml");
        assert_eq!(suggested_xml_file_name("ohne-endung"), "ohne-endung.xml");
    }

    // Spec 0008, AC4
    #[test]
    fn search_status_texts() {
        assert_eq!(search_status_text(Some(1), 1), "1 von 1");
        assert_eq!(search_status_text(Some(2), 5), "2 von 5");
        assert_eq!(search_status_text(None, 3), "3 Treffer");
        assert_eq!(search_status_text(None, 0), "Keine Treffer");
    }

    // AC6
    #[test]
    fn error_messages() {
        let cases = [
            (
                "kaputt.pdf",
                OpenError::Pdf(ExtractError::NotReadablePdf),
                "Die Datei kaputt.pdf kann nicht als Rechnung gelesen werden: keine lesbare PDF-Datei",
            ),
            (
                "rechnung.xml",
                OpenError::Xml(DetectError::NotWellFormed { line: 1 }.into()),
                "Die Datei rechnung.xml kann nicht als Rechnung gelesen werden: XML ist nicht wohlgeformt (Zeile 1)",
            ),
            (
                "bestellung.xml",
                OpenError::Xml(DetectError::NotAnInvoice.into()),
                "Die Datei bestellung.xml kann nicht als Rechnung gelesen werden: keine elektronische Rechnung",
            ),
        ];

        for (file_name, error, expected) in cases {
            assert_eq!(error_message(file_name, &error), expected);
        }
    }
}
