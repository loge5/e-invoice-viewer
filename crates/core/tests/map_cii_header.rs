//! Spec 0004 — Map CII invoice header data to the field model.

mod common;

use common::model::{field, model, section};
use common::{fixture, replace_once, select};
use e_invoice_viewer_core::fields::Section;

fn sample_a() -> String {
    fixture("cii/sample-a.xml")
}

fn sample_b() -> String {
    fixture("cii/sample-b.xml")
}

fn ids_and_displays(section: &Section) -> Vec<(&str, &str)> {
    section
        .fields
        .iter()
        .map(|field| (field.id.as_str(), field.display.as_str()))
        .collect()
}

/// Display value of `id` in `section` after replacing `from` with `to` in
/// sample A.
fn display_in_variant(from: &str, to: &str, section_title: &str, id: &str) -> String {
    let xml = replace_once(&sample_a(), from, to);
    field(section(&model(&xml), section_title), id)
        .display
        .clone()
}

// Contract: section order
#[test]
fn sample_a_sections_in_contract_order() {
    let titles: Vec<_> = model(&sample_a())
        .sections
        .iter()
        .map(|section| section.title.clone())
        .filter(|title| !title.starts_with("Position "))
        .collect();

    assert_eq!(
        titles,
        [
            "Rechnung",
            "Verkäufer (BG-4)",
            "Käufer (BG-7)",
            "Zahlung (BG-16)",
            "Umsatzsteuer (BG-23)",
            "Summen (BG-22)",
        ]
    );
}

// AC1
#[test]
fn sample_a_header_section_in_order() {
    let model = model(&sample_a());
    let header = section(&model, "Rechnung");

    assert_eq!(
        ids_and_displays(header),
        [
            ("BT-1", "RE-2026-0042"),
            ("BT-2", "17.11.2026"),
            ("BT-3", "Rechnung (380)"),
            ("BT-5", "EUR"),
            ("BT-9", "01.12.2026"),
            ("BT-24", "EN 16931"),
        ]
    );
    assert_eq!(field(header, "BT-2").raw, "20261117");
}

// AC2
#[test]
fn sample_a_seller_section_flat_in_order() {
    let model = model(&sample_a());

    assert_eq!(
        ids_and_displays(section(&model, "Verkäufer (BG-4)")),
        [
            ("BT-27", "Muster GmbH"),
            ("BT-31", "DE123456789"),
            // Spec 0021: the seller's electronic address.
            ("BT-34", "rechnung@muster.example (EM)"),
            ("BT-35", "Hauptstraße 1"),
            ("BT-38", "12345"),
            ("BT-37", "Berlin"),
            ("BT-40", "DE"),
            ("BT-41", "Erika Mustermann"),
            ("BT-42", "+49 30 1234567"),
            ("BT-43", "rechnung@muster.example"),
        ]
    );
}

// AC3
#[test]
fn sample_a_payment_section() {
    let model = model(&sample_a());
    let payment = section(&model, "Zahlung (BG-16)");

    assert_eq!(field(payment, "BT-81").display, "SEPA-Überweisung (58)");
    assert_eq!(
        field(payment, "BT-84").display,
        "DE02 1203 0000 0000 2020 51"
    );
    assert_eq!(field(payment, "BT-84").raw, "DE02120300000000202051");
}

// AC3
#[test]
fn sample_a_totals_section() {
    let model = model(&sample_a());
    let totals = section(&model, "Summen (BG-22)");

    assert_eq!(field(totals, "BT-106").display, "1.050,00 EUR");
    assert_eq!(field(totals, "BT-110").display, "199,50 EUR");
    assert_eq!(field(totals, "BT-112").display, "1.249,50 EUR");
    assert_eq!(field(totals, "BT-115").display, "1.249,50 EUR");
}

// AC4
#[test]
fn sample_b_two_vat_breakdowns_in_document_order() {
    let model = model(&sample_b());
    let breakdowns: Vec<_> = model
        .sections
        .iter()
        .filter(|section| section.title == "Umsatzsteuer (BG-23)")
        .collect();

    assert_eq!(breakdowns.len(), 2);
    assert_eq!(field(breakdowns[0], "BT-118").display, "Normalsatz (S)");
    assert_eq!(field(breakdowns[0], "BT-119").display, "19");
    assert_eq!(field(breakdowns[1], "BT-118").display, "Steuerbefreit (E)");
    assert_eq!(field(breakdowns[1], "BT-119").display, "0");
    assert_eq!(
        field(breakdowns[1], "BT-120").display,
        "Steuerbefreit nach § 4 Nr. 21 UStG"
    );
}

// AC4
#[test]
fn sample_b_header_has_buyer_reference_and_note() {
    let model = model(&sample_b());
    let header = section(&model, "Rechnung");

    assert_eq!(field(header, "BT-10").display, "04011000-12345-34");
    assert_eq!(
        field(header, "BT-22").display,
        "Vielen Dank für Ihren Auftrag."
    );
}

