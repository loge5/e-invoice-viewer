//! Builders for synthetic test PDFs (no real invoices in the repository).

#![allow(dead_code)]

pub mod validation;

use lopdf::encryption::{EncryptionState, EncryptionVersion, Permissions};
use lopdf::{Document, Object, Stream, dictionary};

/// A file to embed into a test PDF.
pub struct Attachment<'a> {
    pub name: &'a str,
    pub content: &'a [u8],
    /// Store the stream Flate-encoded instead of uncompressed.
    pub compressed: bool,
}

impl<'a> Attachment<'a> {
    pub fn plain(name: &'a str, content: &'a [u8]) -> Self {
        Self {
            name,
            content,
            compressed: false,
        }
    }

    pub fn compressed(name: &'a str, content: &'a [u8]) -> Self {
        Self {
            name,
            content,
            compressed: true,
        }
    }
}

/// Sample invoice XML (start of CII sample A, enough for byte comparisons).
pub const SAMPLE_XML: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><rsm:CrossIndustryInvoice xmlns:rsm=\"urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100\"><!-- RE-2026-0042 --></rsm:CrossIndustryInvoice>";

/// A one-page PDF document with the given embedded files.
pub fn pdf_document(attachments: &[Attachment]) -> Document {
    pdf_document_with_text(None, attachments)
}

/// A one-page A4 PDF document, optionally showing `text` at the top of the
/// page, with the given embedded files.
pub fn pdf_document_with_text(text: Option<&str>, attachments: &[Attachment]) -> Document {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let mut page = dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
    };
    if let Some(text) = text {
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });
        let content = format!("BT /F1 24 Tf 60 760 Td ({text}) Tj ET");
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.into_bytes()));
        page.set("Contents", content_id);
        page.set(
            "Resources",
            dictionary! { "Font" => dictionary! { "F1" => font_id } },
        );
    }
    let page_id = doc.add_object(page);
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        }),
    );

    let mut name_tree = Vec::new();
    for attachment in attachments {
        let mut stream = Stream::new(
            dictionary! { "Type" => "EmbeddedFile" },
            attachment.content.to_vec(),
        );
        if attachment.compressed {
            stream.compress().expect("compress test stream");
            assert!(
                stream.dict.get(b"Filter").is_ok(),
                "test stream must be Flate-encoded"
            );
        }
        let stream_id = doc.add_object(stream);
        let filespec_id = doc.add_object(dictionary! {
            "Type" => "Filespec",
            "F" => Object::string_literal(attachment.name),
            "UF" => Object::string_literal(attachment.name),
            "EF" => dictionary! { "F" => stream_id },
        });
        name_tree.push(Object::string_literal(attachment.name));
        name_tree.push(filespec_id.into());
    }

    let mut catalog = dictionary! { "Type" => "Catalog", "Pages" => pages_id };
    if !attachments.is_empty() {
        catalog.set(
            "Names",
            dictionary! { "EmbeddedFiles" => dictionary! { "Names" => name_tree } },
        );
    }
    let catalog_id = doc.add_object(catalog);
    doc.trailer.set("Root", catalog_id);
    doc
}

/// Serialised one-page PDF with the given embedded files.
pub fn pdf_with(attachments: &[Attachment]) -> Vec<u8> {
    save(&mut pdf_document(attachments))
}

/// Serialised one-page PDF showing `text`, with the given embedded files.
pub fn pdf_with_text(text: &str, attachments: &[Attachment]) -> Vec<u8> {
    save(&mut pdf_document_with_text(Some(text), attachments))
}

/// Serialised PDF whose catalog has no page tree, so it has no renderable
/// pages, but which still embeds the given files.
pub fn pdf_without_page_tree(attachments: &[Attachment]) -> Vec<u8> {
    let mut doc = pdf_document(attachments);
    let catalog_id = doc
        .trailer
        .get(b"Root")
        .and_then(Object::as_reference)
        .expect("catalog");
    doc.get_dictionary_mut(catalog_id)
        .expect("catalog dictionary")
        .remove(b"Pages");
    save(&mut doc)
}

