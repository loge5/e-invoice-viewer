//! User-facing texts (German), kept free of GTK so they can be unit-tested.

use e_invoice_viewer_core::attachments::{Attachment, Content, ContentError, Source};
use e_invoice_viewer_core::attribution::Target;
use e_invoice_viewer_core::detect::DetectError;
use e_invoice_viewer_core::document::OpenError;
use e_invoice_viewer_core::fields::{Field, FieldModel, MapError};
use e_invoice_viewer_core::pdf::ExtractError;
use e_invoice_viewer_core::validate::{Finding, Severity, Validation};

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
pub const ZOOM_IN: &str = "Vergrößern (Strg++)";
pub const ZOOM_OUT: &str = "Verkleinern (Strg+-)";
pub const COPY_XML: &str = "XML kopieren";
pub const SAVE_XML: &str = "XML speichern unter…";
pub const SEARCH_PLACEHOLDER: &str = "Im XML suchen";
pub const VALIDATION_TITLE: &str = "Prüfergebnis";
pub const DOCUMENT_TARGET: &str = "Dokument";
pub const VALIDATION_FAILED_REASON: &str = "interner Fehler bei der Prüfung";
pub const RECENT_TITLE: &str = "Zuletzt geöffnet";
pub const ATTACHMENTS_TITLE: &str = "Anhänge";
pub const SHOW_IN_XML: &str = "Im XML zeigen";
pub const SHOW_IN_FIELDS: &str = "In Feldern zeigen";
pub const NO_FIELD: &str = "Kein Feld zugeordnet";
pub const NO_ATTACHMENTS: &str = "Keine Anhänge";
pub const SAVE_ATTACHMENT: &str = "Anhang speichern…";
pub const RECENT_EMPTY: &str = "Keine Einträge";
pub const DROP_ONLY_FIRST: &str = "Nur eine Datei pro Fenster – die erste wurde geöffnet";

/// State of the validation of the shown invoice (spec 0011).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationState {
    Running,
    Failed(String),
    Done(Validation),
}

/// German name of a severity, singular or plural.
pub fn severity_name(severity: Severity, count: usize) -> &'static str {
    match (severity, count == 1) {
        (Severity::Error, _) => "Fehler",
        (Severity::Warning, true) => "Warnung",
        (Severity::Warning, false) => "Warnungen",
        (Severity::Information, true) => "Hinweis",
        (Severity::Information, false) => "Hinweise",
    }
}

/// Number of findings per severity: errors, warnings, notes.
fn counts(severities: impl Iterator<Item = Severity>) -> [(Severity, usize); 3] {
    let mut counts = [
        (Severity::Error, 0),
        (Severity::Warning, 0),
        (Severity::Information, 0),
    ];
    for severity in severities {
        if let Some((_, count)) = counts.iter_mut().find(|(s, _)| *s == severity) {
            *count += 1;
        }
    }
    counts
}

/// `2 Fehler, 1 Warnung` for the severities that occur; empty if none.
fn count_parts(counts: &[(Severity, usize)]) -> Vec<String> {
    counts
        .iter()
        .filter(|(_, count)| *count > 0)
        .map(|(severity, count)| format!("{count} {}", severity_name(*severity, *count)))
        .collect()
}

/// Headline of a validation result, e.g. `1 Fehler, 1 Warnung, 1 Hinweis`,
/// `Keine Fehler, 1 Hinweis` or `Keine Befunde`.
pub fn headline(severities: impl Iterator<Item = Severity>) -> String {
    let counts = counts(severities);
    let parts = count_parts(&counts);
    let only_notes = counts[0].1 == 0 && counts[1].1 == 0;
    match (parts.is_empty(), only_notes) {
        (true, _) => "Keine Befunde".to_string(),
        (false, true) => format!("Keine Fehler, {}", parts.join(", ")),
        (false, false) => parts.join(", "),
    }
}

/// Number of findings in a section, e.g. `2 Fehler`; `None` without
/// findings.
pub fn section_count_text(severities: impl Iterator<Item = Severity>) -> Option<String> {
    let parts = count_parts(&counts(severities));
    (!parts.is_empty()).then(|| parts.join(", "))
}

