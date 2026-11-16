//! Spec 0008 — showing the raw invoice XML (core part).

mod common;

use std::time::{Duration, Instant};

use common::{fixture, sample_a_latin1, sample_cii_with_lines, sample_l_cii, select, single_line};
use e_invoice_viewer_core::detect::{DetectError, detect};
use e_invoice_viewer_core::document::{Document, open};
use e_invoice_viewer_core::xml_view::{decode, pretty_print};

/// The text without whitespace between tags, for comparing content.
fn content(xml: &str) -> String {
    single_line(xml)
}

// AC2
#[test]
fn pretty_prints_single_line_xml_one_element_per_line() {
    let compact = single_line(&sample_l_cii());
    assert_eq!(compact.lines().count(), 1, "input must be a single line");

    let pretty = pretty_print(&compact);
    let lines: Vec<&str> = pretty.lines().collect();

    assert!(lines.len() > 500 * 8, "got {} lines", lines.len());
    assert_eq!(lines[0], "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    assert!(lines.contains(&"    <ram:ID>RE-2026-0500</ram:ID>"));
    assert!(lines.contains(&"        <ram:LineID>500</ram:LineID>"));
    assert!(lines.contains(&"  </rsm:SupplyChainTradeTransaction>"));
    assert_eq!(lines.last(), Some(&"</rsm:CrossIndustryInvoice>"));
}

// Requirement: reformatting for display does not change the content
#[test]
fn pretty_print_keeps_content() {
    for xml in [fixture("cii/sample-a.xml"), fixture("cii/sample-b.xml")] {
        assert_eq!(content(&pretty_print(&xml)), content(&xml));
    }
}

// Requirement: comments, CDATA and unusual attribute values survive
#[test]
fn pretty_print_keeps_comments_cdata_and_attribute_values() {
    let xml = "<a><!-- Kommentar <b> --><b note=\"x > y\" other='1'/><c><![CDATA[<keine Elemente>]]></c><d>Text &amp; mehr</d></a>";

    let pretty = pretty_print(xml);

    assert_eq!(
        pretty,
        "<a>\n  <!-- Kommentar <b> -->\n  <b note=\"x > y\" other='1'/>\n  <c><![CDATA[<keine Elemente>]]></c>\n  <d>Text &amp; mehr</d>\n</a>"
    );
}

// AC3
#[test]
fn opens_iso_8859_1_invoice() {
    let latin1 = sample_a_latin1();
    assert!(
        std::str::from_utf8(&latin1).is_err(),
        "fixture must not be UTF-8"
    );

    let Document::Xml { invoice } = open(&latin1).expect("document") else {
        panic!("expected a plain XML invoice");
    };
    let values: Vec<&str> = invoice
        .fields
        .sections
        .iter()
        .flat_map(|section| section.fields.iter().map(|field| field.display.as_str()))
        .collect();
    assert!(values.contains(&"Hauptstraße 1"));
    assert!(values.contains(&"München"));

    let text = decode(&latin1).expect("decoded text");
    assert!(pretty_print(&text).contains("<ram:CityName>München</ram:CityName>"));
    assert_eq!(
        select(
            &text,
            "/rsm:CrossIndustryInvoice/rsm:ExchangedDocument/ram:ID"
        ),
        ["RE-2026-0042"]
    );
}

// AC7 (core part)
#[test]
fn pretty_prints_2_mb_within_one_second() {
    let xml = single_line(&sample_cii_with_lines(3000));
    assert!(xml.len() > 2_000_000, "only {} bytes", xml.len());

    let start = Instant::now();
    let pretty = pretty_print(&xml);
    let elapsed = start.elapsed();

    assert!(pretty.lines().count() > 3000 * 8);
    assert!(elapsed < Duration::from_secs(1), "took {elapsed:?}");
}

// Regression of spec 0003: decoding does not accept invalid UTF-8
#[test]
fn utf8_declared_but_invalid_still_not_well_formed() {
    let mut xml = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<a>".to_vec();
    xml.extend_from_slice(&[0xFC]); // Latin-1 'ü', invalid as UTF-8
    xml.extend_from_slice(b"</a>");

    assert_eq!(
        detect(&xml).unwrap_err(),
        DetectError::NotWellFormed { line: 2 }
    );
}
