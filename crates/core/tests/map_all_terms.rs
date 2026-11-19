//! Spec 0024 — every remaining EN 16931 business term.

mod common;

use std::collections::BTreeSet;

use common::model::{field, model, section};
use common::validation::validate;
use common::{fixture, replace_once};
use e_invoice_viewer_core::attribution::{Target, attribute};
use e_invoice_viewer_core::fields::{FieldModel, Section, map_invoice};
use e_invoice_viewer_core::i18n::Language;

fn cii_d() -> String {
    fixture("cii/sample-d.xml")
}

fn ubl_d() -> String {
    fixture("ubl/sample-d.xml")
}

fn titles(model: &FieldModel) -> Vec<&str> {
    model.sections.iter().map(|s| s.title.as_str()).collect()
}

fn position(model: &FieldModel, title: &str) -> usize {
    titles(model)
        .iter()
        .position(|t| *t == title)
        .unwrap_or_else(|| panic!("no section {title:?} in {:?}", titles(model)))
}

fn ids(section: &Section) -> Vec<&str> {
    section.fields.iter().map(|f| f.id.as_str()).collect()
}

fn ids_and_displays(section: &Section) -> Vec<(&str, &str)> {
    section
        .fields
        .iter()
        .map(|f| (f.id.as_str(), f.display.as_str()))
        .collect()
}

// AC1
#[test]
fn payee_tax_representative_buyer_contact_and_document_fields() {
    for xml in [cii_d(), ubl_d()] {
        let model = model(&xml);
        let payee = position(&model, "Zahlungsempfänger (BG-10)");
        assert_eq!(payee, position(&model, "Käufer (BG-7)") + 1);
        assert_eq!(payee + 1, position(&model, "Steuervertreter (BG-11)"));
        assert_eq!(
            ids_and_displays(section(&model, "Zahlungsempfänger (BG-10)")),
            [
                ("BT-59", "Factoring Nord GmbH"),
                ("BT-60", "4000001555559 (0088)"),
                ("BT-61", "4000001666663 (0088)"),
            ]
        );
        assert_eq!(
            ids(section(&model, "Steuervertreter (BG-11)")),
            [
                "BT-62", "BT-63", "BT-64", "BT-65", "BT-164", "BT-67", "BT-66", "BT-68", "BT-69"
            ]
        );
        let buyer = section(&model, "Käufer (BG-7)");
        let order = ids(buyer);
        assert_eq!(order[order.len() - 3..], ["BT-56", "BT-57", "BT-58"]);
        assert_eq!(field(buyer, "BT-56").display, "Max Beispiel");

        let invoice = section(&model, "Rechnung");
        for (id, display) in [
            ("BT-6", "USD"),
            ("BT-7", "03.11.2026"),
            ("BT-23", "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"),
        ] {
            assert_eq!(field(invoice, id).display, display, "{id}");
        }
        let order = ids(invoice);
        let note = order.iter().position(|id| *id == "BT-22").expect("note");
        assert_eq!(order[note - 1], "BT-21");
        assert_eq!(field(invoice, "BT-21").display, "AAI");
        assert_eq!(
            field(invoice, "BT-22").display,
            "Bitte den Lieferschein beachten."
        );
    }
}

// AC1: BT-8 instead of BT-7 (both together are not allowed, BR-CO-3)
#[test]
fn tax_point_date_code() {
    let cii = replace_once(
        &cii_d(),
        "        <ram:TaxPointDate>\n          <udt:DateString format=\"102\">20261103</udt:DateString>\n        </ram:TaxPointDate>\n",
        "        <ram:DueDateTypeCode>5</ram:DueDateTypeCode>\n",
    );
    let ubl = replace_once(
        &replace_once(
            &ubl_d(),
            "  <cbc:TaxPointDate>2026-11-03</cbc:TaxPointDate>\n",
            "",
        ),
        "    <cbc:EndDate>2026-11-30</cbc:EndDate>\n",
        "    <cbc:EndDate>2026-11-30</cbc:EndDate>\n    <cbc:DescriptionCode>3</cbc:DescriptionCode>\n",
    );
    for (xml, code) in [(cii, "5"), (ubl, "3")] {
        let model = model(&xml);
        let invoice = section(&model, "Rechnung");
        assert_eq!(field(invoice, "BT-8").display, code);
        assert!(!ids(invoice).contains(&"BT-7"));
    }
}

// AC2
#[test]
fn line_allowances_and_charges_follow_their_line() {
    for xml in [cii_d(), ubl_d()] {
        let model = model(&xml);
        let line = position(&model, "Position 1");
        assert_eq!(line + 1, position(&model, "Position 1 – Nachlass (BG-27)"));
        assert_eq!(line + 2, position(&model, "Position 1 – Zuschlag (BG-28)"));
        assert_eq!(line + 3, position(&model, "Position 2"));
        assert_eq!(
            ids_and_displays(section(&model, "Position 1 – Nachlass (BG-27)")),
            [("BT-136", "20,00 EUR"), ("BT-139", "Mengenrabatt")]
        );
        assert_eq!(
            ids_and_displays(section(&model, "Position 1 – Zuschlag (BG-28)")),
            [("BT-141", "20,00 EUR"), ("BT-144", "Expresszuschlag")]
        );
        assert!(section(&model, "Position 1 – Nachlass (BG-27)").repeated);
    }
}

