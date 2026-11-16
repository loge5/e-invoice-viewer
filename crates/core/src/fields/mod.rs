//! The field model: invoice content as sections of labelled business terms
//! (specs 0004 to 0006).

mod cii;
mod display;

use roxmltree::{Document, Node};
use thiserror::Error;

use crate::detect::{DetectError, Detection, Syntax, detect};
use crate::xml_view::decode;

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
    // `detect` has already checked that the document decodes and is
    // well-formed without a DOCTYPE, so parsing it again cannot fail
    // differently.
    let text = decode(xml)?;
    let doc = Document::parse(&text).map_err(|_| MapError::UnsupportedSyntax)?;
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
        .chain(cii::LINE_SECTIONS)
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
    /// A quantity with its unit code taken from the `unitCode` attribute.
    Quantity,
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
    /// Take the value from this attribute instead of the element text.
    attribute: Option<&'static str>,
}

/// A section and the element(s) it is anchored at.
struct SectionDef {
    title: &'static str,
    anchor: &'static [Step],
    /// One section per matching anchor element (e.g. BG-23).
    repeatable: bool,
    /// Element whose text is appended to the title (e.g. the line ID).
    title_suffix: Option<&'static [Step]>,
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
        .map(|anchor| {
            // Computed once per section: counting same-name siblings for
            // every field would be quadratic in the number of lines.
            let anchor_location = location(anchor, None, context.prefixes);
            Section {
                title: section_title(anchor, def),
                fields: def
                    .fields
                    .iter()
                    .flat_map(|field| map_field((anchor, &anchor_location), field, context))
                    .collect(),
            }
        })
        .filter(|section| !section.fields.is_empty())
        .collect()
}

fn section_title(anchor: Node, def: &SectionDef) -> String {
    let suffix = def
        .title_suffix
        .and_then(|path| find_all(anchor, path).first().copied())
        .and_then(|node| node.text())
        .map(str::trim)
        .filter(|text| !text.is_empty());
    match suffix {
        Some(suffix) => format!("{} {suffix}", def.title),
        None => def.title.to_string(),
    }
}

fn map_field(
    (anchor, anchor_location): (Node, &str),
    def: &FieldDef,
    context: &Context,
) -> Vec<Field> {
    let mut nodes = find_all(anchor, def.path);
    if !def.repeatable {
        nodes.truncate(1);
    }
    nodes
        .into_iter()
        .filter(|node| def.attribute.is_none_or(|name| node.has_attribute(name)))
        .map(|node| {
            let raw = match def.attribute {
                Some(name) => node.attribute(name).unwrap_or_default(),
                None => node.text().unwrap_or_default(),
            }
            .to_string();
            Field {
                id: def.id.to_string(),
                label: def.label.to_string(),
                display: display_value(&raw, node, def.format, context),
                location: location(node, Some((anchor, anchor_location)), context.prefixes),
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
        Format::Quantity => display::quantity(raw, node.attribute("unitCode")),
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
/// predicate where siblings share the same name. If `base` is given and is an
/// ancestor (or the element itself), its known location is reused.
fn location(node: Node, base: Option<(Node, &str)>, prefixes: &[(&str, &str)]) -> String {
    let mut steps = Vec::new();
    for element in node.ancestors().filter(Node::is_element) {
        if let Some((base_node, base_location)) = base
            && element == base_node
        {
            steps.push(base_location.to_string());
            steps.reverse();
            return steps.join("/");
        }
        steps.push(location_step(element, prefixes));
    }
    steps.reverse();
    format!("/{}", steps.join("/"))
}

/// One location step, e.g. `ram:ApplicableTradeTax[2]`.
fn location_step(element: Node, prefixes: &[(&str, &str)]) -> String {
    let tag = element.tag_name();
    let prefix = prefixes
        .iter()
        .find(|(_, namespace)| Some(*namespace) == tag.namespace())
        .map(|(prefix, _)| *prefix)
        .unwrap_or("unknown");
    let same_name = |sibling: &Node| sibling.is_element() && sibling.tag_name() == tag;
    let siblings = element
        .parent()
        .map(|parent| parent.children().filter(same_name).count())
        .unwrap_or(1);
    if siblings > 1 {
        // `prev_siblings` includes the element itself, giving a 1-based index.
        let position = element.prev_siblings().filter(same_name).count();
        format!("{prefix}:{}[{position}]", tag.name())
    } else {
        format!("{prefix}:{}", tag.name())
    }
}