/// Text of the status line: `Prüfe…`, the headline, the not-applicable
/// reason or `Prüfung fehlgeschlagen: <reason>`.
pub fn status_text(state: &ValidationState) -> String {
    match state {
        ValidationState::Running => "Prüfe…".to_string(),
        ValidationState::Failed(reason) => format!("Prüfung fehlgeschlagen: {reason}"),
        ValidationState::Done(validation) => match &validation.not_applicable_reason {
            Some(reason) => reason.clone(),
            None => headline(validation.findings.iter().map(|finding| finding.severity)),
        },
    }
}

/// `Geprüft nach EN 16931 CII 1.3.17 und XRechnung 3.0.2`; `None` if no
/// rules were applied.
pub fn rule_sets_text(validation: &Validation) -> Option<String> {
    (!validation.applied_rule_sets.is_empty()).then(|| {
        format!(
            "Geprüft nach {}",
            validation.applied_rule_sets.join(" und ")
        )
    })
}

/// Where a finding belongs: `BT-112 Rechnungsbetrag inkl. USt`,
/// `Position 2` or `Dokument`.
pub fn target_text(model: &FieldModel, target: &Target) -> String {
    match *target {
        Target::Field { section, field } => {
            let field = &model.sections[section].fields[field];
            format!("{} {}", field.id, field.label)
        }
        Target::Section(section) => model.sections[section].title.clone(),
        Target::Document => DOCUMENT_TARGET.to_string(),
    }
}

/// A finding with its severity as a word, e.g. `Fehler BR-CO-15: …`, for
/// screen readers and tooltips.
pub fn finding_text(finding: &Finding) -> String {
    format!(
        "{} {}: {}",
        severity_name(finding.severity, 1),
        finding.rule_id,
        finding.message
    )
}

/// A recent entry: the file name and its folder, with the home directory
/// abbreviated to `~`.
pub fn recent_entry_label(path: &std::path::Path, home: &std::path::Path) -> String {
    let name = path
        .file_name()
        .map_or_else(|| path.to_string_lossy(), |name| name.to_string_lossy());
    let folder = path.parent().unwrap_or(path);
    let folder = match folder.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => folder.display().to_string(),
    };
    format!("{name} — {folder}")
}

/// Message for an address that is not a local file.
pub fn not_local_message(uri: &str) -> String {
    format!("Die Adresse {uri} ist keine lokale Datei")
}

/// Message for a recent entry whose file no longer exists.
pub fn not_found_message(file_name: &str) -> String {
    format!("Die Datei {file_name} wurde nicht gefunden")
}

pub fn read_failed_message(file_name: &str, reason: &str) -> String {
    format!("Die Datei {file_name} kann nicht gelesen werden: {reason}")
}

/// A file size the way the GNOME file manager shows it: decimal units with a
/// German decimal comma, e.g. `24,6 kB`.
pub fn size_text(bytes: usize) -> String {
    const UNITS: [&str; 4] = ["kB", "MB", "GB", "TB"];
    match bytes {
        1 => "1 Byte".to_string(),
        0..1000 => format!("{bytes} Bytes"),
        _ => {
            #[allow(clippy::cast_precision_loss)] // one decimal is shown
            let mut value = bytes as f64 / 1000.0;
            let mut unit = 0;
            while value >= 999.95 && unit < UNITS.len() - 1 {
                value /= 1000.0;
                unit += 1;
            }
            format!("{value:.1} {}", UNITS[unit]).replace('.', ",")
        }
    }
}

/// The details line of an attachment: source, type, size or reason, and the
/// description, e.g. `PDF · application/pdf · 24,6 kB · Lieferschein 4711`.
pub fn attachment_details(attachment: &Attachment) -> String {
    let source = match attachment.source {
        Source::Pdf => "PDF",
        Source::Invoice => "Rechnung (BG-24)",
    };
    let mime_type = attachment.mime_type.as_deref().unwrap_or("unbekannt");
    let state = match &attachment.content {
        Content::Embedded(content) => size_text(content.len()),
        Content::External(address) => address.clone(),
        Content::Failed(ContentError::Unreadable) => "Inhalt nicht lesbar".to_string(),
        Content::Failed(ContentError::TooLarge) => "Zu groß (über 64 MB)".to_string(),
    };
    let mut parts = vec![source.to_string(), mime_type.to_string(), state];
    parts.extend(attachment.description.clone());
    parts.join(" · ")
}

