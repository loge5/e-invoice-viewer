//! Spec 0027 — ZUGFeRD/Factur-X invoices are checked against the schema and
//! the rules of their profile. `validate_embedded` is what the app calls for
//! the invoice of a PDF.

mod common;

use common::validation::{CII_RULES, CII_SCHEMA, errors, findings_with, validate, variant};
use common::{fixture, replace_once};
use e_invoice_viewer_core::i18n::Language;
use e_invoice_viewer_core::validate::{SCHEMA_RULE_ID, Severity, Validation, Validator};

fn embedded(xml: &str) -> Validation {
    Validator::new()
        .validate_embedded(xml.as_bytes(), Language::German)
        .expect("validation result")
}

const EN16931_SCHEMA: &str = "XML-Schema Factur-X 1.09.2 EN 16931";
const EN16931_RULES: &str = "Factur-X 1.09.2 (ZUGFeRD 2.5.2) EN 16931";

#[test]
fn sample_a_in_a_pdf_gets_the_en16931_profile() {
    let validation = embedded(&fixture("cii/sample-a.xml"));
    assert_eq!(
        validation.applied_rule_sets,
        [EN16931_SCHEMA, EN16931_RULES, CII_RULES]
    );
    // Sample A has an empty delivery element: a warning of the profile rules.
    assert_eq!(
        errors(&validation),
        Vec::<&_>::new(),
        "{:#?}",
        validation.findings
    );
}

// AC3
#[test]
fn extended_element_in_en16931_profile() {
    let xml = replace_once(
        &fixture("cii/sample-a.xml"),
        "    <ram:ID>RE-2026-0042</ram:ID>\n",
        "    <ram:ID>RE-2026-0042</ram:ID>\n    <ram:Name>RECHNUNG</ram:Name>\n",
    );
    let validation = embedded(&xml);
    let schema = findings_with(&validation, SCHEMA_RULE_ID);
    assert_eq!(schema.len(), 1, "{:#?}", validation.findings);
    assert!(
        schema[0].message.contains("ram:Name"),
        "{}",
        schema[0].message
    );
    assert_eq!(
        schema[0].location,
        "/rsm:CrossIndustryInvoice/rsm:ExchangedDocument/ram:Name"
    );
    // The same element is allowed in the Extended profile.
    let extended = replace_once(
        &xml,
        "<ram:ID>urn:cen.eu:en16931:2017</ram:ID>",
        "<ram:ID>urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended</ram:ID>",
    );
    assert_eq!(
        findings_with(&embedded(&extended), SCHEMA_RULE_ID),
        Vec::<&_>::new()
    );
}

// AC4
#[test]
fn minimum_profile_rules() {
    let xml = fixture("cii/sample-minimum.xml");
    let validation = embedded(&xml);
    assert_eq!(
        validation.applied_rule_sets,
        [
            "XML-Schema Factur-X 1.09.2 Minimum",
            "Factur-X 1.09.2 (ZUGFeRD 2.5.2) Minimum"
        ]
    );
    assert!(validation.not_applicable_reason.is_some());
    assert_eq!(validation.findings, [], "{:#?}", validation.findings);

    // BR-DEC-12 of the Minimum rules: at most two decimals in BT-109.
    let three_decimals = replace_once(
        &xml,
        "<ram:TaxBasisTotalAmount>1050.00</ram:TaxBasisTotalAmount>",
        "<ram:TaxBasisTotalAmount>1050.001</ram:TaxBasisTotalAmount>",
    );
    let validation = embedded(&three_decimals);
    let br_dec_12 = findings_with(&validation, "BR-DEC-12");
    assert_eq!(br_dec_12.len(), 1, "{:#?}", validation.findings);
    assert_eq!(br_dec_12[0].severity, Severity::Error);
}

/// Sample A with the Basic profile and two preceding invoices (BG-3),
/// without the seller contact, which Basic does not have.
fn basic_with_two_preceding_invoices(identifier: &str) -> String {
    let a = fixture("cii/sample-a.xml");
    let contact_start = a
        .find("        <ram:DefinedTradeContact>")
        .expect("contact");
    let contact_end = a
        .find("        </ram:DefinedTradeContact>\n")
        .expect("contact end")
        + "        </ram:DefinedTradeContact>\n".len();
    let without_contact = format!("{}{}", &a[..contact_start], &a[contact_end..]);
    let xml = replace_once(
        &without_contact,
        "<ram:ID>urn:cen.eu:en16931:2017</ram:ID>",
        &format!("<ram:ID>{identifier}</ram:ID>"),
    );
    replace_once(
        &xml,
        "      </ram:SpecifiedTradeSettlementHeaderMonetarySummation>\n",
        "      </ram:SpecifiedTradeSettlementHeaderMonetarySummation>
      <ram:InvoiceReferencedDocument>
        <ram:IssuerAssignedID>RE-2026-0040</ram:IssuerAssignedID>
      </ram:InvoiceReferencedDocument>
      <ram:InvoiceReferencedDocument>
        <ram:IssuerAssignedID>RE-2026-0041</ram:IssuerAssignedID>
      </ram:InvoiceReferencedDocument>
",
    )
}

// AC5
#[test]
fn two_preceding_invoices_d22b() {
    let basic = basic_with_two_preceding_invoices(
        "urn:cen.eu:en16931:2017#compliant#urn:factur-x.eu:1p0:basic",
    );
    let validation = embedded(&basic);
    assert_eq!(
        validation.applied_rule_sets[0],
        "XML-Schema Factur-X 1.09.2 Basic"
    );
    assert_eq!(
        findings_with(&validation, SCHEMA_RULE_ID),
        Vec::<&_>::new(),
        "{:#?}",
        validation.findings
    );

    // As a plain XML file with the CEN identifier: CII D16B allows one.
    let plain = validate(&basic_with_two_preceding_invoices(
        "urn:cen.eu:en16931:2017",
    ));
    assert_eq!(plain.applied_rule_sets, [CII_SCHEMA, CII_RULES]);
    assert_eq!(
        findings_with(&plain, SCHEMA_RULE_ID).len(),
        1,
        "{:#?}",
        plain.findings
    );
}

// AC6
#[test]
fn report_without_id() {
    let xml = replace_once(
        &fixture("cii/sample-a.xml"),
        "<ram:ID>RE-2026-0042</ram:ID>",
        "<ram:ID schemeID=\"0088\">RE-2026-0042</ram:ID>",
    );
    let validation = embedded(&xml);
    let fx = findings_with(&validation, "FX");
    assert_eq!(fx.len(), 1, "{:#?}", validation.findings);
    assert_eq!(
        fx[0].message,
        "Attribute @schemeID' marked as not used in the given context."
    );
    assert_eq!(fx[0].severity, Severity::Error);
    assert_eq!(
        fx[0].location,
        "/rsm:CrossIndustryInvoice/rsm:ExchangedDocument/ram:ID"
    );
}

// AC7
#[test]
fn rule_in_cen_and_profile_reported_once() {
    let validation = embedded(&variant("a-grand-total-1300.xml"));
    assert!(
        validation
            .applied_rule_sets
            .contains(&EN16931_RULES.to_string())
    );
    assert_eq!(
        findings_with(&validation, "BR-CO-15").len(),
        1,
        "{:#?}",
        validation.findings
    );
}

// AC8: a plain XML file with the CEN identifier is checked as before.
#[test]
fn plain_xml_with_cen_identifier_unchanged() {
    let validation = validate(&fixture("cii/sample-a.xml"));
    assert_eq!(validation.applied_rule_sets, [CII_SCHEMA, CII_RULES]);
}