// AC3
#[test]
fn line_fields_and_item_attributes() {
    for xml in [cii_d(), ubl_d()] {
        let model = model(&xml);
        let line = section(&model, "Position 1");
        for (id, display) in [
            ("BT-148", "130,00 EUR"),
            ("BT-147", "5,00 EUR"),
            ("BT-150", "HUR"),
            ("BT-156", "K-100"),
            ("BT-157", "4000001000005 (0160)"),
            ("BT-158", "80101500 (TST)"),
            ("BT-159", "DE"),
        ] {
            assert_eq!(field(line, id).display, display, "{id}");
        }
        let attributes: Vec<(&str, &str)> = line
            .fields
            .iter()
            .filter(|f| f.id == "BT-161")
            .map(|f| (f.label.as_str(), f.display.as_str()))
            .collect();
        assert_eq!(attributes, [("Ort", "vor Ort"), ("Stufe", "Senior")]);
    }
}

// AC4
#[test]
fn totals_in_accounting_currency_and_payment_fields() {
    for xml in [cii_d(), ubl_d()] {
        let model = model(&xml);
        let totals = section(&model, "Summen (BG-22)");
        assert_eq!(field(totals, "BT-110").display, "182,40 EUR");
        assert_eq!(field(totals, "BT-111").display, "197,00 USD");
        let payment = section(&model, "Zahlung (BG-16)");
        for (id, display) in [
            ("BT-82", "SEPA-Lastschrift"),
            ("BT-87", "1234"),
            ("BT-88", "Max Beispiel"),
        ] {
            assert_eq!(field(payment, id).display, display, "{id}");
        }
    }
}

/// (section title, field id, label, display) of every field, in order.
fn everything(model: &FieldModel) -> Vec<(String, String, String, String)> {
    model
        .sections
        .iter()
        .flat_map(|s| {
            s.fields.iter().map(|f| {
                (
                    s.title.clone(),
                    f.id.clone(),
                    f.label.clone(),
                    f.display.clone(),
                )
            })
        })
        .collect()
}

// AC5
#[test]
fn ubl_and_cii_sample_d_give_same_fields() {
    for language in [Language::German, Language::English] {
        let cii = map_invoice(cii_d().as_bytes(), language).expect("CII");
        let ubl = map_invoice(ubl_d().as_bytes(), language).expect("UBL");
        assert_eq!(everything(&ubl), everything(&cii), "{language:?}");
    }
}

fn target_title(model: &FieldModel, target: Target) -> String {
    match target {
        Target::Field { section, .. } | Target::Section(section) => {
            model.sections[section].title.clone()
        }
        Target::Document => "Dokument".to_string(),
    }
}

// AC6
#[test]
fn findings_at_line_allowances_and_payee() {
    let cii_br41 = replace_once(
        &cii_d(),
        "          <ram:ActualAmount>20.00</ram:ActualAmount>\n          <ram:Reason>Mengenrabatt</ram:Reason>\n",
        "          <ram:Reason>Mengenrabatt</ram:Reason>\n",
    );
    let cii_br17 = replace_once(
        &cii_d(),
        "        <ram:Name>Factoring Nord GmbH</ram:Name>\n",
        "",
    );
    for (xml, rule, title) in [
        (cii_br41, "BR-41", "Position 1 – Nachlass (BG-27)"),
        (cii_br17, "BR-17", "Zahlungsempfänger (BG-10)"),
    ] {
        let model = model(&xml);
        let validation = validate(&xml);
        let finding = validation
            .findings
            .iter()
            .find(|f| f.rule_id == rule)
            .unwrap_or_else(|| panic!("{rule} in {:#?}", validation.findings));
        assert_eq!(
            target_title(&model, attribute(&model, finding)),
            title,
            "{rule}"
        );
    }
}

/// The business terms of EN 16931: BT-1 to BT-165 without BT-4, which does
/// not exist.
fn en16931_terms() -> BTreeSet<String> {
    (1..=165)
        .filter(|n| *n != 4)
        .map(|n| format!("BT-{n}"))
        .collect()
}

/// The business term ids the mapping of a syntax can produce, read from its
/// source (every `BT_<n>` constant used there).
fn mapped_terms(source: &str) -> BTreeSet<String> {
    let mut terms = BTreeSet::new();
    for (index, _) in source.match_indices("BT_") {
        let digits: String = source[index + 3..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if !digits.is_empty() {
            terms.insert(format!("BT-{digits}"));
        }
    }
    terms
}

// AC7
#[test]
fn every_business_term_is_mapped() {
    // Shown elsewhere: the supporting documents (BG-24) in the attachments
    // section (0013); the item attribute name (BT-160) as the label of its
    // value (BT-161).
    let elsewhere: BTreeSet<String> = ["BT-122", "BT-123", "BT-124", "BT-125", "BT-160"]
        .into_iter()
        .map(str::to_string)
        .collect();
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/fields");
    for syntax in ["cii.rs", "ubl.rs"] {
        let source = std::fs::read_to_string(format!("{dir}/{syntax}")).expect(syntax);
        let mapped = mapped_terms(&source);
        let missing: Vec<String> = en16931_terms()
            .into_iter()
            .filter(|term| !mapped.contains(term) && !elsewhere.contains(term))
            .collect();
        assert!(missing.is_empty(), "{syntax} does not map {missing:?}");
    }
}

// AC8: samples A, B, C keep their models apart from the new terms they do
// not have; checked by the existing tests of 0004–0006 and 0021, which pass
// unchanged. Sample D is valid:
#[test]
fn sample_d_has_no_errors() {
    for path in ["cii/sample-d.xml", "ubl/sample-d.xml"] {
        let validation = validate(&fixture(path));
        let errors: Vec<_> = validation
            .findings
            .iter()
            .filter(|f| f.severity == e_invoice_viewer_core::validate::Severity::Error)
            .collect();
        assert!(errors.is_empty(), "{path}: {errors:#?}");
    }
}