/// Message when saving an attachment fails.
pub fn save_failed_message(name: &str, reason: &str) -> String {
    format!("Der Anhang {name} konnte nicht gespeichert werden: {reason}")
}

/// The assignment line of the XML view (0014): `BT-1 Rechnungsnummer`,
/// with the section title in repeated sections (`BT-131 Nettobetrag ·
/// Position 2`), a section title, or `Kein Feld zugeordnet`.
pub fn assignment_text(model: &FieldModel, target: &Target) -> String {
    match *target {
        Target::Field { section, field } => {
            let section = &model.sections[section];
            let field = &section.fields[field];
            if section.repeated {
                format!("{} {} · {}", field.id, field.label, section.title)
            } else {
                format!("{} {}", field.id, field.label)
            }
        }
        Target::Section(section) => model.sections[section].title.clone(),
        Target::Document => NO_FIELD.to_string(),
    }
}

/// The page indicator of the PDF pane: `Seite 2 von 3` (page is 0-based).
pub fn page_indicator_text(page: usize, count: usize) -> String {
    format!("Seite {} von {count}", page + 1)
}

/// The zoom level of the PDF pane, e.g. `150 %`.
pub fn zoom_text(percent: f64) -> String {
    format!("{percent:.0} %")
}

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

/// Reason shown when validation rejects the invoice it was given.
pub fn validation_error_reason(error: &DetectError) -> String {
    reason(&OpenError::Xml(MapError::Detect(error.clone())))
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
            "das Rechnungsformat wird nicht unterstützt".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use e_invoice_viewer_core::fields::Section;
    use std::path::Path;

    fn iban_field() -> Field {
        Field {
            id: "BT-84".into(),
            label: "IBAN".into(),
            raw: "DE02120300000000202051".into(),
            display: "DE02 1203 0000 0000 2020 51".into(),
            location: "/rsm:CrossIndustryInvoice".into(),
            attribute: None,
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
            attribute: None,
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

    fn finding(rule_id: &str, severity: Severity) -> Finding {
        Finding {
            rule_id: rule_id.into(),
            severity,
            message: format!("[{rule_id}]-Meldung"),
            location: "/rsm:CrossIndustryInvoice".into(),
            business_terms: Vec::new(),
        }
    }

    fn validation(severities: &[Severity]) -> Validation {
        Validation {
            applied_rule_sets: vec!["EN 16931 CII 1.3.17".into()],
            findings: severities
                .iter()
                .enumerate()
                .map(|(index, severity)| finding(&format!("BR-{index}"), *severity))
                .collect(),
            ..Validation::default()
        }
    }

    // Spec 0011, requirement: headlines
    #[test]
    fn headline_texts() {
        use Severity::{Error, Information, Warning};
        let cases: [(&[Severity], &str); 6] = [
            (
                &[Error, Warning, Information],
                "1 Fehler, 1 Warnung, 1 Hinweis",
            ),
            (&[Error, Error], "2 Fehler"),
            (
                &[Information, Warning, Information, Warning, Information],
                "2 Warnungen, 3 Hinweise",
            ),
            (&[Information], "Keine Fehler, 1 Hinweis"),
            (&[Information, Information], "Keine Fehler, 2 Hinweise"),
            (&[], "Keine Befunde"),
        ];
        for (severities, expected) in cases {
            assert_eq!(headline(severities.iter().copied()), expected);
        }
    }

    // Spec 0011, AC6-AC9, AC11: status line
    #[test]
    fn status_texts() {
        assert_eq!(status_text(&ValidationState::Running), "Prüfe…");
        assert_eq!(
            status_text(&ValidationState::Failed("interner Fehler".into())),
            "Prüfung fehlgeschlagen: interner Fehler"
        );
        assert_eq!(
            status_text(&ValidationState::Done(validation(&[Severity::Error]))),
            "1 Fehler"
        );
        let not_applicable = Validation {
            not_applicable_reason: Some(
                "Profil Minimum: keine vollständige Rechnung nach EN 16931".into(),
            ),
            ..Validation::default()
        };
        assert_eq!(
            status_text(&ValidationState::Done(not_applicable)),
            "Profil Minimum: keine vollständige Rechnung nach EN 16931"
        );
    }

    // Spec 0011, AC3 and AC6: rule sets and version note
    #[test]
    fn rule_set_texts() {
        let mut validation = validation(&[]);
        assert_eq!(
            rule_sets_text(&validation).as_deref(),
            Some("Geprüft nach EN 16931 CII 1.3.17")
        );
        validation.applied_rule_sets.push("XRechnung 3.0.2".into());
        assert_eq!(
            rule_sets_text(&validation).as_deref(),
            Some("Geprüft nach EN 16931 CII 1.3.17 und XRechnung 3.0.2")
        );
        assert_eq!(rule_sets_text(&Validation::default()), None);
    }

    // Spec 0011, AC1: section headers
    #[test]
    fn section_count_texts() {
        use Severity::{Error, Warning};
        assert_eq!(
            section_count_text([Error, Error].into_iter()).as_deref(),
            Some("2 Fehler")
        );
        assert_eq!(
            section_count_text([Warning, Error].into_iter()).as_deref(),
            Some("1 Fehler, 1 Warnung")
        );
        assert_eq!(section_count_text(std::iter::empty()), None);
    }

    // Spec 0011, AC3: where a finding belongs
    #[test]
    fn target_texts() {
        let model = FieldModel {
            sections: vec![
                Section {
                    title: "Summen (BG-22)".into(),
                    location: "/rsm:CrossIndustryInvoice/x".into(),
                    repeated: false,
                    fields: vec![Field {
                        id: "BT-112".into(),
                        label: "Rechnungsbetrag inkl. USt".into(),
                        raw: "1300.00".into(),
                        display: "1.300,00 EUR".into(),
                        location: "/rsm:CrossIndustryInvoice/x/y".into(),
                        attribute: None,
                    }],
                },
                Section {
                    title: "Position 2".into(),
                    location: "/rsm:CrossIndustryInvoice/z".into(),
                    repeated: true,
                    fields: Vec::new(),
                },
            ],
        };
        let field = Target::Field {
            section: 0,
            field: 0,
        };
        assert_eq!(
            target_text(&model, &field),
            "BT-112 Rechnungsbetrag inkl. USt"
        );
        assert_eq!(target_text(&model, &Target::Section(1)), "Position 2");
        assert_eq!(target_text(&model, &Target::Document), "Dokument");
    }

    // Spec 0014, AC4, AC5: the assignment line
    #[test]
    fn assignment_texts() {
        let field = |id: &str, label: &str| Field {
            id: id.into(),
            label: label.into(),
            raw: String::new(),
            display: String::new(),
            location: String::new(),
            attribute: None,
        };
        let model = FieldModel {
            sections: vec![
                Section {
                    title: "Rechnung".into(),
                    location: "/rsm:CrossIndustryInvoice".into(),
                    repeated: false,
                    fields: vec![field("BT-1", "Rechnungsnummer")],
                },
                Section {
                    title: "Position 2".into(),
                    location: "/rsm:CrossIndustryInvoice/x[2]".into(),
                    repeated: true,
                    fields: vec![field("BT-131", "Nettobetrag")],
                },
            ],
        };
        let at = |section, field| Target::Field { section, field };

        assert_eq!(assignment_text(&model, &at(0, 0)), "BT-1 Rechnungsnummer");
        assert_eq!(
            assignment_text(&model, &at(1, 0)),
            "BT-131 Nettobetrag · Position 2"
        );
        assert_eq!(assignment_text(&model, &Target::Section(1)), "Position 2");
        assert_eq!(
            assignment_text(&model, &Target::Document),
            "Kein Feld zugeordnet"
        );
    }

    // Spec 0011, requirement: severity is announced, not only coloured
    #[test]
    fn finding_accessible_text() {
        assert_eq!(
            finding_text(&finding("BR-CO-15", Severity::Error)),
            "Fehler BR-CO-15: [BR-CO-15]-Meldung"
        );
        assert_eq!(
            finding_text(&finding("BR-DE-21", Severity::Warning)),
            "Warnung BR-DE-21: [BR-DE-21]-Meldung"
        );
        assert_eq!(
            finding_text(&finding("BR-DE-TMP-32", Severity::Information)),
            "Hinweis BR-DE-TMP-32: [BR-DE-TMP-32]-Meldung"
        );
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

    // AC5
    #[test]
    fn recent_entry_label_shows_folder() {
        let home = Path::new("/home/anna");
        assert_eq!(
            recent_entry_label(Path::new("/home/anna/x/rechnung.pdf"), home),
            "rechnung.pdf — ~/x"
        );
        assert_eq!(
            recent_entry_label(Path::new("/home/anna/a.pdf"), home),
            "a.pdf — ~"
        );
        assert_eq!(
            recent_entry_label(Path::new("/tmp/y/rechnung.pdf"), home),
            "rechnung.pdf — /tmp/y"
        );
        assert_eq!(
            recent_entry_label(Path::new("/home/annabel/a.pdf"), home),
            "a.pdf — /home/annabel"
        );
    }

    // AC1, AC4, AC8
    #[test]
    fn message_texts() {
        assert_eq!(
            DROP_ONLY_FIRST,
            "Nur eine Datei pro Fenster – die erste wurde geöffnet"
        );
        assert_eq!(
            not_local_message("https://example.com/RE-2026-0042.pdf"),
            "Die Adresse https://example.com/RE-2026-0042.pdf ist keine lokale Datei"
        );
        assert_eq!(
            not_found_message("alt.pdf"),
            "Die Datei alt.pdf wurde nicht gefunden"
        );
        assert_eq!(RECENT_TITLE, "Zuletzt geöffnet");
    }

    // Spec 0013 AC1
    #[test]
    fn size_texts() {
        assert_eq!(size_text(1), "1 Byte");
        assert_eq!(size_text(999), "999 Bytes");
        assert_eq!(size_text(24_576), "24,6 kB");
        assert_eq!(size_text(1_300_000), "1,3 MB");
        assert_eq!(size_text(64 * 1024 * 1024), "67,1 MB");
    }

    // Spec 0013 AC1, AC6, AC7, AC9, AC10
    #[test]
    fn attachment_texts() {
        let attachment = |mime_type: Option<&str>, content| Attachment {
            name: Some("lieferschein.pdf".into()),
            source: Source::Pdf,
            mime_type: mime_type.map(str::to_string),
            description: Some("Lieferschein 4711".into()),
            content,
        };
        assert_eq!(
            attachment_details(&attachment(
                Some("application/pdf"),
                Content::Embedded(vec![0; 24_576])
            )),
            "PDF · application/pdf · 24,6 kB · Lieferschein 4711"
        );
        assert_eq!(
            attachment_details(&Attachment {
                source: Source::Invoice,
                description: None,
                ..attachment(
                    None,
                    Content::External("https://example.com/stunden.csv".into())
                )
            }),
            "Rechnung (BG-24) · unbekannt · https://example.com/stunden.csv"
        );
        assert!(
            attachment_details(&attachment(None, Content::Failed(ContentError::Unreadable)))
                .contains("Inhalt nicht lesbar")
        );
        assert!(
            attachment_details(&attachment(None, Content::Failed(ContentError::TooLarge)))
                .contains("Zu groß (über 64 MB)")
        );
        assert_eq!(
            save_failed_message("stunden.csv", "Keine Berechtigung"),
            "Der Anhang stunden.csv konnte nicht gespeichert werden: Keine Berechtigung"
        );
    }

    // Spec 0016, AC2, AC4
    #[test]
    fn pdf_navigation_texts() {
        assert_eq!(page_indicator_text(0, 3), "Seite 1 von 3");
        assert_eq!(page_indicator_text(1, 3), "Seite 2 von 3");
        assert_eq!(zoom_text(150.0), "150 %");
        assert_eq!(zoom_text(87.4), "87 %");
    }
}
