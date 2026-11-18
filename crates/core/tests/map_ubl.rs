//! Spec 0006 — Map UBL invoices and credit notes to the field model.

mod common;

use common::validation::validate;
use common::{fixture, replace_once, sample_l_cii, sample_l_ubl, select};
use e_invoice_viewer_core::attribution::{Target, attribute};
use e_invoice_viewer_core::fields::{Field, FieldModel, Section, map_invoice};

fn model(xml: &str) -> FieldModel {
    map_invoice(xml.as_bytes()).expect("field model")
}

fn sections<'a>(model: &'a FieldModel, title: &str) -> Vec<&'a Section> {
    model
        .sections
        .iter()
        .filter(|section| section.title == title)
        .collect()
}

fn section<'a>(model: &'a FieldModel, title: &str) -> &'a Section {
    sections(model, title)
        .first()
        .copied()
        .unwrap_or_else(|| panic!("no section {title:?}"))
}

fn field<'a>(section: &'a Section, id: &str) -> &'a Field {
    section
        .fields
        .iter()
        .find(|field| field.id == id)
        .unwrap_or_else(|| panic!("no field {id} in {:?}", section.title))
}

/// (section title, field id, display) of every field, in order.
fn displays(model: &FieldModel) -> Vec<(String, String, String)> {
    model
        .sections
        .iter()
        .flat_map(|section| {
            section.fields.iter().map(|field| {
                (
                    section.title.clone(),
                    field.id.clone(),
                    field.display.clone(),
                )
            })
        })
        .collect()
}

/// UBL sample A as a credit note with BT-3 `381`.
fn credit_note_a() -> String {
    let a = fixture("ubl/sample-a.xml");
    let a = replace_once(&a, "  <cbc:DueDate>2026-12-01</cbc:DueDate>\n", "");
    let a = replace_once(
        &a,
        "    <cbc:PaymentMeansCode>58</cbc:PaymentMeansCode>\n",
        "    <cbc:PaymentMeansCode>58</cbc:PaymentMeansCode>\n    <cbc:PaymentDueDate>2026-12-01</cbc:PaymentDueDate>\n",
    );
    a.replace(
        "ubl:Invoice xmlns:ubl=\"urn:oasis:names:specification:ubl:schema:xsd:Invoice-2\"",
        "cn:CreditNote xmlns:cn=\"urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2\"",
    )
    .replace("</ubl:Invoice>", "</cn:CreditNote>")
    .replace(
        "<cbc:InvoiceTypeCode>380</cbc:InvoiceTypeCode>",
        "<cbc:CreditNoteTypeCode>381</cbc:CreditNoteTypeCode>",
    )
    .replace("cac:InvoiceLine>", "cac:CreditNoteLine>")
    .replace("cbc:InvoicedQuantity", "cbc:CreditedQuantity")
}

/// CII sample B without the second note of line 1: UBL allows only one note
/// per line, so UBL sample B has only the first (docs/fixtures.md).
fn cii_b_with_one_line_note() -> String {
    let b = fixture("cii/sample-b.xml");
    let note = b.find("inkl. Anfahrt").expect("second note");
    let start = b[..note].rfind("<ram:IncludedNote>").expect("note start");
    let end = note
        + b[note..].find("</ram:IncludedNote>").expect("note end")
        + "</ram:IncludedNote>".len();
    let line_start = b[..start].rfind('\n').expect("line start");
    format!("{}{}", &b[..line_start], &b[end..])
}

// AC1
#[test]
fn ubl_and_cii_give_same_fields() {
    for (name, ubl, cii) in [
        (
            "A",
            fixture("ubl/sample-a.xml"),
            fixture("cii/sample-a.xml"),
        ),
        ("B", fixture("ubl/sample-b.xml"), cii_b_with_one_line_note()),
        ("L", sample_l_ubl(), sample_l_cii()),
    ] {
        let (ubl, cii) = (displays(&model(&ubl)), displays(&model(&cii)));
        let differences: Vec<String> = ubl
            .iter()
            .zip(&cii)
            .filter(|(u, c)| u != c)
            .take(10)
            .map(|(u, c)| format!("UBL {u:?} vs CII {c:?}"))
            .collect();
        assert!(
            differences.is_empty(),
            "sample {name}:\n{}",
            differences.join("\n")
        );
        assert_eq!(ubl.len(), cii.len(), "sample {name}: number of fields");
    }
}

// AC2
#[test]
fn ubl_issue_date_raw_and_display() {
    let bt2 = field(
        section(&model(&fixture("ubl/sample-a.xml")), "Rechnung"),
        "BT-2",
    )
    .clone();

    assert_eq!(
        (bt2.raw.as_str(), bt2.display.as_str()),
        ("2026-11-17", "17.11.2026")
    );
}

// AC3
#[test]
fn ubl_second_vat_breakdown() {
    let xml = fixture("ubl/sample-b.xml");
    let model = model(&xml);
    let breakdowns = sections(&model, "Umsatzsteuer (BG-23)");
    assert_eq!(breakdowns.len(), 2);
    let second = breakdowns[1];

    assert_eq!(field(second, "BT-118").display, "Steuerbefreit (E)");
    assert_eq!(
        field(second, "BT-120").display,
        "Steuerbefreit nach § 4 Nr. 21 UStG"
    );
    let bt119 = &field(second, "BT-119").location;
    assert!(
        bt119.contains("cac:TaxSubtotal[2]/cac:TaxCategory/cbc:Percent"),
        "{bt119}"
    );
    assert_eq!(select(&xml, bt119), [field(second, "BT-119").raw.clone()]);
}

