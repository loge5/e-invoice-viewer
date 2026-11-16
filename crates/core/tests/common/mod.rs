//! Builders for synthetic test PDFs (no real invoices in the repository).

#![allow(dead_code)]

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
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
    });
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

/// Namespace prefixes used in field locations.
const LOCATION_PREFIXES: [(&str, &str); 4] = [
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
        let namespace = LOCATION_PREFIXES
            .iter()
            .find(|(known, _)| *known == prefix)
            .map(|(_, namespace)| *namespace)
            .unwrap_or_else(|| panic!("unknown prefix {prefix}"));
        current = current
            .into_iter()
            .flat_map(|node| {
                let matching: Vec<_> = node
                    .children()
                    .filter(|child| {
                        child.is_element()
                            && child.tag_name().namespace() == Some(namespace)
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
    let a = fixture("cii/sample-a.xml");
    let first_line = a
        .find("    <ram:IncludedSupplyChainTradeLineItem>")
        .expect("first line");
    let agreement = a
        .find("    <ram:ApplicableHeaderTradeAgreement>")
        .expect("agreement");

    let mut lines = String::new();
    for n in 1..=500 {
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