// AC5 (rows 1-3)
#[test]
fn bt24_display_uses_profile_name() {
    let id = "<ram:ID>urn:cen.eu:en16931:2017</ram:ID>";
    let cases = [
        (
            "urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0",
            "XRechnung 3.0",
        ),
        (
            "urn:cen.eu:en16931:2017#conformant#urn:xeinkauf.de:kosit:extension:xrechnung_3.0",
            "XRechnung 3.0 (Extension)",
        ),
        (
            "urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0#conformant#urn:xeinkauf.de:kosit:extension:xrechnung_3.0",
            "XRechnung 3.0 (Extension)",
        ),
        ("urn:example:custom-profile", "urn:example:custom-profile"),
    ];

    for (identifier, expected) in cases {
        let display = display_in_variant(
            id,
            &format!("<ram:ID>{identifier}</ram:ID>"),
            "Rechnung",
            "BT-24",
        );
        assert_eq!(display, expected, "BT-24 {identifier}");
    }
}

// AC5 (rows 4-7)
#[test]
fn amount_display_rules() {
    let due = "<ram:DuePayableAmount>1249.50</ram:DuePayableAmount>";
    for (raw, expected) in [
        ("12.3456", "12,3456 EUR"),
        ("-50.00", "-50,00 EUR"),
        ("1000", "1.000,00 EUR"),
    ] {
        let display = display_in_variant(
            due,
            &format!("<ram:DuePayableAmount>{raw}</ram:DuePayableAmount>"),
            "Summen (BG-22)",
            "BT-115",
        );
        assert_eq!(display, expected, "BT-115 {raw}");
    }

    let xml = replace_once(
        &replace_once(
            &sample_a(),
            "<ram:InvoiceCurrencyCode>EUR</ram:InvoiceCurrencyCode>",
            "<ram:InvoiceCurrencyCode>CHF</ram:InvoiceCurrencyCode>",
        ),
        due,
        "<ram:DuePayableAmount>1000.00</ram:DuePayableAmount>",
    );
    let model = model(&xml);
    assert_eq!(
        field(section(&model, "Summen (BG-22)"), "BT-115").display,
        "1.000,00 CHF"
    );
}

// AC5 (rows 8-9)
#[test]
fn date_display_falls_back_to_raw() {
    let issue = "<udt:DateTimeString format=\"102\">20261117</udt:DateTimeString>";
    let cases = [
        (
            "<udt:DateTimeString format=\"610\">202611</udt:DateTimeString>",
            "202611",
        ),
        (
            "<udt:DateTimeString format=\"102\">20261345</udt:DateTimeString>",
            "20261345",
        ),
    ];

    for (replacement, expected) in cases {
        let display = display_in_variant(issue, replacement, "Rechnung", "BT-2");
        assert_eq!(display, expected);
    }
}

// AC6
#[test]
fn section_without_fields_is_omitted() {
    let original = model(&sample_a());
    let payment_means = &sample_a()[sample_a()
        .find("      <ram:SpecifiedTradeSettlementPaymentMeans>")
        .unwrap()
        ..sample_a()
            .find("      <ram:ApplicableTradeTax>\n        <ram:CalculatedAmount>")
            .unwrap()];
    let without_payment = model(&replace_once(&sample_a(), payment_means, ""));

    assert!(
        without_payment
            .sections
            .iter()
            .all(|section| section.title != "Zahlung (BG-16)")
    );
    let others: Vec<_> = original
        .sections
        .iter()
        .filter(|section| section.title != "Zahlung (BG-16)")
        .cloned()
        .collect();
    assert_eq!(without_payment.sections, others);
}

// AC7
#[test]
fn locations_select_the_source_elements() {
    let a = sample_a();
    let model_a = model(&a);
    let b = sample_b();
    let model_b = model(&b);
    let second_breakdown = model_b
        .sections
        .iter()
        .filter(|section| section.title == "Umsatzsteuer (BG-23)")
        .nth(1)
        .expect("second BG-23");

    let checks = [
        (&a, field(section(&model_a, "Rechnung"), "BT-1")),
        (&a, field(section(&model_a, "Zahlung (BG-16)"), "BT-84")),
        (&b, field(second_breakdown, "BT-119")),
    ];

    for (xml, field) in checks {
        assert_eq!(
            select(xml, &field.location),
            std::slice::from_ref(&field.raw),
            "{} at {}",
            field.id,
            field.location
        );
    }
    assert_eq!(field(second_breakdown, "BT-119").raw, "0");
}

// Requirement: German labels
#[test]
fn fields_carry_german_labels() {
    let model = model(&sample_a());

    for section in &model.sections {
        for field in &section.fields {
            assert!(!field.label.is_empty(), "{} has no label", field.id);
        }
    }
    assert_eq!(
        field(section(&model, "Rechnung"), "BT-1").label,
        "Rechnungsnummer"
    );
    assert_eq!(
        field(section(&model, "Summen (BG-22)"), "BT-112").label,
        "Rechnungsbetrag inkl. USt"
    );
    assert_eq!(
        field(section(&model, "Zahlung (BG-16)"), "BT-84").label,
        "IBAN"
    );
}