// AC4
#[test]
fn credit_note_maps_like_invoice() {
    let credit_note = model(&credit_note_a());
    let invoice = model(&fixture("ubl/sample-a.xml"));

    assert_eq!(
        field(section(&credit_note, "Rechnung"), "BT-3").display,
        "Gutschrift (381)"
    );
    assert_eq!(
        field(section(&credit_note, "Rechnung"), "BT-9").display,
        "01.12.2026"
    );
    for title in ["Position 1", "Position 2"] {
        let fields = |model: &FieldModel| {
            section(model, title)
                .fields
                .iter()
                .map(|field| (field.id.clone(), field.display.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(fields(&credit_note), fields(&invoice), "{title}");
    }
}

// AC5
#[test]
fn bt31_comes_from_vat_tax_scheme() {
    // A second company ID of another tax scheme must not be taken.
    let xml = replace_once(
        &fixture("ubl/sample-a.xml"),
        "      <cac:PartyTaxScheme>\n        <cbc:CompanyID>DE123456789</cbc:CompanyID>",
        "      <cac:PartyTaxScheme>\n        <cbc:CompanyID>201/113/40209</cbc:CompanyID>\n        <cac:TaxScheme>\n          <cbc:ID>FC</cbc:ID>\n        </cac:TaxScheme>\n      </cac:PartyTaxScheme>\n      <cac:PartyTaxScheme>\n        <cbc:CompanyID>DE123456789</cbc:CompanyID>",
    );
    let model = model(&xml);
    let seller = section(&model, "Verkäufer (BG-4)");
    let bt31 = field(seller, "BT-31");

    assert_eq!(bt31.display, "DE123456789");
    assert!(
        bt31.location
            .ends_with("cac:PartyTaxScheme[2]/cbc:CompanyID"),
        "{}",
        bt31.location
    );
    assert_eq!(select(&xml, &bt31.location), ["DE123456789"]);
    assert_eq!(field(seller, "BT-32").display, "201/113/40209");
}

// AC6
#[test]
fn missing_payment_means_has_no_payment_section() {
    let a = fixture("ubl/sample-a.xml");
    let start = a.find("  <cac:PaymentMeans>").expect("payment means");
    let end = a.find("  <cac:TaxTotal>").expect("tax total");
    let xml = format!("{}{}", &a[..start], &a[end..]);

    assert!(sections(&model(&xml), "Zahlung (BG-16)").is_empty());
}

// Contract: every section and field location selects exactly one element
#[test]
fn every_location_selects_one_element() {
    for xml in [
        fixture("ubl/sample-a.xml"),
        fixture("ubl/sample-b.xml"),
        credit_note_a(),
    ] {
        for section in &model(&xml).sections {
            assert_eq!(
                select(&xml, &section.location).len(),
                1,
                "{}",
                section.location
            );
            for field in &section.fields {
                assert_eq!(
                    select(&xml, &field.location).len(),
                    1,
                    "{} {} at {}",
                    section.title,
                    field.id,
                    field.location
                );
            }
        }
    }
}

// Spec 0011: UBL findings are attributed like CII findings
#[test]
fn ubl_grand_total_findings_go_to_bt112_and_bt115() {
    let xml = replace_once(
        &fixture("ubl/sample-a.xml"),
        "<cbc:TaxInclusiveAmount currencyID=\"EUR\">1249.50</cbc:TaxInclusiveAmount>",
        "<cbc:TaxInclusiveAmount currencyID=\"EUR\">1300.00</cbc:TaxInclusiveAmount>",
    );
    let model = model(&xml);
    let target_of = |rule_id: &str| {
        let finding = validate(&xml)
            .findings
            .into_iter()
            .find(|finding| finding.rule_id == rule_id)
            .unwrap_or_else(|| panic!("no {rule_id}"));
        match attribute(&model, &finding) {
            Target::Field { section, field } => format!(
                "{} {}",
                model.sections[section].title, model.sections[section].fields[field].id
            ),
            Target::Section(section) => model.sections[section].title.clone(),
            Target::Document => "Dokument".to_string(),
        }
    };

    assert_eq!(target_of("BR-CO-15"), "Summen (BG-22) BT-112");
    assert_eq!(target_of("BR-CO-16"), "Summen (BG-22) BT-115");
}

// BT-110 is the VAT total with the breakdown, not the VAT total in
// accounting currency (BT-111), whichever comes first.
#[test]
fn vat_total_ignores_accounting_currency_total() {
    let xml = replace_once(
        &fixture("ubl/sample-a.xml"),
        "  <cac:TaxTotal>\n",
        "  <cac:TaxTotal>\n    <cbc:TaxAmount currencyID=\"USD\">215.00</cbc:TaxAmount>\n  </cac:TaxTotal>\n  <cac:TaxTotal>\n",
    );
    let model = model(&xml);
    let bt110 = field(section(&model, "Summen (BG-22)"), "BT-110");

    assert_eq!(bt110.raw, "199.50");
    assert_eq!(bt110.location, "/ubl:Invoice/cac:TaxTotal[2]/cbc:TaxAmount");
}
