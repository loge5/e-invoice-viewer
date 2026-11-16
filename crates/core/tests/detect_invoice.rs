//! Spec 0003 — Detect the syntax and profile of an invoice XML.

use std::time::{Duration, Instant};

use e_invoice_viewer_core::detect::{DetectError, DocumentKind, Profile, Syntax, detect};

const CII_NS: &str = "urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100";
const RAM_NS: &str =
    "urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100";
const UBL_INVOICE_NS: &str = "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2";
const UBL_CREDIT_NOTE_NS: &str = "urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2";
const CBC_NS: &str = "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2";

const EN16931: &str = "urn:cen.eu:en16931:2017";
const XRECHNUNG_3_0: &str = "urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0";

/// Minimal CII invoice with optional BT-24 and BT-3.
fn cii(bt24: Option<&str>, type_code: &str) -> String {
    let context = match bt24 {
        Some(id) => format!(
            "<rsm:ExchangedDocumentContext><ram:GuidelineSpecifiedDocumentContextParameter><ram:ID>{id}</ram:ID></ram:GuidelineSpecifiedDocumentContextParameter></rsm:ExchangedDocumentContext>"
        ),
        None => "<rsm:ExchangedDocumentContext/>".to_string(),
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rsm:CrossIndustryInvoice xmlns:rsm=\"{CII_NS}\" xmlns:ram=\"{RAM_NS}\">{context}<rsm:ExchangedDocument><ram:ID>RE-2026-0042</ram:ID><ram:TypeCode>{type_code}</ram:TypeCode></rsm:ExchangedDocument></rsm:CrossIndustryInvoice>"
    )
}

/// Minimal UBL invoice.
fn ubl_invoice(bt24: &str, type_code: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Invoice xmlns=\"{UBL_INVOICE_NS}\" xmlns:cbc=\"{CBC_NS}\"><cbc:CustomizationID>{bt24}</cbc:CustomizationID><cbc:ID>RE-2026-0042</cbc:ID><cbc:InvoiceTypeCode>{type_code}</cbc:InvoiceTypeCode></Invoice>"
    )
}

/// Minimal UBL credit note, optionally without BT-3.
fn ubl_credit_note(bt24: &str, type_code: Option<&str>) -> String {
    let type_code = type_code
        .map(|code| format!("<cbc:CreditNoteTypeCode>{code}</cbc:CreditNoteTypeCode>"))
        .unwrap_or_default();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<CreditNote xmlns=\"{UBL_CREDIT_NOTE_NS}\" xmlns:cbc=\"{CBC_NS}\"><cbc:CustomizationID>{bt24}</cbc:CustomizationID><cbc:ID>GS-2026-0001</cbc:ID>{type_code}</CreditNote>"
    )
}

// AC1
#[test]
fn detects_cii_syntax() {
    let detection = detect(cii(Some(EN16931), "380").as_bytes()).unwrap();
    assert_eq!(detection.syntax, Syntax::Cii);
}

// AC1
#[test]
fn detects_ubl_invoice_syntax() {
    let detection = detect(ubl_invoice(EN16931, "380").as_bytes()).unwrap();
    assert_eq!(detection.syntax, Syntax::Ubl);
}

// AC1
#[test]
fn detects_ubl_credit_note_syntax() {
    let detection = detect(ubl_credit_note(EN16931, Some("381")).as_bytes()).unwrap();
    assert_eq!(detection.syntax, Syntax::Ubl);
}

// AC2
#[test]
fn cii_type_code_380_is_invoice() {
    let detection = detect(cii(Some(EN16931), "380").as_bytes()).unwrap();
    assert_eq!(detection.kind, DocumentKind::Invoice);
}

// AC2
#[test]
fn cii_type_code_381_is_credit_note() {
    let detection = detect(cii(Some(EN16931), "381").as_bytes()).unwrap();
    assert_eq!(detection.kind, DocumentKind::CreditNote);
}

// AC2
#[test]
fn ubl_invoice_380_is_invoice_and_credit_note_381_is_credit_note() {
    let invoice = detect(ubl_invoice(EN16931, "380").as_bytes()).unwrap();
    let credit_note = detect(ubl_credit_note(EN16931, Some("381")).as_bytes()).unwrap();

    assert_eq!(invoice.kind, DocumentKind::Invoice);
    assert_eq!(credit_note.kind, DocumentKind::CreditNote);
}

// Requirement "Document kind": 261 and a UBL CreditNote root mean credit note
#[test]
fn type_code_261_and_ubl_credit_note_root_mean_credit_note() {
    let cii_261 = detect(cii(Some(EN16931), "261").as_bytes()).unwrap();
    let ubl_without_type_code = detect(ubl_credit_note(EN16931, None).as_bytes()).unwrap();

    assert_eq!(cii_261.kind, DocumentKind::CreditNote);
    assert_eq!(ubl_without_type_code.kind, DocumentKind::CreditNote);
}

