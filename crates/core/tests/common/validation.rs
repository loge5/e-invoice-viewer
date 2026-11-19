//! Shared helpers and sample variants of the validation specs (0009, 0010).

use e_invoice_viewer_core::i18n::Language;
use e_invoice_viewer_core::validate::{Finding, Severity, Validation, Validator};

use super::{fixture, replace_once};

/// The schema check comes first (spec 0023); `validate` reports in German.
pub const CII_SCHEMA: &str = "XML-Schema CII D16B";
pub const UBL_SCHEMA: &str = "XML-Schema UBL 2.1";
pub const CII_RULES: &str = "EN 16931 CII 1.3.17";
pub const UBL_RULES: &str = "EN 16931 UBL 1.3.17";
pub const XRECHNUNG_RULES: &str = "XRechnung 3.0.2";

const CEN_ID: &str = "urn:cen.eu:en16931:2017";
const XRECHNUNG_ID: &str = "urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0";

pub fn validate(xml: &str) -> Validation {
    Validator::new()
        .validate(xml.as_bytes(), Language::German)
        .expect("validation result")
}

pub fn findings_with<'a>(validation: &'a Validation, rule_id: &str) -> Vec<&'a Finding> {
    validation
        .findings
        .iter()
        .filter(|finding| finding.rule_id == rule_id)
        .collect()
}

pub fn errors(validation: &Validation) -> Vec<&Finding> {
    validation
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .collect()
}

/// Whether a rule ID belongs to the XRechnung rule set (spec 0010).
pub fn is_xrechnung_rule(rule_id: &str) -> bool {
    ["BR-DE-", "BR-DEX-", "BR-TMP-", "PEPPOL-EN16931-"]
        .iter()
        .any(|prefix| rule_id.starts_with(prefix))
}

/// Severity as written in the KoSIT reports.
pub fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Information => "information",
    }
}

fn with_specification_identifier(xml: &str, from: &str, to: &str) -> String {
    replace_once(
        xml,
        &format!("<ram:ID>{from}</ram:ID>"),
        &format!("<ram:ID>{to}</ram:ID>"),
    )
}

/// The variants of the samples the validation specs describe, by file name.
/// They are part of the KoSIT reference set (`write_variants`).
pub fn variants() -> Vec<(&'static str, String)> {
    let a = fixture("cii/sample-a.xml");
    let b = fixture("cii/sample-b.xml");
    let a_with = |id: &str| with_specification_identifier(&a, CEN_ID, id);
    let b_with = |id: &str| with_specification_identifier(&b, XRECHNUNG_ID, id);
    vec![
        // Spec 0009
        (
            "a-without-bt24.xml",
            replace_once(&a, "      <ram:ID>urn:cen.eu:en16931:2017</ram:ID>\n", ""),
        ),
        (
            "a-grand-total-1300.xml",
            replace_once(
                &a,
                "<ram:GrandTotalAmount>1249.50</ram:GrandTotalAmount>",
                "<ram:GrandTotalAmount>1300.00</ram:GrandTotalAmount>",
            ),
        ),
        (
            "a-ubl-without-bt1.xml",
            replace_once(
                &fixture("ubl/sample-a.xml"),
                "  <cbc:ID>RE-2026-0042</cbc:ID>\n",
                "",
            ),
        ),
        ("a-minimum.xml", a_with("urn:factur-x.eu:1p0:minimum")),
        ("a-basicwl.xml", a_with("urn:factur-x.eu:1p0:basicwl")),
        (
            "a-extended.xml",
            a_with("urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended"),
        ),
        (
            "a-unknown-profile.xml",
            a_with("urn:example:unknown-profile"),
        ),
        // Spec 0010
        (
            "b-without-bt10.xml",
            replace_once(
                &b,
                "      <ram:BuyerReference>04011000-12345-34</ram:BuyerReference>\n",
                "",
            ),
        ),
        (
            "b-without-payment-means.xml",
            replace_once(
                &b,
                "      <ram:SpecifiedTradeSettlementPaymentMeans>
        <ram:TypeCode>58</ram:TypeCode>
        <ram:PayeePartyCreditorFinancialAccount>
          <ram:IBANID>DE02120300000000202051</ram:IBANID>
        </ram:PayeePartyCreditorFinancialAccount>
      </ram:SpecifiedTradeSettlementPaymentMeans>
",
                "",
            ),
        ),
        (
            "b-version-2-3.xml",
            b_with("urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_2.3"),
        ),
        (
            "b-extension.xml",
            b_with(
                "urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0#conformant#urn:xeinkauf.de:kosit:extension:xrechnung_3.0",
            ),
        ),
        (
            "b-cvd.xml",
            b_with(
                "urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0#compliant#urn:xeinkauf.de:kosit:xrechnung:cvd_0.9",
            ),
        ),
        (
            "r040-en16931.xml",
            with_specification_identifier(
                &fixture("kosit/cii/cii-peppol-en16931-r040.xml"),
                XRECHNUNG_ID,
                CEN_ID,
            ),
        ),
    ]
}

pub fn variant(name: &str) -> String {
    variants()
        .into_iter()
        .find(|(file, _)| *file == name)
        .map(|(_, xml)| xml)
        .unwrap_or_else(|| panic!("no variant {name}"))
}
