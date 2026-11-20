//! Spec 0021 — references, delivery and invoicing period, direct debit,
//! document-level allowances and charges, line period.

mod common;

use common::model::{field, model, section, sections};
use common::validation::validate;
use common::{fixture, replace_once};
use e_invoice_viewer_core::attribution::{Target, attribute};
use e_invoice_viewer_core::fields::{FieldModel, Section, map_invoice};
use e_invoice_viewer_core::i18n::Language;

fn cii_c() -> String {
    fixture("cii/sample-c.xml")
}

fn ubl_c() -> String {
    fixture("ubl/sample-c.xml")
}

fn english(xml: &str) -> FieldModel {
    map_invoice(xml.as_bytes(), Language::English).expect("field model")
}

fn ids_and_displays(section: &Section) -> Vec<(&str, &str)> {
    section
        .fields
        .iter()
        .map(|field| (field.id.as_str(), field.display.as_str()))
        .collect()
}

fn ids(section: &Section) -> Vec<&str> {
    section
        .fields
        .iter()
        .map(|field| field.id.as_str())
        .collect()
}

fn titles(model: &FieldModel) -> Vec<&str> {
    model
        .sections
        .iter()
        .map(|section| section.title.as_str())
        .collect()
}

fn position(model: &FieldModel, title: &str) -> usize {
    titles(model)
        .iter()
        .position(|t| *t == title)
        .unwrap_or_else(|| panic!("no section {title:?} in {:?}", titles(model)))
}

/// The id right before and right after `id` in the section.
fn neighbours<'a>(section: &'a Section, id: &str) -> (Option<&'a str>, Option<&'a str>) {
    let ids = ids(section);
    let at = ids.iter().position(|i| *i == id).expect(id);
    (at.checked_sub(1).map(|i| ids[i]), ids.get(at + 1).copied())
}

// AC1
#[test]
fn delivery_section_of_sample_c() {
    for xml in [cii_c(), ubl_c()] {
        let model = model(&xml);
        let delivery = position(&model, "Lieferung (BG-13)");
        assert_eq!(delivery, position(&model, "Käufer (BG-7)") + 1);
        assert_eq!(delivery + 1, position(&model, "Position 1"));
        assert_eq!(
            ids_and_displays(section(&model, "Lieferung (BG-13)")),
            [
                ("BT-70", "Lager Nord"),
                ("BT-72", "03.11.2026"),
                ("BT-73", "01.11.2026"),
                ("BT-74", "30.11.2026"),
                ("BT-75", "Industriestraße 5"),
                ("BT-78", "20457"),
                ("BT-77", "Hamburg"),
                ("BT-80", "DE"),
            ]
        );

        let model = english(&xml);
        let bt73 = field(section(&model, "Delivery information (BG-13)"), "BT-73");
        assert_eq!(bt73.label, "Invoicing period start date");
        assert_eq!(bt73.display, "2026-11-01");
    }
}

// AC2
#[test]
fn invoicing_period_alone_makes_a_delivery_section() {
    let cii = replace_once(
        &fixture("cii/sample-a.xml"),
        "      <ram:ApplicableTradeTax>\n        <ram:CalculatedAmount>",
        "      <ram:BillingSpecifiedPeriod>\n\
         \x20       <ram:StartDateTime><udt:DateTimeString format=\"102\">20261101</udt:DateTimeString></ram:StartDateTime>\n\
         \x20       <ram:EndDateTime><udt:DateTimeString format=\"102\">20261130</udt:DateTimeString></ram:EndDateTime>\n\
         \x20     </ram:BillingSpecifiedPeriod>\n\
         \x20     <ram:ApplicableTradeTax>\n        <ram:CalculatedAmount>",
    );
    // UBL sample A has no cac:Delivery at all.
    let ubl = replace_once(
        &fixture("ubl/sample-a.xml"),
        "  <cac:AccountingSupplierParty>",
        "  <cac:InvoicePeriod>\n\
         \x20   <cbc:StartDate>2026-11-01</cbc:StartDate>\n\
         \x20   <cbc:EndDate>2026-11-30</cbc:EndDate>\n\
         \x20 </cac:InvoicePeriod>\n\
         \x20 <cac:AccountingSupplierParty>",
    );
    for (name, xml) in [("CII", cii), ("UBL", ubl)] {
        let model = model(&xml);
        assert_eq!(
            ids_and_displays(section(&model, "Lieferung (BG-13)")),
            [("BT-73", "01.11.2026"), ("BT-74", "30.11.2026")],
            "{name}"
        );
    }
}