/// Serialised PDF that requires the user password `geheim` to open.
pub fn password_protected_pdf() -> Vec<u8> {
    let mut doc = pdf_document(&[Attachment::plain("factur-x.xml", SAMPLE_XML)]);
    let file_id = Object::string_literal("e-invoice-viewer-test-id");
    doc.trailer.set("ID", vec![file_id.clone(), file_id]);
    let version = EncryptionVersion::V2 {
        document: &doc,
        owner_password: "owner",
        user_password: "geheim",
        key_length: 128,
        permissions: Permissions::all(),
    };
    let state = EncryptionState::try_from(version).expect("encryption state");
    doc.encrypt(&state).expect("encrypt test PDF");
    save(&mut doc)
}

/// Serialised PDF of at least 4 KB: the invoice is preceded by a 4 KB
/// attachment, so that truncating it cuts through the file body.
pub fn large_pdf() -> Vec<u8> {
    let filler = vec![b'x'; 4096];
    pdf_with(&[
        Attachment::plain("lieferschein.pdf", &filler),
        Attachment::plain("factur-x.xml", SAMPLE_XML),
    ])
}

/// Bytes of a minimal PNG image (signature and IHDR chunk header).
pub fn png_bytes() -> Vec<u8> {
    let mut bytes = vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];
    bytes.extend_from_slice(&[0, 0, 0, 13, b'I', b'H', b'D', b'R']);
    bytes.extend_from_slice(&[0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0]);
    bytes
}

fn save(doc: &mut Document) -> Vec<u8> {
    let mut out = Vec::new();
    doc.save_to(&mut out).expect("save test PDF");
    out
}

