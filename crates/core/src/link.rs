//! Linking fields and XML elements in both directions (spec 0014).
//!
//! The XML view shows the invoice reformatted (0008); this module knows where
//! each element ended up in the reformatted text, so that a field or finding
//! location can be shown there, and a cursor position can be traced back to
//! its field or section.

use std::collections::HashMap;
use std::ops::Range;

use roxmltree::{Document, Node};

use crate::attribution::Target;
use crate::detect::{DetectError, detect};
use crate::fields::{FieldModel, location_prefixes, location_step};
use crate::xml_view::{Piece, decode, pretty_print_mapped};

/// The reformatted XML with the position of every element.
#[derive(Debug, Clone)]
pub struct XmlLayout {
    text: String,
    /// Byte offset of each line start.
    line_starts: Vec<usize>,
    /// All elements in document order.
    elements: Vec<Element>,
    by_location: HashMap<String, usize>,
}

/// Where an element is in the reformatted text (byte ranges).
#[derive(Debug, Clone)]
struct Element {
    location: String,
    parent: Option<usize>,
    /// From start tag to end tag.
    range: Range<usize>,
    /// Start of the element's own text, if it has text.
    text_start: Option<usize>,
    /// Local name and value range of each attribute.
    attributes: Vec<(String, Range<usize>)>,
}

impl XmlLayout {
    /// Reformats the invoice XML (as 0008) and records element positions.
    pub fn new(xml: &[u8]) -> Result<Self, DetectError> {
        let detection = detect(xml)?;
        let prefixes = location_prefixes(detection.syntax);
        let source = decode(xml)?;
        // `detect` has checked that the document is well-formed.
        let doc = Document::parse(&source).map_err(|_| DetectError::NotWellFormed { line: 1 })?;
        let (text, pieces) = pretty_print_mapped(&source);
        let map = |input: usize| output_offset(&pieces, input);

        let mut elements = Vec::new();
        let mut by_location = HashMap::new();
        // (element, parent index, parent location)
        let mut stack: Vec<(Node, Option<usize>, String)> =
            vec![(doc.root_element(), None, String::new())];
        while let Some((node, parent, parent_location)) = stack.pop() {
            let location = format!("{parent_location}/{}", location_step(node, prefixes));
            let range = node.range();
            let (Some(start), Some(last)) = (map(range.start), map(range.end - 1)) else {
                continue;
            };
            let text_start = node
                .first_child()
                .filter(|child| child.is_text() && node.children().count() == 1)
                .and_then(|child| {
                    let raw = child.text().unwrap_or_default();
                    let leading = raw.len() - raw.trim_start().len();
                    (!raw.trim().is_empty())
                        .then(|| map(child.range().start + leading))
                        .flatten()
                });
            let attributes = node
                .attributes()
                .filter_map(|attribute| {
                    let value = attribute.range_value();
                    let start = map(value.start)?;
                    Some((attribute.name().to_string(), start..start + value.len()))
                })
                .collect();
            let index = elements.len();
            by_location.insert(location.clone(), index);
            elements.push(Element {
                location: location.clone(),
                parent,
                range: start..last + 1,
                text_start,
                attributes,
            });
            // Children in reverse, so that they are visited in document order.
            let children: Vec<Node> = node.children().filter(Node::is_element).collect();
            for child in children.into_iter().rev() {
                stack.push((child, Some(index), location.clone()));
            }
        }

        let line_starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(index, _)| index + 1))
            .collect();
        Ok(Self {
            text,
            line_starts,
            elements,
            by_location,
        })
    }

    /// The reformatted text, as shown in the XML view.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Byte range of the element at `location`, from start to end tag.
    pub fn element_range(&self, location: &str) -> Option<Range<usize>> {
        self.element(location).map(|element| element.range.clone())
    }

    /// Byte offset where the cursor goes for the element at `location`: the
    /// start of the value of `attribute`, or of the element's text, or of
    /// the element.
    pub fn cursor_offset(&self, location: &str, attribute: Option<&str>) -> Option<usize> {
        let element = self.element(location)?;
        let attribute = attribute.and_then(|wanted| {
            element
                .attributes
                .iter()
                .find(|(name, _)| name == wanted)
                .map(|(_, range)| range.start)
        });
        Some(
            attribute
                .or(element.text_start)
                .unwrap_or(element.range.start),
        )
    }

    /// Line (0-based) and byte index within the line of a byte offset.
    pub fn position(&self, offset: usize) -> (usize, usize) {
        let line = self
            .line_starts
            .partition_point(|start| *start <= offset)
            .saturating_sub(1);
        (line, offset - self.line_starts[line])
    }

    /// Byte offset of a line (0-based) and byte index within the line.
    pub fn offset(&self, line: usize, index: usize) -> usize {
        self.line_starts
            .get(line)
            .map_or(self.text.len(), |start| start + index)
    }

    fn element(&self, location: &str) -> Option<&Element> {
        self.by_location
            .get(location)
            .map(|index| &self.elements[*index])
    }

    /// The deepest element containing `offset`, and the attribute whose
    /// value contains it, if any.
    fn element_at(&self, offset: usize) -> Option<(&Element, Option<&str>)> {
        let candidate = self
            .elements
            .partition_point(|element| element.range.start <= offset);
        let mut index = candidate.checked_sub(1)?;
        loop {
            let element = &self.elements[index];
            if element.range.contains(&offset) {
                let attribute = element
                    .attributes
                    .iter()
                    .find(|(_, range)| range.start <= offset && offset <= range.end)
                    .map(|(name, _)| name.as_str());
                return Some((element, attribute));
            }
            index = element.parent?;
        }
    }
}

/// Where byte `input` of the source ended up in the reformatted text, if it
/// is part of a copied piece.
fn output_offset(pieces: &[Piece], input: usize) -> Option<usize> {
    let index = pieces
        .partition_point(|piece| piece.input <= input)
        .checked_sub(1)?;
    let piece = pieces[index];
    (input < piece.input + piece.len).then(|| piece.output + (input - piece.input))
}

/// What the element at byte `offset` of the layout belongs to: a field, the
/// deepest section other than the root section, or nothing
/// (`Target::Document`).
pub fn assignment(model: &FieldModel, layout: &XmlLayout, offset: usize) -> Target {
    let Some((element, attribute)) = layout.element_at(offset) else {
        return Target::Document;
    };
    let fields = || {
        model.sections.iter().enumerate().flat_map(|(section, s)| {
            s.fields
                .iter()
                .enumerate()
                .filter(|(_, field)| field.location == element.location)
                .map(move |(field, f)| (section, field, f.attribute.as_deref()))
        })
    };
    let by_attribute = attribute.and_then(|wanted| fields().find(|(_, _, a)| *a == Some(wanted)));
    if let Some((section, field, _)) =
        by_attribute.or_else(|| fields().find(|(_, _, a)| a.is_none()))
    {
        return Target::Field { section, field };
    }

    let mut current = Some(element);
    while let Some(element) = current {
        let below_root = element.location.matches('/').count() > 1;
        if below_root
            && let Some(section) = model
                .sections
                .iter()
                .position(|section| section.location == element.location)
        {
            return Target::Section(section);
        }
        current = element.parent.map(|parent| &layout.elements[parent]);
    }
    Target::Document
}
