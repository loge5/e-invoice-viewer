//! The field model: invoice content as sections of labelled business terms
//! (specs 0004 to 0006), and the other data of the invoice (spec 0031).

mod cii;
mod display;
mod other;
mod terms;
mod ubl;

use std::cell::RefCell;
use std::collections::HashSet;

use roxmltree::{Document, Node, NodeId};
use thiserror::Error;

use crate::attachments::{self, Attachment};
use crate::detect::{DetectError, Detection, Syntax, detect};
use crate::i18n::{Language, tr};
use crate::location::Locations;
use crate::xml_view::decode;

/// Invoice content as an ordered list of sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldModel {
    pub sections: Vec<Section>,
}

impl FieldModel {
    /// All fields with their section and field index.
    pub fn fields(&self) -> impl Iterator<Item = (usize, usize, &Field)> {
        self.sections.iter().enumerate().flat_map(|(section, s)| {
            s.fields
                .iter()
                .enumerate()
                .map(move |(field, f)| (section, field, f))
        })
    }

    /// The deepest section whose element contains `location`, except
    /// sections of the root element.
    pub fn section_at(&self, location: &str) -> Option<usize> {
        self.sections
            .iter()
            .enumerate()
            .filter(|(_, section)| {
                is_below_root(&section.location) && within(location, &section.location)
            })
            .max_by_key(|(_, section)| section.location.len())
            .map(|(index, _)| index)
    }
}

/// Whether `location` is `ancestor` or lies below it.
pub(crate) fn within(location: &str, ancestor: &str) -> bool {
    location
        .strip_prefix(ancestor)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

/// Whether a location has more than one step, i.e. is not the root element.
fn is_below_root(location: &str) -> bool {
    location.trim_start_matches('/').contains('/')
}

/// A titled group of fields, e.g. "Verkäufer (BG-4)".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub title: String,
    /// XPath of the element the section stands for (e.g. the invoice line);
    /// the root element for sections of the whole invoice.
    pub location: String,
    /// One of several sections of the same kind (invoice lines, VAT
    /// breakdowns).
    pub repeated: bool,
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
    /// The attribute of that element holding the value (e.g. `unitCode` for
    /// BT-130), `None` for the element text.
    pub attribute: Option<String>,
}

/// Why no field model could be built.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MapError {
    #[error(transparent)]
    Detect(#[from] DetectError),
    #[error("syntax not supported yet")]
    UnsupportedSyntax,
}

/// Builds the field model of an invoice XML, with labels, titles and values
/// in `language` (spec 0017).
pub fn map_invoice(xml: &[u8], language: Language) -> Result<FieldModel, MapError> {
    read_invoice(xml, language).map(|(model, _)| model)
}

/// The field model and the supporting documents (BG-24, spec 0013) of an
/// invoice XML, from one parse.
pub(crate) fn read_invoice(
    xml: &[u8],
    language: Language,
) -> Result<(FieldModel, Vec<Attachment>), MapError> {
    let detection = detect(xml)?;
    let syntax = match detection.syntax {
        Syntax::Cii => &cii::SYNTAX,
        Syntax::Ubl => &ubl::SYNTAX,
    };
    // `detect` has already checked that the document decodes and is
    // well-formed without a DOCTYPE, so parsing it again cannot fail
    // differently.
    let text = decode(xml)?;
    let doc = Document::parse(&text).map_err(|_| MapError::UnsupportedSyntax)?;
    let root = doc.root_element();

    let locations = Locations::new(&doc, syntax.prefixes);
    let context = Context {
        detection: &detection,
        language,
        root,
        locations: &locations,
        currency: find_all(root, syntax.currency_path)
            .first()
            .and_then(|node| node.text())
            .map(|text| text.trim().to_string()),
        used: RefCell::new(HashSet::new()),
    };

    let sections = syntax
        .sections
        .iter()
        .copied()
        .flatten()
        .flat_map(|def| map_section(root, def, &context))
        .collect();
    let supporting = supporting_documents(root, &syntax.supporting_documents);
    let mut model = FieldModel { sections };
    other::add(
        &mut model,
        root,
        syntax,
        &context.used.borrow(),
        &locations,
        language,
    );
    Ok((model, supporting))
}