/// Content of a sample invoice from `tests/fixtures` (see docs/fixtures.md).
pub fn fixture(relative_path: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{relative_path}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// Replaces exactly one occurrence of `from` in `xml`, failing loudly if the
/// fixture does not contain it.
pub fn replace_once(xml: &str, from: &str, to: &str) -> String {
    assert_eq!(
        xml.matches(from).count(),
        1,
        "expected exactly one {from:?}"
    );
    xml.replacen(from, to, 1)
}

/// Namespace prefixes used in locations (field model and validation
/// findings). `udt` and `qdt` differ between CII and UBL, so a prefix may
/// stand for more than one namespace.
const LOCATION_PREFIXES: [(&str, &str); 11] = [
    (
        "rsm",
        "urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100",
    ),
    (
        "ram",
        "urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100",
    ),
    (
        "udt",
        "urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100",
    ),
    (
        "qdt",
        "urn:un:unece:uncefact:data:standard:QualifiedDataType:100",
    ),
    (
        "ubl",
        "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2",
    ),
    (
        "cn",
        "urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2",
    ),
    (
        "cac",
        "urn:oasis:names:specification:ubl:schema:xsd:CommonAggregateComponents-2",
    ),
    (
        "cbc",
        "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2",
    ),
    (
        "ext",
        "urn:oasis:names:specification:ubl:schema:xsd:CommonExtensionComponents-2",
    ),
    (
        "udt",
        "urn:oasis:names:specification:ubl:schema:xsd:UnqualifiedDataTypes-2",
    ),
    (
        "qdt",
        "urn:oasis:names:specification:ubl:schema:xsd:QualifiedDataTypes-2",
    ),
];

/// Evaluates a simple absolute location path such as
/// `/rsm:A/ram:B[2]/ram:C` and returns the texts of all selected elements.
pub fn select(xml: &str, location: &str) -> Vec<String> {
    let doc = roxmltree::Document::parse(xml).expect("well-formed fixture");
    let mut current = vec![doc.root()];
    for step in location
        .strip_prefix('/')
        .expect("absolute path")
        .split('/')
    {
        let (qname, position) = match step.split_once('[') {
            Some((qname, rest)) => (
                qname,
                Some(
                    rest.trim_end_matches(']')
                        .parse::<usize>()
                        .expect("position"),
                ),
            ),
            None => (step, None),
        };
        let (prefix, name) = qname.split_once(':').expect("prefixed name");
        let namespaces: Vec<&str> = LOCATION_PREFIXES
            .iter()
            .filter(|(known, _)| *known == prefix)
            .map(|(_, namespace)| *namespace)
            .collect();
        assert!(!namespaces.is_empty(), "unknown prefix {prefix}");
        current = current
            .into_iter()
            .flat_map(|node| {
                let matching: Vec<_> = node
                    .children()
                    .filter(|child| {
                        child.is_element()
                            && child
                                .tag_name()
                                .namespace()
                                .is_some_and(|namespace| namespaces.contains(&namespace))
                            && child.tag_name().name() == name
                    })
                    .collect();
                match position {
                    Some(position) => matching.get(position - 1).copied().into_iter().collect(),
                    None => matching,
                }
            })
            .collect();
    }
    current
        .iter()
        .map(|node| node.text().unwrap_or_default().to_string())
        .collect()
}

/// CII sample L (docs/fixtures.md): sample A with 500 generated lines
/// `Artikel 1` .. `Artikel 500` (1 C62 at 1.00 EUR, VAT S 19 %) and matching
/// totals. Generated instead of stored to keep the repository small.
pub fn sample_l_cii() -> String {
    sample_cii_with_lines(500)
}

/// Sample A with `count` generated lines (see `sample_l_cii`); totals match
/// for 500 lines, larger counts are only used for size and speed tests.
pub fn sample_cii_with_lines(count: usize) -> String {
    let a = fixture("cii/sample-a.xml");
    let first_line = a
        .find("    <ram:IncludedSupplyChainTradeLineItem>")
        .expect("first line");
    let agreement = a
        .find("    <ram:ApplicableHeaderTradeAgreement>")
        .expect("agreement");

    let mut lines = String::new();
    for n in 1..=count {
        lines.push_str(&format!(
            "    <ram:IncludedSupplyChainTradeLineItem>\n\
             \x20     <ram:AssociatedDocumentLineDocument><ram:LineID>{n}</ram:LineID></ram:AssociatedDocumentLineDocument>\n\
             \x20     <ram:SpecifiedTradeProduct><ram:Name>Artikel {n}</ram:Name></ram:SpecifiedTradeProduct>\n\
             \x20     <ram:SpecifiedLineTradeAgreement><ram:NetPriceProductTradePrice><ram:ChargeAmount>1.00</ram:ChargeAmount></ram:NetPriceProductTradePrice></ram:SpecifiedLineTradeAgreement>\n\
             \x20     <ram:SpecifiedLineTradeDelivery><ram:BilledQuantity unitCode=\"C62\">1</ram:BilledQuantity></ram:SpecifiedLineTradeDelivery>\n\
             \x20     <ram:SpecifiedLineTradeSettlement><ram:ApplicableTradeTax><ram:TypeCode>VAT</ram:TypeCode><ram:CategoryCode>S</ram:CategoryCode><ram:RateApplicablePercent>19</ram:RateApplicablePercent></ram:ApplicableTradeTax><ram:SpecifiedTradeSettlementLineMonetarySummation><ram:LineTotalAmount>1.00</ram:LineTotalAmount></ram:SpecifiedTradeSettlementLineMonetarySummation></ram:SpecifiedLineTradeSettlement>\n\
             \x20   </ram:IncludedSupplyChainTradeLineItem>\n"
        ));
    }
    let xml = format!("{}{lines}{}", &a[..first_line], &a[agreement..]);

    [
        (
            "<ram:ID>RE-2026-0042</ram:ID>",
            "<ram:ID>RE-2026-0500</ram:ID>",
        ),
        (
            "<ram:CalculatedAmount>199.50</ram:CalculatedAmount>",
            "<ram:CalculatedAmount>95.00</ram:CalculatedAmount>",
        ),
        (
            "<ram:BasisAmount>1050.00</ram:BasisAmount>",
            "<ram:BasisAmount>500.00</ram:BasisAmount>",
        ),
        (
            "<ram:LineTotalAmount>1050.00</ram:LineTotalAmount>",
            "<ram:LineTotalAmount>500.00</ram:LineTotalAmount>",
        ),
        (
            "<ram:TaxBasisTotalAmount>1050.00</ram:TaxBasisTotalAmount>",
            "<ram:TaxBasisTotalAmount>500.00</ram:TaxBasisTotalAmount>",
        ),
        (
            "<ram:TaxTotalAmount currencyID=\"EUR\">199.50</ram:TaxTotalAmount>",
            "<ram:TaxTotalAmount currencyID=\"EUR\">95.00</ram:TaxTotalAmount>",
        ),
        (
            "<ram:GrandTotalAmount>1249.50</ram:GrandTotalAmount>",
            "<ram:GrandTotalAmount>595.00</ram:GrandTotalAmount>",
        ),
        (
            "<ram:DuePayableAmount>1249.50</ram:DuePayableAmount>",
            "<ram:DuePayableAmount>595.00</ram:DuePayableAmount>",
        ),
    ]
    .iter()
    .fold(xml, |xml, (from, to)| replace_once(&xml, from, to))
}

/// UBL sample L: UBL sample A with the same 500 generated lines as
/// `sample_l_cii` and matching totals (spec 0006).
pub fn sample_l_ubl() -> String {
    let a = fixture("ubl/sample-a.xml");
    let first_line = a.find("  <cac:InvoiceLine>").expect("first line");
    let end = a.find("</ubl:Invoice>").expect("end");

    let mut lines = String::new();
    for n in 1..=500 {
        lines.push_str(&format!(
            "  <cac:InvoiceLine>\n\
             \x20   <cbc:ID>{n}</cbc:ID>\n\
             \x20   <cbc:InvoicedQuantity unitCode=\"C62\">1</cbc:InvoicedQuantity>\n\
             \x20   <cbc:LineExtensionAmount currencyID=\"EUR\">1.00</cbc:LineExtensionAmount>\n\
             \x20   <cac:Item><cbc:Name>Artikel {n}</cbc:Name><cac:ClassifiedTaxCategory><cbc:ID>S</cbc:ID><cbc:Percent>19</cbc:Percent><cac:TaxScheme><cbc:ID>VAT</cbc:ID></cac:TaxScheme></cac:ClassifiedTaxCategory></cac:Item>\n\
             \x20   <cac:Price><cbc:PriceAmount currencyID=\"EUR\">1.00</cbc:PriceAmount></cac:Price>\n\
             \x20 </cac:InvoiceLine>\n"
        ));
    }
    format!("{}{lines}{}", &a[..first_line], &a[end..])
        .replace(
            "<cbc:ID>RE-2026-0042</cbc:ID>",
            "<cbc:ID>RE-2026-0500</cbc:ID>",
        )
        .replace(">199.50<", ">95.00<")
        .replace(">1050.00<", ">500.00<")
        .replace(">1249.50<", ">595.00<")
}

/// `xml` on a single line: whitespace between tags removed, whitespace runs
/// inside tags (e.g. between attributes) collapsed to one space.
pub fn single_line(xml: &str) -> String {
    let mut out = String::with_capacity(xml.len());
    let mut pending = String::new();
    let mut after_tag = false;
    let mut in_tag = false;
    for c in xml.chars() {
        if in_tag && c.is_whitespace() {
            if !out.ends_with(' ') {
                out.push(' ');
            }
            continue;
        }
        if after_tag && c.is_whitespace() {
            pending.push(c);
            continue;
        }
        if c != '<' {
            out.push_str(&pending);
        }
        pending.clear();
        out.push(c);
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => {}
        }
        after_tag = c == '>';
    }
    out
}

/// Sample A encoded as ISO-8859-1 with a matching XML declaration.
pub fn sample_a_latin1() -> Vec<u8> {
    let xml = replace_once(
        &fixture("cii/sample-a.xml"),
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
        "<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?>",
    );
    xml.chars()
        .map(|c| u8::try_from(u32::from(c)).expect("character representable in Latin-1"))
        .collect()
}