// AC3
#[test]
fn references_preceding_invoice_and_electronic_addresses() {
    for xml in [cii_c(), ubl_c()] {
        let model = model(&xml);
        let invoice = section(&model, "Rechnung");
        let references: Vec<(&str, &str)> = ids_and_displays(invoice)
            .into_iter()
            .skip_while(|(id, _)| *id != "BT-11")
            .take_while(|(id, _)| *id != "BT-24")
            .collect();
        assert_eq!(
            references,
            [
                ("BT-11", "PRJ-7"),
                ("BT-12", "V-2026-03"),
                ("BT-13", "PO-4711"),
                ("BT-18", "OBJ-1"),
            ]
        );
        let order = ids(invoice);
        let at = |id| order.iter().position(|i| *i == id).expect(id);
        assert!(at("BT-9") < at("BT-11") && at("BT-18") < at("BT-24"));

        let preceding = position(&model, "Vorausgegangene Rechnung (BG-3)");
        assert_eq!(preceding, position(&model, "Rechnung") + 1);
        assert_eq!(
            ids_and_displays(section(&model, "Vorausgegangene Rechnung (BG-3)")),
            [("BT-25", "RE-2026-0017"), ("BT-26", "15.10.2026")]
        );

        let seller = section(&model, "Verkäufer (BG-4)");
        assert_eq!(
            field(seller, "BT-34").display,
            "rechnung@muster.example (EM)"
        );
        assert_eq!(neighbours(seller, "BT-34"), (Some("BT-31"), Some("BT-35")));
        let buyer = section(&model, "Käufer (BG-7)");
        assert_eq!(field(buyer, "BT-49").display, "04011000-12345-34 (0204)");
        assert_eq!(neighbours(buyer, "BT-49").1, Some("BT-50"));

        let document = e_invoice_viewer_core::document::open(xml.as_bytes(), Language::German)
            .expect("document");
        let attachments: Vec<_> = document.attachments().collect();
        assert!(
            attachments.is_empty(),
            "BT-18 is not an attachment: {attachments:?}"
        );
    }
}

// AC4
#[test]
fn direct_debit_in_payment_section() {
    for xml in [cii_c(), ubl_c()] {
        let model = model(&xml);
        let payment = section(&model, "Zahlung (BG-16)");
        assert_eq!(
            ids_and_displays(payment),
            [
                ("BT-81", "SEPA-Lastschrift (59)"),
                ("BT-89", "MANDAT-0815"),
                ("BT-90", "DE98ZZZ09999999999"),
                ("BT-91", "DE02 1203 0000 0000 2020 51"),
            ]
        );
    }
    let seller_ids: Vec<String> = section(&model(&ubl_c()), "Verkäufer (BG-4)")
        .fields
        .iter()
        .filter(|field| field.id == "BT-29")
        .map(|field| field.raw.clone())
        .collect();
    assert_eq!(seller_ids, Vec::<String>::new());
}

// AC5
#[test]
fn allowances_and_charges() {
    for xml in [cii_c(), ubl_c()] {
        let model = model(&xml);
        let allowance = position(&model, "Nachlass (BG-20)");
        assert_eq!(allowance, position(&model, "Zahlung (BG-16)") + 1);
        assert_eq!(allowance + 1, position(&model, "Zuschlag (BG-21)"));
        assert_eq!(allowance + 2, position(&model, "Umsatzsteuer (BG-23)"));
        assert_eq!(
            ids_and_displays(section(&model, "Nachlass (BG-20)")),
            [
                ("BT-92", "50,00 EUR"),
                ("BT-95", "Normalsatz (S)"),
                ("BT-96", "19"),
                ("BT-97", "Rabatt"),
            ]
        );
        assert_eq!(
            ids_and_displays(section(&model, "Zuschlag (BG-21)")),
            [
                ("BT-99", "10,00 EUR"),
                ("BT-102", "Normalsatz (S)"),
                ("BT-103", "19"),
                ("BT-104", "Fracht"),
            ]
        );
        let totals = section(&model, "Summen (BG-22)");
        for (id, display) in [
            ("BT-107", "50,00 EUR"),
            ("BT-108", "10,00 EUR"),
            ("BT-109", "1.010,00 EUR"),
            ("BT-112", "1.201,90 EUR"),
        ] {
            assert_eq!(field(totals, id).display, display, "{id}");
        }

        let model = english(&xml);
        let amount = |title, id| field(section(&model, title), id).display.clone();
        assert_eq!(
            amount("Document level allowances (BG-20)", "BT-92"),
            "50.00 EUR"
        );
        assert_eq!(
            amount("Document level charges (BG-21)", "BT-99"),
            "10.00 EUR"
        );
    }
}

