//! The field model: invoice content as sections of labelled business terms
//! (specs 0004 to 0006).

mod cii;
mod display;

use roxmltree::{Document, Node};
use thiserror::Error;

use crate::detect::{DetectError, Detection, Syntax, detect};

/// Invoice content as an ordered list of sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldModel {
    pub sections: Vec<Section>,
}

/// A titled group of fields, e.g. "Verkäufer (BG-4)".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub title: String,
    pub fields: Vec<Field>,
}

/// One business term value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Business term number, e.g. `BT-1`.
    pub id: String,
    /// German label.
    pub label: String,
    /// Value as in the XML.
    pub raw: String,
    /// Value formatted for humans.
    pub display: String,
    /// XPath of the source element.
    pub location: String,
}

/// Why no field model could be built.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MapError {
    #[error(transparent)]
    Detect(#[from] DetectError),
    #[error("syntax not supported yet")]
    UnsupportedSyntax,
}

/// Builds the field model of an invoice XML.
pub fn map_invoice(xml: &[u8]) -> Result<FieldModel, MapError> {
    let detection = detect(xml)?;
    if detection.syntax != Syntax::Cii {
        return Err(MapError::UnsupportedSyntax);
    }
    // `detect` has already checked that the document is well-formed UTF-8
    // without a DOCTYPE, so parsing it again cannot fail differently.
    let text = std::str::from_utf8(xml).map_err(|_| MapError::UnsupportedSyntax)?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let doc = Document::parse(text).map_err(|_| MapError::UnsupportedSyntax)?;
    let root = doc.root_element();

    let context = Context {
        detection: &detection,
        currency: find_all(root, cii::CURRENCY_PATH)
            .first()
            .and_then(|node| node.text())
            .map(|text| text.trim().to_string()),
        prefixes: cii::PREFIXES,
    };

    let sections = cii::HEADER_SECTIONS
        .iter()
        .chain(cii::TRAILER_SECTIONS)
        .flat_map(|def| map_section(root, def, &context))
        .collect();
    Ok(FieldModel { sections })
}

/// One step of a path: an element with namespace and local name, optionally
/// restricted to a given attribute value.
#[derive(Debug, Clone, Copy)]
struct Step {
    namespace: &'static str,
    name: &'static str,
    attribute: Option<(&'static str, &'static str)>,
}

impl Step {
    const fn new(namespace: &'static str, name: &'static str) -> Self {
        Self {
            namespace,
            name,
            attribute: None,
        }
    }

    const fn with_attribute(self, name: &'static str, value: &'static str) -> Self {
        Self {
            attribute: Some((name, value)),
            ..self
        }
    }

    fn matches(&self, node: &Node) -> bool {
        node.is_element()
            && node.tag_name().namespace() == Some(self.namespace)
            && node.tag_name().name() == self.name
            && self
                .attribute
                .is_none_or(|(name, value)| node.attribute(name) == Some(value))
    }
}

/// How a field value is displayed.
#[derive(Debug, Clone, Copy)]
enum Format {
    Text,
    Date,
    Amount,
    Iban,
    InvoiceType,
    PaymentMeans,
    VatCategory,
    SpecificationIdentifier,
}

/// A business term and where to find it, relative to its section anchor.
struct FieldDef {
    id: &'static str,
    label: &'static str,
    path: &'static [Step],
    format: Format,
    /// All matches become fields (e.g. BT-22), instead of only the first.
    repeatable: bool,
}

/// A section and the element(s) it is anchored at.
struct SectionDef {
    title: &'static str,
    anchor: &'static [Step],
    /// One section per matching anchor element (e.g. BG-23).
    repeatable: bool,
    fields: &'static [FieldDef],
}

/// Document-wide information needed to display values.
struct Context<'a> {
    detection: &'a Detection,
    currency: Option<String>,
    prefixes: &'static [(&'static str, &'static str)],
}

fn map_section(root: Node, def: &SectionDef, context: &Context) -> Vec<Section> {
    let mut anchors = find_all(root, def.anchor);
    if !def.repeatable {
        anchors.truncate(1);
    }
    anchors
        .into_iter()
        .map(|anchor| Section {
            title: def.title.to_string(),
            fields: def
                .fields
                .iter()
                .flat_map(|field| map_field(anchor, field, context))
                .collect(),
        })
        .filter(|section| !section.fields.is_empty())
        .collect()
}

fn map_field(anchor: Node, def: &FieldDef, context: &Context) -> Vec<Field> {
    let mut nodes = find_all(anchor, def.path);
    if !def.repeatable {
        nodes.truncate(1);
    }
    nodes
        .into_iter()
        .map(|node| {
            let raw = node.text().unwrap_or_default().to_string();
            Field {
                id: def.id.to_string(),
                label: def.label.to_string(),
                display: display_value(&raw, node, def.format, context),
                location: location(node, context.prefixes),
                raw,
            }
        })
        .collect()
}

fn display_value(raw: &str, node: Node, format: Format, context: &Context) -> String {
    match format {
        Format::Text => raw.to_string(),
        Format::Date => display::date(raw, node.attribute("format")),
        Format::Amount => display::amount(raw, context.currency.as_deref()),
        Format::Iban => display::iban(raw),
        Format::InvoiceType => display::code(raw, display::INVOICE_TYPE_CODES),
        Format::PaymentMeans => display::code(raw, display::PAYMENT_MEANS_CODES),
        Format::VatCategory => display::code(raw, display::VAT_CATEGORY_CODES),
        Format::SpecificationIdentifier => {
            display::specification_identifier(raw, context.detection)
        }
    }
}

/// All elements reached from `node` along `path`, in document order. An
/// empty path selects `node` itself.
fn find_all<'a, 'input>(node: Node<'a, 'input>, path: &[Step]) -> Vec<Node<'a, 'input>> {
    let mut current = vec![node];
    for step in path {
        current = current
            .into_iter()
            .flat_map(|parent| parent.children().filter(|child| step.matches(child)))
            .collect();
    }
    current
}

/// Absolute location path of an element, e.g.
/// `/rsm:CrossIndustryInvoice/rsm:ExchangedDocument/ram:ID`, with a position
/// predicate where siblings share the same name.
fn location(node: Node, prefixes: &[(&str, &str)]) -> String {
    let mut steps: Vec<String> = node
        .ancestors()
        .filter(Node::is_element)
        .map(|element| {
            let tag = element.tag_name();
            let prefix = prefixes
                .iter()
                .find(|(_, namespace)| Some(*namespace) == tag.namespace())
                .map(|(prefix, _)| *prefix)
                .unwrap_or("unknown");
            let same_name =
                |sibling: &Node| sibling.is_element() && sibling.tag_name() == element.tag_name();
            let siblings = element
                .parent()
                .map(|parent| parent.children().filter(same_name).count())
                .unwrap_or(1);
            if siblings > 1 {
                let position = element.prev_siblings().filter(same_name).count();
                format!("{prefix}:{}[{position}]", tag.name())
            } else {
                format!("{prefix}:{}", tag.name())
            }
        })
        .collect();
    steps.reverse();
    format!("/{}", steps.join("/"))
}
