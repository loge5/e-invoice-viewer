//! Other data (spec 0031): elements with text that no field shows and that
//! the mapping does not use are listed at the end of the section they belong
//! to, so that nothing in an invoice is invisible.

use std::collections::HashSet;

use roxmltree::{Node, NodeId};

use super::{
    Condition, Field, FieldDef, FieldModel, Format, Section, SectionDef, Step, SyntaxDef, find_all,
};
use crate::i18n::{Language, tr};
use crate::location::Locations;

/// An element with this text below this parent is part of the syntax, not of
/// the invoice content (e.g. the tax scheme `VAT`).
pub(super) struct FixedValue {
    pub(super) parent: Step,
    pub(super) element: Step,
    pub(super) value: &'static str,
}

/// Adds the other data of the invoice at `root` to `model`. `used` holds
/// elements the mapping read beyond the fields (further occurrences of a
/// field with the same value).
pub(super) fn add(
    model: &mut FieldModel,
    root: Node,
    syntax: &SyntaxDef,
    used: &HashSet<NodeId>,
    locations: &Locations,
    language: Language,
) {
    let used = used_elements(root, syntax, used);
    let shown: HashSet<String> = model
        .sections
        .iter()
        .flat_map(|section| &section.fields)
        .map(|field| field.location.clone())
        .collect();

    let mut rows: Vec<(Option<usize>, Field)> = Vec::new();
    for node in root.descendants().filter(Node::is_element) {
        let Some(text) = own_text(node) else { continue };
        let location = locations.of(node);
        if used.contains(&node.id()) || shown.contains(location) {
            continue;
        }
        let section = model.section_at(location);
        let base = section.map_or(root, |index| {
            ancestor_at(node, &model.sections[index].location, locations).unwrap_or(root)
        });
        rows.push((
            section,
            Field {
                id: String::new(),
                label: label(node, base),
                raw: text.clone(),
                display: display(node, &text, language),
                location: location.to_string(),
                attribute: None,
            },
        ));
    }

    let mut unplaced = Vec::new();
    for (section, field) in rows {
        match section {
            Some(index) => model.sections[index].fields.push(field),
            None => unplaced.push(field),
        }
    }
    if !unplaced.is_empty() {
        model.sections.push(Section {
            title: tr(language, "Other data").to_string(),
            location: locations.of(root).to_string(),
            repeated: false,
            fields: unplaced,
        });
    }
}

/// The element's own text, whitespace normalised, if it has text and no
/// element children.
fn own_text(node: Node) -> Option<String> {
    if node.children().any(|child| child.is_element()) {
        return None;
    }
    let text = node
        .text()?
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!text.is_empty()).then_some(text)
}

/// The ancestor (or the node itself) at `location`.
fn ancestor_at<'a, 'input>(
    node: Node<'a, 'input>,
    location: &str,
    locations: &Locations,
) -> Option<Node<'a, 'input>> {
    node.ancestors()
        .filter(Node::is_element)
        .find(|ancestor| locations.of(*ancestor) == location)
}

/// The path from `base` (exclusive) to `node` in local names, e.g.
/// `ExchangedDocument › Name`.
fn label(node: Node, base: Node) -> String {
    let mut names: Vec<&str> = node
        .ancestors()
        .filter(Node::is_element)
        .take_while(|ancestor| *ancestor != base)
        .map(|ancestor| ancestor.tag_name().name())
        .collect();
    names.reverse();
    names.join(" › ")
}

/// The value as shown: the text, or the size of binary content, followed by
/// the attributes, e.g. `4012345000016 (schemeID: 0160)`.
fn display(node: Node, text: &str, language: Language) -> String {
    let value = if node.tag_name().name().ends_with("BinaryObject") {
        let kilobytes = (text.len() * 3 / 4).div_ceil(1000).max(1);
        tr(language, "binary data, {size}").replace("{size}", &format!("{kilobytes} kB"))
    } else {
        text.to_string()
    };
    let attributes: Vec<String> = node
        .attributes()
        .map(|attribute| format!("{}: {}", attribute.name(), attribute.value()))
        .collect();
    if attributes.is_empty() {
        value
    } else {
        format!("{value} ({})", attributes.join(", "))
    }
}

/// An element the mapping reads below every element matching `parent`.
struct Read {
    parent: Step,
    path: &'static [Step],
}

/// Elements the mapping uses without showing them as a field: what its
/// conditions, labels and titles read, fixed values of the syntax, the
/// supporting documents of the attachment list, and `further`.
fn used_elements(root: Node, syntax: &SyntaxDef, further: &HashSet<NodeId>) -> HashSet<NodeId> {
    let mut reads = Vec::new();
    for section in syntax.sections.iter().copied().flatten() {
        section_reads(section, &mut reads);
    }
    let mut used: HashSet<NodeId> = further.clone();
    for element in root.descendants().filter(Node::is_element) {
        for read in &reads {
            if read.parent.matches_name(&element) {
                used.extend(find_all(element, read.path).iter().map(Node::id));
            }
        }
        let parent = element.parent_element();
        let fixed = syntax.fixed_values.iter().any(|fixed| {
            fixed.element.matches_name(&element)
                && parent.is_some_and(|parent| fixed.parent.matches_name(&parent))
                && element.text().map(str::trim) == Some(fixed.value)
        });
        if fixed {
            used.insert(element.id());
        }
    }
    // The attachment list shows each supporting document as a whole.
    for document in find_all(root, syntax.supporting_documents.anchor) {
        used.extend(document.descendants().map(|node| node.id()));
    }
    used
}

fn section_reads(section: &'static SectionDef, reads: &mut Vec<Read>) {
    path_reads(section.anchor, reads);
    if let Some(fallback) = section.fallback {
        path_reads(fallback, reads);
    }
    if let (Some(suffix), Some(last)) = (section.title_suffix, section.anchor.last()) {
        reads.push(Read {
            parent: *last,
            path: suffix,
        });
    }
    fields_reads(section.fields, section.anchor.last().copied(), reads);
    for child in section.children {
        section_reads(child, reads);
    }
}

fn fields_reads(fields: &'static [FieldDef], parent: Option<Step>, reads: &mut Vec<Read>) {
    for field in fields {
        path_reads(field.path, reads);
        if let (Some(name), Some(parent)) = (field.label_from, parent) {
            reads.push(Read { parent, path: name });
        }
        if let (Format::SchemeElement(scheme), [.., parent, _]) = (field.format, field.path) {
            reads.push(Read {
                parent: *parent,
                path: scheme,
            });
        }
        if let Some(group) = field.group {
            fields_reads(group, field.path.last().copied(), reads);
        }
    }
}

/// The elements the conditions of a path read.
fn path_reads(path: &'static [Step], reads: &mut Vec<Read>) {
    for step in path {
        let read = match step.condition {
            Some(
                Condition::ChildText(path, _)
                | Condition::OtherChildText(path, _)
                | Condition::NotChildText(path, _)
                | Condition::HasChild(path)
                | Condition::NotChild(path),
            ) => path,
            _ => continue,
        };
        reads.push(Read {
            parent: *step,
            path: read,
        });
    }
}
