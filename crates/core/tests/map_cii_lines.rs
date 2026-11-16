//! Spec 0005 — Map CII invoice line items to the field model.

mod common;

use std::time::{Duration, Instant};

use common::{fixture, replace_once, sample_l_cii, select};
use e_invoice_viewer_core::fields::{Field, FieldModel, Section, map_invoice};

fn model(xml: &str) -> FieldModel {
    map_invoice(xml.as_bytes()).expect("field model")
}

fn section<'a>(model: &'a FieldModel, title: &str) -> &'a Section {
    model
        .sections
        .iter()
        .find(|section| section.title == title)
        .unwrap_or_else(|| panic!("no section {title:?}"))
}

fn field<'a>(section: &'a Section, id: &str) -> &'a Field {
    section
        .fields
        .iter()
        .find(|field| field.id == id)
        .unwrap_or_else(|| panic!("no field {id} in {:?}", section.title))
}

fn titles(model: &FieldModel) -> Vec<&str> {
    model
        .sections
        .iter()
        .map(|section| section.title.as_str())
        .collect()
}

/// Sample A with line 1's quantity element replaced.
fn quantity_display(quantity: &str, unit: &str) -> String {
    let xml = replace_once(
        &fixture("cii/sample-a.xml"),
        "<ram:BilledQuantity unitCode=\"HUR\">8</ram:BilledQuantity>",
        &format!("<ram:BilledQuantity unitCode=\"{unit}\">{quantity}</ram:BilledQuantity>"),
    );
    field(section(&model(&xml), "Position 1"), "BT-129")
        .display
        .clone()
}

// AC1
#[test]
fn sample_a_line_sections_between_buyer_and_payment() {
    let model = model(&fixture("cii/sample-a.xml"));
    let titles = titles(&model);
    let buyer = titles.iter().position(|t| *t == "Käufer (BG-7)").unwrap();

    assert_eq!(
        titles[buyer..buyer + 4],
        [
            "Käufer (BG-7)",
            "Position 1",
            "Position 2",
            "Zahlung (BG-16)"
        ]
    );
}

// AC2
#[test]
fn sample_a_line_values() {
    let model = model(&fixture("cii/sample-a.xml"));
    let line_1 = section(&model, "Position 1");
    let line_2 = section(&model, "Position 2");

    assert_eq!(field(line_1, "BT-153").display, "Beratung");
    assert_eq!(field(line_1, "BT-129").display, "8 Stunden (HUR)");
    assert_eq!(field(line_1, "BT-146").display, "125,00 EUR");
    assert_eq!(field(line_1, "BT-131").display, "1.000,00 EUR");
    assert_eq!(field(line_1, "BT-151").display, "Normalsatz (S)");
    assert_eq!(field(line_1, "BT-152").display, "19");
    assert_eq!(field(line_2, "BT-129").display, "2 Stück (C62)");
}

// AC3
#[test]
fn sample_b_line_notes_and_exempt_category() {
    let model = model(&fixture("cii/sample-b.xml"));
    let notes: Vec<_> = section(&model, "Position 1")
        .fields
        .iter()
        .filter(|field| field.id == "BT-127")
        .map(|field| field.display.as_str())
        .collect();

    assert_eq!(notes, ["Lieferung KW 46", "inkl. Anfahrt"]);
    assert_eq!(
        field(section(&model, "Position 2"), "BT-151").display,
        "Steuerbefreit (E)"
    );
}

// AC4
#[test]
fn quantity_and_price_display_rules() {
    assert_eq!(quantity_display("2.500", "KGM"), "2,5 Kilogramm (KGM)");
    assert_eq!(quantity_display("2.5", "XPP"), "2,5 XPP");
    assert_eq!(quantity_display("-2", "C62"), "-2 Stück (C62)");

    let xml = replace_once(
        &fixture("cii/sample-a.xml"),
        "<ram:ChargeAmount>125.00</ram:ChargeAmount>",
        "<ram:ChargeAmount>12.3456</ram:ChargeAmount>",
    );
    assert_eq!(
        field(section(&model(&xml), "Position 1"), "BT-146").display,
        "12,3456 EUR"
    );
}

// AC5
#[test]
fn line_ids_keep_document_order() {
    let with_ids = |first: &str, second: &str| {
        let xml = replace_once(
            &replace_once(
                &fixture("cii/sample-a.xml"),
                "<ram:LineID>1</ram:LineID>",
                &format!("<ram:LineID>{first}</ram:LineID>"),
            ),
            "<ram:LineID>2</ram:LineID>",
            &format!("<ram:LineID>{second}</ram:LineID>"),
        );
        let model = model(&xml);
        titles(&model)
            .into_iter()
            .filter(|title| title.starts_with("Position "))
            .map(str::to_string)
            .collect::<Vec<_>>()
    };

    assert_eq!(with_ids("A1", "A2"), ["Position A1", "Position A2"]);
    assert_eq!(with_ids("10", "2"), ["Position 10", "Position 2"]);
}

// AC6
#[test]
fn sample_l_maps_500_lines_within_one_second() {
    let xml = sample_l_cii();

    let start = Instant::now();
    let model = model(&xml);
    let elapsed = start.elapsed();

    let lines = model
        .sections
        .iter()
        .filter(|section| section.title.starts_with("Position "))
        .count();
    assert_eq!(lines, 500);
    assert_eq!(
        field(section(&model, "Position 500"), "BT-153").display,
        "Artikel 500"
    );
    assert!(elapsed < Duration::from_secs(1), "took {elapsed:?}");
}

// AC7
#[test]
fn line_location_selects_second_line_net_amount() {
    let xml = fixture("cii/sample-a.xml");
    let model = model(&xml);
    let amount = field(section(&model, "Position 2"), "BT-131");

    assert_eq!(select(&xml, &amount.location), ["50.00"]);
}

// Contract: field order within a line section
#[test]
fn line_fields_in_contract_order() {
    let model = model(&fixture("cii/sample-b.xml"));
    let ids: Vec<_> = section(&model, "Position 1")
        .fields
        .iter()
        .map(|field| field.id.as_str())
        .collect();

    assert_eq!(
        ids,
        [
            "BT-126", "BT-127", "BT-127", "BT-153", "BT-129", "BT-130", "BT-146", "BT-131",
            "BT-151", "BT-152",
        ]
    );
}

// Contract: units with a German name
#[test]
fn known_units_have_german_names() {
    let units = [
        ("C62", "Stück"),
        ("H87", "Stück"),
        ("HUR", "Stunden"),
        ("DAY", "Tage"),
        ("MON", "Monate"),
        ("KGM", "Kilogramm"),
        ("MTR", "Meter"),
        ("LTR", "Liter"),
        ("LS", "pauschal"),
    ];

    for (code, name) in units {
        assert_eq!(quantity_display("3", code), format!("3 {name} ({code})"));
    }
}