/// The supporting documents with embedded content or an external address.
fn supporting_documents(root: Node, def: &SupportingDocumentsDef) -> Vec<Attachment> {
    find_all(root, def.anchor)
        .into_iter()
        .filter_map(|node| {
            let text = |path| {
                find_all(node, path)
                    .first()
                    .and_then(|child| child.text())
                    .map(str::trim)
                    .filter(|text| !text.is_empty())
                    .map(str::to_string)
            };
            attachments::supporting_document(
                find_all(node, def.binary).first().copied(),
                text(def.address),
                def.description.iter().find_map(|path| text(path)),
            )
        })
        .collect()
}

/// The namespace prefixes of locations in `syntax`.
pub(crate) fn location_prefixes(syntax: Syntax) -> &'static [(&'static str, &'static str)] {
    match syntax {
        Syntax::Cii => cii::SYNTAX.prefixes,
        Syntax::Ubl => ubl::SYNTAX.prefixes,
    }
}

/// Where the business terms live in one syntax.
struct SyntaxDef {
    /// Namespace prefixes used in locations; the same as the validation
    /// rules use, so that finding and field locations can be compared.
    prefixes: &'static [(&'static str, &'static str)],
    /// Path from the root to the invoice currency (BT-5), used for amounts.
    currency_path: &'static [Step],
    /// Groups of sections in display order: header (0004), lines (0005),
    /// trailer.
    sections: &'static [&'static [SectionDef]],
    supporting_documents: SupportingDocumentsDef,
    /// Elements that belong to the syntax, not to the invoice content
    /// (spec 0031).
    fixed_values: &'static [other::FixedValue],
}

/// Where the additional supporting documents (BG-24) are.
struct SupportingDocumentsDef {
    /// From the root to each supporting document.
    anchor: &'static [Step],
    /// The attached document (BT-125).
    binary: &'static [Step],
    /// The external address (BT-124).
    address: &'static [Step],
    /// Candidates for the description; the first with text is used.
    description: &'static [&'static [Step]],
}

/// One step of a path: an element with namespace and local name (or one of
/// two names), optionally restricted by a condition.
#[derive(Debug, Clone, Copy)]
struct Step {
    namespace: &'static str,
    name: &'static str,
    /// A second accepted name, e.g. `CreditNoteLine` next to `InvoiceLine`.
    alternative: Option<&'static str>,
    condition: Option<Condition>,
}