fn cii_c_with_second_allowance() -> String {
    replace_once(
        &cii_c(),
        "        <ram:ActualAmount>50.00</ram:ActualAmount>\n        <ram:Reason>Rabatt</ram:Reason>\n        <ram:CategoryTradeTax>\n          <ram:TypeCode>VAT</ram:TypeCode>\n          <ram:CategoryCode>S</ram:CategoryCode>\n          <ram:RateApplicablePercent>19</ram:RateApplicablePercent>\n        </ram:CategoryTradeTax>\n      </ram:SpecifiedTradeAllowanceCharge>\n",
        "        <ram:ActualAmount>50.00</ram:ActualAmount>\n        <ram:Reason>Rabatt</ram:Reason>\n        <ram:CategoryTradeTax>\n          <ram:TypeCode>VAT</ram:TypeCode>\n          <ram:CategoryCode>S</ram:CategoryCode>\n          <ram:RateApplicablePercent>19</ram:RateApplicablePercent>\n        </ram:CategoryTradeTax>\n      </ram:SpecifiedTradeAllowanceCharge>\n      <ram:SpecifiedTradeAllowanceCharge>\n        <ram:ChargeIndicator>\n          <udt:Indicator>false</udt:Indicator>\n        </ram:ChargeIndicator>\n        <ram:ActualAmount>5.00</ram:ActualAmount>\n        <ram:Reason>Skonto</ram:Reason>\n        <ram:CategoryTradeTax>\n          <ram:TypeCode>VAT</ram:TypeCode>\n          <ram:CategoryCode>S</ram:CategoryCode>\n          <ram:RateApplicablePercent>19</ram:RateApplicablePercent>\n        </ram:CategoryTradeTax>\n      </ram:SpecifiedTradeAllowanceCharge>\n",
    )
}

fn ubl_c_with_second_allowance() -> String {
    replace_once(
        &ubl_c(),
        "  <cac:AllowanceCharge>\n    <cbc:ChargeIndicator>true</cbc:ChargeIndicator>",
        "  <cac:AllowanceCharge>\n    <cbc:ChargeIndicator>false</cbc:ChargeIndicator>\n    <cbc:AllowanceChargeReason>Skonto</cbc:AllowanceChargeReason>\n    <cbc:Amount currencyID=\"EUR\">5.00</cbc:Amount>\n    <cac:TaxCategory>\n      <cbc:ID>S</cbc:ID>\n      <cbc:Percent>19</cbc:Percent>\n      <cac:TaxScheme>\n        <cbc:ID>VAT</cbc:ID>\n      </cac:TaxScheme>\n    </cac:TaxCategory>\n  </cac:AllowanceCharge>\n  <cac:AllowanceCharge>\n    <cbc:ChargeIndicator>true</cbc:ChargeIndicator>",
    )
}

// AC5
#[test]
fn two_allowances_give_two_sections_in_document_order() {
    for xml in [cii_c_with_second_allowance(), ubl_c_with_second_allowance()] {
        let model = model(&xml);
        let allowances = sections(&model, "Nachlass (BG-20)");
        let amounts: Vec<&str> = allowances
            .iter()
            .map(|section| field(section, "BT-92").display.as_str())
            .collect();
        assert_eq!(amounts, ["50,00 EUR", "5,00 EUR"]);
        assert!(allowances.iter().all(|section| section.repeated));
    }
}

// AC6
#[test]
fn line_period_and_order_line_reference() {
    for xml in [cii_c(), ubl_c()] {
        let model = model(&xml);
        let line = section(&model, "Position 1");
        let order = ids(line);
        assert_eq!(order[order.len() - 2..], ["BT-134", "BT-135"]);
        assert_eq!(field(line, "BT-132").display, "1");
        assert_eq!(field(line, "BT-134").display, "01.11.2026");
        assert_eq!(field(line, "BT-135").display, "15.11.2026");
        // BT-128 is absent in sample C; BT-132 comes after the fields before it.
        let bt132 = order.iter().position(|id| *id == "BT-132").unwrap();
        let bt131 = order.iter().position(|id| *id == "BT-131").unwrap();
        assert!(bt131 < bt132);
        let line2 = section(&model, "Position 2");
        assert!(!ids(line2).contains(&"BT-134"));
    }
}

