//! Spec 0014 — link fields and XML elements in both directions.

mod common;

use std::time::{Duration, Instant};

use common::validation::{validate, variant};
use common::{fixture, replace_once, sample_l_cii, single_line};
use e_invoice_viewer_core::attribution::Target;
use e_invoice_viewer_core::fields::{Field, FieldModel, map_invoice};
use e_invoice_viewer_core::link::{XmlLayout, assignment};

struct Linked {
    model: FieldModel,
    layout: XmlLayout,
}

fn linked(xml: &str) -> Linked {
    Linked {
        model: map_invoice(xml.as_bytes()).expect("field model"),
        layout: XmlLayout::new(xml.as_bytes()).expect("layout"),
    }
}

impl Linked {
    fn field(&self, section_title: &str, id: &str) -> &Field {
        self.nth_field(section_title, 0, id)
    }

    fn nth_field(&self, section_title: &str, nth: usize, id: &str) -> &Field {
        self.model
            .sections
            .iter()
            .filter(|section| section.title == section_title)
            .nth(nth)
            .and_then(|section| section.fields.iter().find(|field| field.id == id))
            .unwrap_or_else(|| panic!("no {id} in {section_title} #{nth}"))
    }

    /// The text of the element a field is shown at.
    fn element_text(&self, field: &Field) -> &str {
        let range = self
            .layout
            .element_range(&field.location)
            .expect("element range");
        &self.layout.text()[range]
    }

    /// The assignment line for the cursor at the first occurrence of
    /// `needle` (plus `shift` bytes) in the reformatted text.
    fn assigned_at(&self, needle: &str, shift: usize) -> String {
        let offset = self.layout.text().find(needle).expect("needle") + shift;
        match assignment(&self.model, &self.layout, offset) {
            Target::Field { section, field } => {
                let section = &self.model.sections[section];
                let field = &section.fields[field];
                if section.repeated {
                    format!("{} {} · {}", field.id, field.label, section.title)
                } else {
                    format!("{} {}", field.id, field.label)
                }
            }
            Target::Section(section) => self.model.sections[section].title.clone(),
            Target::Document => "Kein Feld zugeordnet".to_string(),
        }
    }
}

// AC1
#[test]
fn bt84_range_selects_iban_element() {
    let a = linked(&fixture("cii/sample-a.xml"));

    assert_eq!(
        a.element_text(a.field("Zahlung (BG-16)", "BT-84")),
        "<ram:IBANID>DE02120300000000202051</ram:IBANID>"
    );
}

// AC2
#[test]
fn second_vat_breakdown_rate_range() {
    let b = linked(&fixture("cii/sample-b.xml"));
    let first = b.nth_field("Umsatzsteuer (BG-23)", 0, "BT-119");
    let second = b.nth_field("Umsatzsteuer (BG-23)", 1, "BT-119");

    let first_range = b.layout.element_range(&first.location).expect("first");
    let second_range = b.layout.element_range(&second.location).expect("second");
    assert!(second_range.start > first_range.end);
    assert_eq!(
        b.element_text(second),
        format!(
            "<ram:RateApplicablePercent>{}</ram:RateApplicablePercent>",
            second.raw
        )
    );
}

// AC3
#[test]
fn line_2_net_amount_and_unit_attribute_range() {
    let a = linked(&fixture("cii/sample-a.xml"));

    assert_eq!(
        a.element_text(a.field("Position 2", "BT-131")),
        "<ram:LineTotalAmount>50.00</ram:LineTotalAmount>"
    );
    let unit = a.field("Position 1", "BT-130");
    let offset = a
        .layout
        .cursor_offset(&unit.location, unit.attribute.as_deref())
        .expect("cursor offset");
    assert!(
        a.layout.text()[offset..].starts_with("HUR\""),
        "{}",
        &a.layout.text()[offset..offset + 10]
    );
    let quantity = a.field("Position 1", "BT-129");
    let offset = a
        .layout
        .cursor_offset(&quantity.location, None)
        .expect("cursor offset");
    assert!(a.layout.text()[offset..].starts_with("8</"));
}

// AC4, AC5
#[test]
fn cursor_assignments() {
    let a = linked(&fixture("cii/sample-a.xml"));

    assert_eq!(a.assigned_at("RE-2026-0042", 3), "BT-1 Rechnungsnummer");
    assert_eq!(
        a.assigned_at(">8</ram:BilledQuantity>", 1),
        "BT-129 Menge · Position 1"
    );
    assert_eq!(
        a.assigned_at("unitCode=\"HUR\"", 11),
        "BT-130 Einheit · Position 1"
    );
    assert_eq!(
        a.assigned_at(">50.00</ram:LineTotalAmount>", 2),
        "BT-131 Nettobetrag · Position 2"
    );
    assert_eq!(
        a.assigned_at("<ram:SellerTradeParty>", 2),
        "Verkäufer (BG-4)"
    );
    assert_eq!(
        a.assigned_at("<rsm:CrossIndustryInvoice", 3),
        "Kein Feld zugeordnet"
    );
}

// AC6
#[test]
fn finding_location_range() {
    let xml = replace_once(
        &variant("b-version-2-3.xml"),
        "      <ram:BuyerReference>04011000-12345-34</ram:BuyerReference>\n",
        "",
    );
    let layout = XmlLayout::new(xml.as_bytes()).expect("layout");
    let finding = validate(&xml)
        .findings
        .into_iter()
        .find(|finding| finding.rule_id == "BR-DE-15")
        .expect("BR-DE-15");

    let range = layout
        .element_range(&finding.location)
        .expect("finding range");
    let local_name = finding.location.rsplit(':').next().expect("name");
    let local_name = local_name.split('[').next().expect("name");
    assert!(
        layout.text()[range.clone()].starts_with(&format!("<rsm:{local_name}")),
        "{}",
        finding.location
    );
    assert!(layout.text()[range].ends_with('>'));
}

// AC7
#[test]
fn single_line_sample_l_position_500() {
    let l = linked(&single_line(&sample_l_cii()));
    let net = l.field("Position 500", "BT-131");

    assert_eq!(
        l.element_text(net),
        "<ram:LineTotalAmount>1.00</ram:LineTotalAmount>"
    );
    let offset = l
        .layout
        .cursor_offset(&net.location, None)
        .expect("cursor offset");
    let (line, index) = l.layout.position(offset);
    assert_eq!(l.layout.offset(line, index), offset);
    assert!(line > 500 * 10, "reformatted: line {line}");
    match assignment(&l.model, &l.layout, offset) {
        Target::Field { section, field } => {
            assert_eq!(l.model.sections[section].title, "Position 500");
            assert_eq!(l.model.sections[section].fields[field].id, "BT-131");
        }
        _ => panic!("not a field"),
    }
}

// AC8
#[test]
fn lookup_in_sample_l_is_fast() {
    let l = linked(&sample_l_cii());
    let text_length = l.layout.text().len();

    let mut slowest = Duration::ZERO;
    for step in 0..200 {
        let offset = text_length / 200 * step;
        let started = Instant::now();
        let _ = assignment(&l.model, &l.layout, offset);
        slowest = slowest.max(started.elapsed());
    }

    assert!(
        slowest < Duration::from_millis(100),
        "slowest lookup {slowest:?}"
    );
}
