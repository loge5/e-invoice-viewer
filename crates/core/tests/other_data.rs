//! Spec 0031 — other data: elements without a business term are listed, so
//! that nothing in an invoice is invisible.

mod common;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use common::{fixture, replace_once};
use e_invoice_viewer_core::attribution::{Target, attribute};
use e_invoice_viewer_core::fields::{Field, FieldModel, Section, map_invoice};
use e_invoice_viewer_core::i18n::Language;
use e_invoice_viewer_core::link::XmlLayout;
use e_invoice_viewer_core::validate::{Finding, Severity};

fn model(xml: &str) -> FieldModel {
    map_invoice(xml.as_bytes(), Language::English).expect("field model")
}

/// The other-data rows (no business term number) with their section title.
fn other_data(model: &FieldModel) -> Vec<(&str, &Field)> {
    model
        .sections
        .iter()
        .flat_map(|section| {
            section
                .fields
                .iter()
                .filter(|field| field.id.is_empty())
                .map(move |field| (section.title.as_str(), field))
        })
        .collect()
}

fn section<'a>(model: &'a FieldModel, title: &str) -> &'a Section {
    model
        .sections
        .iter()
        .find(|section| section.title == title)
        .unwrap_or_else(|| panic!("no section {title}"))
}

// ---- T1: nothing is invisible -------------------------------------------

fn prefix(namespace: &str) -> &'static str {
    match namespace {
        "urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100" => "rsm",
        "urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100" => {
            "ram"
        }
        "urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100" => "udt",
        "urn:un:unece:uncefact:data:standard:QualifiedDataType:100" => "qdt",
        "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2" => "cbc",
        "urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2" => "cac",
        "urn:oasis:names:specification:ubl:schema:xsd:CommonExtensionComponents-2" => "ext",
        "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2" => "ubl",
        "urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2" => "cn",
        _ => "unknown",
    }
}

/// The location of an element as the field model writes it, computed
/// independently: prefixed names, a position only among same-named
/// siblings.
fn location(node: roxmltree::Node) -> String {
    let mut steps: Vec<String> = node
        .ancestors()
        .filter(roxmltree::Node::is_element)
        .map(|element| {
            let name = element.tag_name();
            let step = format!("{}:{}", prefix(name.namespace().unwrap_or("")), name.name());
            let same: Vec<_> = element
                .parent()
                .map(|parent| {
                    parent
                        .children()
                        .filter(|sibling| sibling.is_element() && sibling.tag_name() == name)
                        .collect()
                })
                .unwrap_or_default();
            if same.len() > 1 {
                let position = same.iter().position(|s| *s == element).expect("self") + 1;
                format!("{step}[{position}]")
            } else {
                step
            }
        })
        .collect();
    steps.reverse();
    format!("/{}", steps.join("/"))
}

fn without_positions(location: &str) -> String {
    location
        .split('/')
        .map(|step| step.split('[').next().unwrap_or(step))
        .collect::<Vec<_>>()
        .join("/")
}

/// Whether a hidden element is one the mapping uses, by the categories of
/// the requirement: fixed values, charge indicators, type codes and parts of
/// referenced documents, names of item attributes, the scheme of BT-128, or
/// a further occurrence of a shown value.
fn is_used(node: roxmltree::Node, text: &str, shown: &[(String, String)]) -> bool {
    let name = node.tag_name().name();
    let parent = node
        .parent_element()
        .map(|p| p.tag_name().name())
        .unwrap_or("");
    let in_document_reference = node.ancestors().any(|ancestor| {
        matches!(
            ancestor.tag_name().name(),
            "AdditionalReferencedDocument" | "AdditionalDocumentReference" | "DocumentReference"
        )
    });
    let further = shown.iter().any(|(location, raw)| {
        raw.trim() == text && without_positions(location) == without_positions(&location_of(node))
    });
    (text == "VAT" && matches!(name, "TypeCode" | "ID"))
        || name == "ChargeIndicator"
        || parent == "ChargeIndicator"
        || in_document_reference
        || (name == "Description" && parent == "ApplicableProductCharacteristic")
        || (name == "Name" && parent == "AdditionalItemProperty")
        || name == "ReferenceTypeCode"
        // Decides between the VAT identifier and the tax number (BT-31/32).
        || parent == "TaxScheme"
        || further
}

fn location_of(node: roxmltree::Node) -> String {
    location(node)
}

/// The elements with text that are neither shown nor used.
fn invisible(xml: &str) -> Vec<String> {
    let Ok(model) = map_invoice(xml.as_bytes(), Language::English) else {
        return Vec::new();
    };
    let shown: Vec<(String, String)> = model
        .fields()
        .map(|(_, _, field)| (field.location.clone(), field.raw.clone()))
        .collect();
    let locations: HashSet<&str> = shown
        .iter()
        .map(|(location, _)| location.as_str())
        .collect();
    let doc = roxmltree::Document::parse(xml).expect("parsed by the mapping before");
    doc.descendants()
        .filter(roxmltree::Node::is_element)
        .filter(|node| !node.children().any(|child| child.is_element()))
        .filter_map(|node| {
            let text = node
                .text()?
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let location = location(node);
            (!text.is_empty()
                && !locations.contains(location.as_str())
                && !is_used(node, &text, &shown))
            .then(|| format!("{location} = {text}"))
        })
        .collect()
}