/// (section title, field id, display) of every field, in order.
fn displays(model: &FieldModel) -> Vec<(String, String, String)> {
    model
        .sections
        .iter()
        .flat_map(|section| {
            // Other data differs between the syntaxes (spec 0031).
            section
                .fields
                .iter()
                .filter(|field| !field.id.is_empty())
                .map(|field| {
                    (
                        section.title.clone(),
                        field.id.clone(),
                        field.display.clone(),
                    )
                })
        })
        .collect()
}

// AC7
#[test]
fn ubl_and_cii_sample_c_give_same_fields() {
    assert_eq!(displays(&model(&ubl_c())), displays(&model(&cii_c())));
    assert_eq!(displays(&english(&ubl_c())), displays(&english(&cii_c())));
}

fn target_title(model: &FieldModel, target: Target) -> (String, Option<String>) {
    match target {
        Target::Field { section, field } => (
            model.sections[section].title.clone(),
            Some(model.sections[section].fields[field].id.clone()),
        ),
        Target::Section(section) => (model.sections[section].title.clone(), None),
        Target::Document => ("Dokument".to_string(), None),
    }
}

// AC8
#[test]
fn findings_at_new_fields() {
    let cii_period = replace_once(
        &cii_c(),
        "<udt:DateTimeString format=\"102\">20261130</udt:DateTimeString>",
        "<udt:DateTimeString format=\"102\">20261031</udt:DateTimeString>",
    );
    let ubl_period = replace_once(
        &ubl_c(),
        "<cbc:EndDate>2026-11-30</cbc:EndDate>",
        "<cbc:EndDate>2026-10-31</cbc:EndDate>",
    );
    for xml in [cii_period, ubl_period] {
        let model = model(&xml);
        let validation = validate(&xml);
        let br29 = validation
            .findings
            .iter()
            .find(|finding| finding.rule_id == "BR-29")
            .unwrap_or_else(|| panic!("BR-29 in {:#?}", validation.findings));
        let (title, id) = target_title(&model, attribute(&model, br29));
        assert_eq!(title, "Lieferung (BG-13)");
        assert!(matches!(id.as_deref(), Some("BT-73" | "BT-74")), "{id:?}");
    }

    let cii_no_reason = replace_once(&cii_c(), "        <ram:Reason>Rabatt</ram:Reason>\n", "");
    let ubl_no_reason = replace_once(
        &ubl_c(),
        "    <cbc:AllowanceChargeReason>Rabatt</cbc:AllowanceChargeReason>\n",
        "",
    );
    for xml in [cii_no_reason, ubl_no_reason] {
        let model = model(&xml);
        let validation = validate(&xml);
        let br33 = validation
            .findings
            .iter()
            .find(|finding| finding.rule_id == "BR-33")
            .unwrap_or_else(|| panic!("BR-33 in {:#?}", validation.findings));
        let (title, _) = target_title(&model, attribute(&model, br33));
        assert_eq!(title, "Nachlass (BG-20)");
    }
}

// AC9
#[test]
fn new_field_locations_select_their_elements() {
    use common::select;
    use e_invoice_viewer_core::link::{XmlLayout, assignment};

    for xml in [cii_c_with_second_allowance(), ubl_c_with_second_allowance()] {
        let model = model(&xml);
        let layout = XmlLayout::new(xml.as_bytes()).expect("layout");
        for (title, nth, id) in [
            ("Lieferung (BG-13)", 0, "BT-73"),
            ("Zahlung (BG-16)", 0, "BT-91"),
            ("Nachlass (BG-20)", 1, "BT-92"),
            ("Position 1", 0, "BT-135"),
        ] {
            let field = field(sections(&model, title)[nth], id);
            assert_eq!(
                select(&xml, &field.location),
                std::slice::from_ref(&field.raw),
                "{id}"
            );
            // The XML view names the field for its element (0014).
            let range = layout.element_range(&field.location).expect("range");
            let inside = range.start + layout.text()[range.clone()].find('>').unwrap() + 1;
            match assignment(&model, &layout, inside) {
                Target::Field { section, field: f } => {
                    assert_eq!(model.sections[section].fields[f].id, id);
                    assert_eq!(model.sections[section].title, title);
                }
                other => panic!("{id}: {other:?}"),
            }
        }
    }
}
