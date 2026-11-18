//! Location paths of XML elements, e.g.
//! `/rsm:CrossIndustryInvoice/rsm:ExchangedDocument/ram:ID`, with a 1-based
//! position predicate where siblings share the same name (`ram:Note[2]`).
//!
//! Field locations (0004), XML positions (0014) and finding locations of the
//! validation rules must match exactly; fields and XML positions take them
//! from here.

use std::collections::HashMap;

use roxmltree::{Document, Node};

/// The locations of all elements of a document, computed in one pass.
pub(crate) struct Locations {
    /// Indexed by node id; `None` for nodes that are not elements.
    by_node: Vec<Option<String>>,
}

impl Locations {
    /// `prefixes` maps namespace prefixes used in locations to namespaces.
    pub(crate) fn new(doc: &Document, prefixes: &[(&str, &str)]) -> Self {
        let mut by_node = vec![None; doc.descendants().count()];
        let root = doc.root_element();
        let root_location = format!("/{}", step(root, prefixes, None));
        let mut pending = vec![(root, root_location)];
        while let Some((element, location)) = pending.pop() {
            let children: Vec<Node> = element.children().filter(Node::is_element).collect();
            let mut totals: HashMap<_, usize> = HashMap::new();
            for child in &children {
                *totals.entry(name_of(*child)).or_default() += 1;
            }
            let mut seen: HashMap<_, usize> = HashMap::new();
            for child in children {
                let position = (totals[&name_of(child)] > 1).then(|| {
                    let seen = seen.entry(name_of(child)).or_default();
                    *seen += 1;
                    *seen
                });
                let child_location = format!("{location}/{}", step(child, prefixes, position));
                pending.push((child, child_location));
            }
            by_node[element.id().get_usize()] = Some(location);
        }
        Self { by_node }
    }

    /// The location of an element of the document.
    pub(crate) fn of(&self, element: Node) -> &str {
        self.by_node
            .get(element.id().get_usize())
            .and_then(Option::as_deref)
            .unwrap_or_default()
    }
}

/// Namespace and local name: siblings with the same one get positions.
fn name_of<'a, 'input>(node: Node<'a, 'input>) -> (Option<&'a str>, &'input str) {
    let tag = node.tag_name();
    (tag.namespace(), tag.name())
}

/// One step, e.g. `ram:ApplicableTradeTax[2]`.
fn step(element: Node, prefixes: &[(&str, &str)], position: Option<usize>) -> String {
    let tag = element.tag_name();
    let prefix = prefixes
        .iter()
        .find(|(_, namespace)| Some(*namespace) == tag.namespace())
        .map_or("unknown", |(prefix, _)| *prefix);
    match position {
        Some(position) => format!("{prefix}:{}[{position}]", tag.name()),
        None => format!("{prefix}:{}", tag.name()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_only_where_names_repeat() {
        let doc =
            Document::parse("<a xmlns='urn:a' xmlns:b='urn:b'><n/><x/><n/><b:n/><x2><n/></x2></a>")
                .expect("parse");
        let locations = Locations::new(&doc, &[("p", "urn:a"), ("q", "urn:b")]);
        let all: Vec<&str> = doc
            .descendants()
            .filter(Node::is_element)
            .map(|node| locations.of(node))
            .collect();
        assert_eq!(
            all,
            [
                "/p:a",
                "/p:a/p:n[1]",
                "/p:a/p:x",
                "/p:a/p:n[2]",
                "/p:a/q:n",
                "/p:a/p:x2",
                "/p:a/p:x2/p:n",
            ]
        );
    }
}