fn xml_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("read dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "xml") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

fn assert_nothing_invisible(files: &[PathBuf]) {
    let mut problems = Vec::new();
    for path in files {
        let Ok(xml) = std::fs::read_to_string(path) else {
            continue;
        };
        for element in invisible(&xml) {
            problems.push(format!("{}: {element}", path.display()));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

// AC1
#[test]
fn nothing_is_invisible() {
    let files = xml_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"));
    assert!(files.len() > 300, "{} fixtures", files.len());
    assert_nothing_invisible(&files);
}

// AC1, locally with the FeRD package (spec 0027)
#[test]
fn nothing_is_invisible_in_ferd_samples() {
    let Some(package) = std::env::var_os("FERD_PACKAGE") else {
        return;
    };
    let files = xml_files(&PathBuf::from(package).join("Examples"));
    assert!(!files.is_empty());
    assert_nothing_invisible(&files);
}

// ---- T2–T7 ---------------------------------------------------------------

/// Sample A with elements of the Extended profile in the document, the
/// seller and the first line.
fn sample_a_with_extended_elements() -> String {
    let xml = replace_once(
        &fixture("cii/sample-a.xml"),
        "    <ram:ID>RE-2026-0042</ram:ID>\n",
        "    <ram:ID>RE-2026-0042</ram:ID>\n    <ram:Name>RECHNUNG</ram:Name>\n",
    );
    let xml = replace_once(
        &xml,
        "          <ram:PersonName>Erika Mustermann</ram:PersonName>\n",
        "          <ram:PersonName>Erika Mustermann</ram:PersonName>\n          <ram:TypeCode>LB</ram:TypeCode>\n",
    );
    replace_once(
        &xml,
        "        <ram:Name>Beratung</ram:Name>\n",
        "        <ram:Name>Beratung</ram:Name>\n        <ram:IncludedReferencedProduct>\n          <ram:GlobalID schemeID=\"0160\">4012345000016</ram:GlobalID>\n        </ram:IncludedReferencedProduct>\n",
    )
}

// AC2
#[test]
fn extended_elements_listed_at_their_place() {
    let model = model(&sample_a_with_extended_elements());
    let rows: Vec<(&str, &str, &str)> = other_data(&model)
        .into_iter()
        .map(|(section, field)| (section, field.label.as_str(), field.display.as_str()))
        .collect();
    assert_eq!(
        rows,
        [
            ("Seller (BG-4)", "DefinedTradeContact › TypeCode", "LB"),
            (
                "Invoice line 1",
                "SpecifiedTradeProduct › IncludedReferencedProduct › GlobalID",
                "4012345000016 (schemeID: 0160)"
            ),
            ("Other data", "ExchangedDocument › Name", "RECHNUNG"),
        ]
    );
    // At the end of their section, after the business terms.
    let seller = section(&model, "Seller (BG-4)");
    assert!(
        seller
            .fields
            .last()
            .is_some_and(|field| field.id.is_empty())
    );
    assert_eq!(
        model.sections.last().map(|section| section.title.as_str()),
        Some("Other data")
    );
}

// AC3
#[test]
fn samples_have_no_other_data() {
    for path in [
        "cii/sample-a.xml",
        "cii/sample-b.xml",
        "ubl/sample-a.xml",
        "ubl/sample-b.xml",
        "ubl/sample-c.xml",
    ] {
        assert_eq!(other_data(&model(&fixture(path))).len(), 0, "{path}");
    }
    // Not business terms: the project name CII requires next to BT-11, the
    // card network UBL requires in a card account.
    for (path, label) in [
        (
            "cii/sample-c.xml",
            "SupplyChainTradeTransaction › ApplicableHeaderTradeAgreement › SpecifiedProcuringProject › Name",
        ),
        (
            "cii/sample-d.xml",
            "SupplyChainTradeTransaction › ApplicableHeaderTradeAgreement › SpecifiedProcuringProject › Name",
        ),
        ("ubl/sample-d.xml", "PaymentMeans › CardAccount › NetworkID"),
    ] {
        let model = model(&fixture(path));
        let labels: Vec<&str> = other_data(&model)
            .iter()
            .map(|(_, field)| field.label.as_str())
            .collect();
        assert_eq!(labels, [label], "{path}");
    }
}

/// Elements of EN 16931 invoices that are not business terms (notes of
/// spec 0031), by their path without positions, from the invoice's root.
const NOT_BUSINESS_TERMS: [&str; 9] = [
    "ram:SpecifiedProcuringProject/ram:Name",
    "ram:GrossPriceProductTradePrice/ram:BasisQuantity",
    "ram:PayerSpecifiedDebtorFinancialInstitution/ram:BICID",
    "cac:CardAccount/cbc:NetworkID",
    "cac:CreditAccount/cbc:AccountID",
    "cac:OrderLineReference/cac:OrderReference/cbc:ID",
    // A second tax total in the invoice currency (negative test invoices).
    "cac:TaxTotal/cbc:TaxAmount",
    "ram:SpecifiedTradeSettlementHeaderMonetarySummation/ram:TaxTotalAmount",
    // The XRechnung extension (spec 0010).
    "XRechnung extension",
];

// AC4
#[test]
fn kosit_other_data_is_not_business_terms() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/kosit");
    let mut unexpected = Vec::new();
    for path in xml_files(&dir) {
        let xml = std::fs::read_to_string(&path).expect("read");
        let extension = xml.contains("urn:xeinkauf.de:kosit:extension:xrechnung");
        let Ok(model) = map_invoice(xml.as_bytes(), Language::English) else {
            continue;
        };
        for (_, field) in other_data(&model) {
            let known = NOT_BUSINESS_TERMS
                .iter()
                .any(|path| without_positions(&field.location).ends_with(path));
            if !known && !extension {
                unexpected.push(format!("{}: {}", path.display(), field.location));
            }
        }
    }
    assert!(unexpected.is_empty(), "{}", unexpected.join("\n"));
}

// AC5
#[test]
fn party_identifiers_from_global_id() {
    let xml = replace_once(
        &fixture("cii/sample-a.xml"),
        "        <ram:Name>Muster GmbH</ram:Name>\n",
        "        <ram:GlobalID schemeID=\"0088\">4000001000005</ram:GlobalID>\n        <ram:Name>Muster GmbH</ram:Name>\n",
    );
    let model = model(&xml);
    let seller = section(&model, "Seller (BG-4)");
    let bt29: Vec<&str> = seller
        .fields
        .iter()
        .filter(|field| field.id == "BT-29")
        .map(|field| field.display.as_str())
        .collect();
    assert_eq!(bt29, ["4000001000005 (0088)"]);
    assert_eq!(other_data(&model).len(), 0);
}

// AC5: BT-46 and BT-71 from `ram:GlobalID` too
#[test]
fn buyer_and_delivery_identifiers_from_global_id() {
    let xml = fixture("cii/sample-d.xml");
    for (id, element) in [("BT-46", "BuyerTradeParty"), ("BT-71", "ShipToTradeParty")] {
        let open = format!("<ram:{element}>\n");
        let Some(start) = xml.find(&open) else {
            panic!("{element} in sample D")
        };
        let indent = "          ";
        let with_global = format!(
            "{}{open}{indent}<ram:GlobalID schemeID=\"0088\">4000001000012</ram:GlobalID>\n{}",
            &xml[..start],
            &xml[start + open.len()..]
        );
        let model = model(&with_global);
        let shown = model
            .fields()
            .any(|(_, _, field)| field.id == id && field.display == "4000001000012 (0088)");
        assert!(shown, "{id}: {:#?}", other_data(&model));
        assert!(
            other_data(&model)
                .iter()
                .all(|(_, field)| !field.display.starts_with("4000001000012"))
        );
    }
}

// AC6
#[test]
fn further_occurrences_only_if_different() {
    let d = fixture("cii/sample-d.xml");
    let with_date = |date: &str| {
        replace_once(
            &d,
            "        <ram:CalculatedAmount>0.00</ram:CalculatedAmount>\n",
            &format!(
                "        <ram:CalculatedAmount>0.00</ram:CalculatedAmount>\n        <ram:TaxPointDate>\n          <udt:DateString format=\"102\">{date}</udt:DateString>\n        </ram:TaxPointDate>\n"
            ),
        )
    };
    let same = model(&with_date("20261103"));
    assert!(
        other_data(&same)
            .iter()
            .all(|(_, field)| !field.label.contains("TaxPointDate")),
        "{:#?}",
        other_data(&same)
    );
    let different = model(&with_date("20261104"));
    let listed: Vec<&str> = other_data(&different)
        .iter()
        .filter(|(_, field)| field.label.contains("TaxPointDate"))
        .map(|(_, field)| field.display.as_str())
        .collect();
    assert_eq!(listed, ["20261104 (format: 102)"]);
}

// AC7
#[test]
fn finding_at_other_data_row() {
    let xml = sample_a_with_extended_elements();
    let model = model(&xml);
    let (_, row) = other_data(&model)
        .into_iter()
        .find(|(_, field)| field.label == "ExchangedDocument › Name")
        .expect("row");
    let finding = Finding {
        rule_id: "XSD".to_string(),
        severity: Severity::Error,
        message: "Element 'ram:Name': This element is not expected.".to_string(),
        location: row.location.clone(),
        business_terms: Vec::new(),
    };
    let Target::Field { section, field } = attribute(&model, &finding) else {
        panic!("not at the row")
    };
    assert_eq!(&model.sections[section].fields[field], row);
    let layout = XmlLayout::new(xml.as_bytes()).expect("layout");
    assert!(layout.element_range(&row.location).is_some());
}