// AC3
#[test]
fn detects_profile_from_specification_identifier() {
    let cases: [(&str, Profile, Option<&str>, bool); 10] = [
        ("urn:factur-x.eu:1p0:minimum", Profile::Minimum, None, false),
        ("urn:factur-x.eu:1p0:basicwl", Profile::BasicWl, None, false),
        (
            "urn:cen.eu:en16931:2017#compliant#urn:factur-x.eu:1p0:basic",
            Profile::Basic,
            None,
            false,
        ),
        ("urn:cen.eu:en16931:2017", Profile::En16931, None, false),
        (" urn:cen.eu:en16931:2017 ", Profile::En16931, None, false),
        (
            "urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended",
            Profile::Extended,
            None,
            false,
        ),
        (XRECHNUNG_3_0, Profile::XRechnung, Some("3.0"), false),
        (
            "urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_2.3",
            Profile::XRechnung,
            Some("2.3"),
            false,
        ),
        (
            "urn:cen.eu:en16931:2017#conformant#urn:xeinkauf.de:kosit:extension:xrechnung_3.0",
            Profile::XRechnung,
            Some("3.0"),
            true,
        ),
        (
            "urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0",
            Profile::Unknown,
            None,
            false,
        ),
    ];

    for (bt24, profile, version, extension) in cases {
        let detection = detect(cii(Some(bt24), "380").as_bytes()).unwrap();
        assert_eq!(detection.profile, profile, "profile for {bt24:?}");
        assert_eq!(
            detection.profile_version.as_deref(),
            version,
            "version for {bt24:?}"
        );
        assert_eq!(detection.extension, extension, "extension for {bt24:?}");
    }
}

// AC3 / contract: the raw identifier is kept untrimmed
#[test]
fn keeps_specification_identifier_untrimmed() {
    let detection = detect(cii(Some(" urn:cen.eu:en16931:2017 "), "380").as_bytes()).unwrap();

    assert_eq!(detection.profile, Profile::En16931);
    assert_eq!(
        detection.specification_identifier,
        " urn:cen.eu:en16931:2017 "
    );
}

// AC4
#[test]
fn detects_xrechnung_profile_in_ubl() {
    let detection = detect(ubl_invoice(XRECHNUNG_3_0, "380").as_bytes()).unwrap();

    assert_eq!(detection.profile, Profile::XRechnung);
    assert_eq!(detection.profile_version.as_deref(), Some("3.0"));
}

// AC5
#[test]
fn unknown_identifier_is_kept_raw() {
    let detection = detect(cii(Some("urn:example:custom-profile"), "380").as_bytes()).unwrap();

    assert_eq!(detection.profile, Profile::Unknown);
    assert_eq!(
        detection.specification_identifier,
        "urn:example:custom-profile"
    );
}

// AC5
#[test]
fn missing_identifier_is_unknown_and_empty_without_error() {
    let detection = detect(cii(None, "380").as_bytes()).unwrap();

    assert_eq!(detection.profile, Profile::Unknown);
    assert_eq!(detection.specification_identifier, "");
}

// AC6
#[test]
fn non_invoice_root_is_rejected() {
    let error = detect(b"<?xml version=\"1.0\"?><order><id>1</id></order>").unwrap_err();
    assert_eq!(error, DetectError::NotAnInvoice);
}

// AC6
#[test]
fn malformed_xml_reports_line_1() {
    let error = detect(b"<a><b></a>").unwrap_err();
    assert_eq!(error, DetectError::NotWellFormed { line: 1 });
}

// AC6
#[test]
fn zugferd_1_0_is_unsupported() {
    let xml = b"<?xml version=\"1.0\"?><rsm:CrossIndustryDocument xmlns:rsm=\"urn:ferd:CrossIndustryDocument:invoice:1p0\"/>";
    assert_eq!(detect(xml).unwrap_err(), DetectError::UnsupportedZugferd1);
}

// AC7
#[test]
fn doctype_with_external_entity_is_rejected() {
    let xml = format!(
        "<?xml version=\"1.0\"?><!DOCTYPE x [<!ENTITY e SYSTEM \"file:///etc/passwd\">]><rsm:CrossIndustryInvoice xmlns:rsm=\"{CII_NS}\">&e;</rsm:CrossIndustryInvoice>"
    );

    let error = detect(xml.as_bytes()).unwrap_err();

    assert_eq!(error, DetectError::DoctypeNotSupported);
    assert!(!format!("{error:?} {error}").contains("root:"));
}

// AC7
#[test]
fn billion_laughs_is_rejected_within_one_second() {
    let mut xml = String::from("<?xml version=\"1.0\"?><!DOCTYPE lolz [<!ENTITY lol0 \"lol\">");
    for level in 1..=9 {
        let previous = format!("&lol{};", level - 1).repeat(10);
        xml.push_str(&format!("<!ENTITY lol{level} \"{previous}\">"));
    }
    xml.push_str("]><lolz>&lol9;</lolz>");

    let start = Instant::now();
    let error = detect(xml.as_bytes()).unwrap_err();
    let elapsed = start.elapsed();

    assert_eq!(error, DetectError::DoctypeNotSupported);
    assert!(elapsed < Duration::from_secs(1), "took {elapsed:?}");
}