/// A restriction on the element of a step.
#[derive(Debug, Clone, Copy)]
enum Condition {
    /// The attribute has this value.
    Attribute(&'static str, &'static str),
    /// The element reached along the path has this text.
    ChildText(&'static [Step], &'static str),
    /// The element reached along the path exists and has another text.
    OtherChildText(&'static [Step], &'static str),
    /// The element reached along the path is missing or has another text.
    NotChildText(&'static [Step], &'static str),
    /// An element is reached along the path.
    HasChild(&'static [Step]),
    /// No element is reached along the path.
    NotChild(&'static [Step]),
    /// The attribute is missing or has another value.
    NotAttribute(&'static str, &'static str),
    /// The attribute equals the text of the element at the path from the
    /// root element (e.g. a VAT amount in the VAT accounting currency BT-6).
    AttributeIsTextAt(&'static str, &'static [Step]),
    /// The attribute is missing, or differs from the text at the path from
    /// the root element.
    AttributeIsNotTextAt(&'static str, &'static [Step]),
}

impl Step {
    const fn new(namespace: &'static str, name: &'static str) -> Self {
        Self {
            namespace,
            name,
            alternative: None,
            condition: None,
        }
    }

    const fn or(self, alternative: &'static str) -> Self {
        Self {
            alternative: Some(alternative),
            ..self
        }
    }

    const fn with_attribute(self, name: &'static str, value: &'static str) -> Self {
        self.with(Condition::Attribute(name, value))
    }

    const fn with(self, condition: Condition) -> Self {
        Self {
            condition: Some(condition),
            ..self
        }
    }

    /// Whether the element has the step's name, whatever its condition.
    fn matches_name(&self, node: &Node) -> bool {
        let name = node.tag_name().name();
        node.is_element()
            && node.tag_name().namespace() == Some(self.namespace)
            && (name == self.name || Some(name) == self.alternative)
    }

    fn matches(&self, node: &Node) -> bool {
        let name = node.tag_name().name();
        node.is_element()
            && node.tag_name().namespace() == Some(self.namespace)
            && (name == self.name || Some(name) == self.alternative)
            && self
                .condition
                .is_none_or(|condition| condition.holds(*node))
    }
}

impl Condition {
    fn holds(self, node: Node) -> bool {
        let text_at = |path| {
            find_all(node, path)
                .first()
                .map(|child| child.text().unwrap_or_default().trim().to_string())
        };
        match self {
            Self::Attribute(name, value) => node.attribute(name) == Some(value),
            Self::ChildText(path, value) => text_at(path).is_some_and(|text| text == value),
            Self::OtherChildText(path, value) => text_at(path).is_some_and(|text| text != value),
            Self::NotChildText(path, value) => text_at(path).is_none_or(|text| text != value),
            Self::HasChild(path) => !find_all(node, path).is_empty(),
            Self::NotChild(path) => find_all(node, path).is_empty(),
            Self::NotAttribute(name, value) => node.attribute(name) != Some(value),
            Self::AttributeIsTextAt(name, path) => {
                let root = node.document().root_element();
                let wanted = find_all(root, path)
                    .first()
                    .and_then(|element| element.text())
                    .map(str::trim);
                wanted.is_some() && node.attribute(name).map(str::trim) == wanted
            }
            Self::AttributeIsNotTextAt(name, path) => {
                !Self::AttributeIsTextAt(name, path).holds(node)
            }
        }
    }
}

/// How a field value is displayed.
#[derive(Debug, Clone, Copy)]
enum Format {
    Text,
    /// A CII date with its `format` attribute (102 = YYYYMMDD).
    Date,
    /// A UBL date, `YYYY-MM-DD`.
    IsoDate,
    Amount,
    /// A quantity with its unit code taken from the `unitCode` attribute.
    Quantity,
    Iban,
    InvoiceType,
    PaymentMeans,
    VatCategory,
    SpecificationIdentifier,
    /// A value with its scheme from the named attribute, e.g. `schemeID`
    /// for electronic addresses (BT-34) or `listID` for classifications.
    Scheme(&'static str),
    /// A value with its scheme from a sibling element, e.g. the reference
    /// type code of a CII invoiced object identifier (BT-128-1).
    SchemeElement(&'static [Step]),
    /// An amount in the currency of its own `currencyID` attribute (BT-111).
    OwnCurrencyAmount,
    /// The subject code of a UBL note written as `#AAI#text` (BT-21).
    NoteSubject,
    /// The text of a UBL note without a leading `#code#` (BT-22).
    NoteText,
}

/// A business term: its number and German label.
#[derive(Debug, Clone, Copy)]
struct Term {
    id: &'static str,
    label: &'static str,
}

/// A business term with its number and English label; the label is marked
/// for translation.
const fn term(id: &'static str, label: &'static str) -> Term {
    Term { id, label }
}

/// A business term and where to find it, relative to its section anchor.
struct FieldDef {
    term: Term,
    path: &'static [Step],
    format: Format,
    /// All matches become fields (e.g. BT-22), instead of only the first.
    repeatable: bool,
    /// Take the value from this attribute instead of the element text.
    attribute: Option<&'static str>,
    /// The path starts at the root element instead of the section anchor
    /// (e.g. UBL BT-110, which is outside the monetary total).
    absolute: bool,
    /// The path selects group elements (e.g. each note); these fields are
    /// mapped for each of them in turn, relative to it.
    group: Option<&'static [FieldDef]>,
    /// The label is the text at this path (relative to the anchor or group
    /// element) instead of the term's label (item attributes, BG-32).
    label_from: Option<&'static [Step]>,
}

/// A field with the first value found at `path`.
const fn field(term: Term, path: &'static [Step], format: Format) -> FieldDef {
    FieldDef {
        term,
        path,
        format,
        repeatable: false,
        attribute: None,
        absolute: false,
        group: None,
        label_from: None,
    }
}

/// A field for every value found at `path` (e.g. BT-22).
const fn repeatable(term: Term, path: &'static [Step], format: Format) -> FieldDef {
    FieldDef {
        repeatable: true,
        ..field(term, path, format)
    }
}

/// A field whose value is the attribute `attribute` of the element at `path`.
const fn from_attribute(term: Term, path: &'static [Step], attribute: &'static str) -> FieldDef {
    FieldDef {
        attribute: Some(attribute),
        ..field(term, path, Format::Text)
    }
}

/// A field whose path starts at the root element instead of the section
/// anchor (e.g. UBL BT-110, which is outside the monetary total).
const fn absolute(term: Term, path: &'static [Step], format: Format) -> FieldDef {
    FieldDef {
        absolute: true,
        ..field(term, path, format)
    }
}

/// The fields `fields` for each element at `path`, in document order (e.g.
/// subject code and text of each note, BT-21 and BT-22).
const fn group(path: &'static [Step], fields: &'static [FieldDef]) -> FieldDef {
    FieldDef {
        repeatable: true,
        group: Some(fields),
        ..field(Term { id: "", label: "" }, path, Format::Text)
    }
}

/// A field whose label is the text at `name` (relative to the anchor or
/// group element), e.g. an item attribute (BT-160 name, BT-161 value).
const fn named(term: Term, path: &'static [Step], name: &'static [Step]) -> FieldDef {
    FieldDef {
        label_from: Some(name),
        ..field(term, path, Format::Text)
    }
}

/// A section and the element(s) it is anchored at.
struct SectionDef {
    title: &'static str,
    anchor: &'static [Step],
    /// Anchor used when no element matches `anchor` (e.g. the UBL invoicing
    /// period for the delivery section, spec 0021).
    fallback: Option<&'static [Step]>,
    /// One section per matching anchor element (e.g. BG-23).
    repeatable: bool,
    /// Element whose text is appended to the title (e.g. the line ID).
    title_suffix: Option<&'static [Step]>,
    fields: &'static [FieldDef],
    /// Sections inside each section of this kind, placed directly after it
    /// and titled with it (e.g. allowances of an invoice line, BG-27).
    children: &'static [SectionDef],
}

/// Document-wide information needed to display values.
struct Context<'a, 'input> {
    detection: &'a Detection,
    language: Language,
    root: Node<'a, 'input>,
    currency: Option<String>,
    locations: &'a Locations,
    /// Further occurrences of single fields with the same value (spec 0031).
    used: RefCell<HashSet<NodeId>>,
}

fn map_section(root: Node, def: &SectionDef, context: &Context) -> Vec<Section> {
    let mut anchors = find_all(root, def.anchor);
    if anchors.is_empty()
        && let Some(fallback) = def.fallback
    {
        anchors = find_all(root, fallback);
    }
    if !def.repeatable {
        anchors.truncate(1);
    }
    let mut sections = Vec::new();
    for anchor in anchors {
        let section = Section {
            title: section_title(anchor, def, context.language),
            location: context.locations.of(anchor).to_string(),
            repeated: def.repeatable,
            fields: def
                .fields
                .iter()
                .flat_map(|field| map_field(anchor, field, context))
                .collect(),
        };
        let children: Vec<Section> = def
            .children
            .iter()
            .flat_map(|child| map_section(anchor, child, context))
            .map(|child| Section {
                title: format!("{} – {}", section.title, child.title),
                ..child
            })
            .collect();
        if !section.fields.is_empty() {
            sections.push(section);
        }
        sections.extend(children);
    }
    sections
}

fn section_title(anchor: Node, def: &SectionDef, language: Language) -> String {
    let title = tr(language, def.title);
    let suffix = def
        .title_suffix
        .and_then(|path| find_all(anchor, path).first().copied())
        .and_then(|node| node.text())
        .map(str::trim)
        .filter(|text| !text.is_empty());
    match suffix {
        Some(suffix) => format!("{title} {suffix}"),
        None => title.to_string(),
    }
}

fn map_field(anchor: Node, def: &FieldDef, context: &Context) -> Vec<Field> {
    let mut nodes = find_all(if def.absolute { context.root } else { anchor }, def.path);
    if !def.repeatable {
        // Further occurrences with the same value add nothing; others are
        // listed as other data (spec 0031).
        if let Some((first, further)) = nodes.split_first() {
            let value = |node: &Node| match def.attribute {
                Some(name) => node.attribute(name).map(str::trim).map(str::to_string),
                None => node.text().map(str::trim).map(str::to_string),
            };
            let mut used = context.used.borrow_mut();
            for node in further.iter().filter(|node| value(node) == value(first)) {
                used.extend(node.descendants().map(|n| n.id()));
            }
        }
        nodes.truncate(1);
    }
    if let Some(fields) = def.group {
        return nodes
            .into_iter()
            .flat_map(|node| {
                fields
                    .iter()
                    .flat_map(move |field| map_field(node, field, context))
            })
            .collect();
    }
    let label = def
        .label_from
        .and_then(|path| find_all(anchor, path).first().copied())
        .and_then(|name| name.text())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map_or_else(
            || tr(context.language, def.term.label).to_string(),
            str::to_string,
        );
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
                id: def.term.id.to_string(),
                label: label.clone(),
                display: display_value(&raw, node, def.format, context),
                location: context.locations.of(node).to_string(),
                attribute: def.attribute.map(str::to_string),
                raw,
            }
        })
        // A UBL note without a `#code#` has no subject code (BT-21).
        .filter(|field| !(matches!(def.format, Format::NoteSubject) && field.display.is_empty()))
        .collect()
}

fn display_value(raw: &str, node: Node, format: Format, context: &Context) -> String {
    let language = context.language;
    match format {
        Format::Text => raw.to_string(),
        Format::Date => display::date(raw, node.attribute("format"), language),
        Format::IsoDate => display::iso_date(raw, language),
        Format::Amount => display::amount(raw, context.currency.as_deref(), language),
        Format::Quantity => display::quantity(raw, node.attribute("unitCode"), language),
        Format::Iban => display::iban(raw),
        Format::InvoiceType => display::code(raw, display::INVOICE_TYPE_CODES, language),
        Format::PaymentMeans => display::code(raw, display::PAYMENT_MEANS_CODES, language),
        Format::VatCategory => display::code(raw, display::VAT_CATEGORY_CODES, language),
        Format::SpecificationIdentifier => {
            display::specification_identifier(raw, context.detection)
        }
        Format::Scheme(attribute) => display::electronic_address(raw, node.attribute(attribute)),
        Format::SchemeElement(path) => {
            let scheme = node
                .parent_element()
                .and_then(|parent| find_all(parent, path).first().copied())
                .and_then(|element| element.text());
            display::electronic_address(raw, scheme)
        }
        Format::OwnCurrencyAmount => display::amount(raw, node.attribute("currencyID"), language),
        Format::NoteSubject => display::note_parts(raw).0.unwrap_or_default().to_string(),
        Format::NoteText => display::note_parts(raw).1.to_string(),
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
