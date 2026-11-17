//! A minimal Schematron runner for the official rule artefacts
//! (`queryBinding="xslt2"`), as decided in ADR 0002.
//!
//! - Rule contexts (XSLT match patterns) become XPath expressions: every
//!   top-level union branch that does not start with `/` gets a `//` prefix.
//! - Within a pattern, a node is checked only by the first rule whose context
//!   matches it.
//! - An assert is violated if its test is false. A test that raises a dynamic
//!   error counts as violated too: the engine does not short-circuit `and`,
//!   where Saxon (used by the official validator) does.
//! - Tests that start with `/` do not depend on the context node and are
//!   evaluated once per document.

use std::collections::{HashMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};

use xee_xpath::context::StaticContextBuilder;
use xee_xpath::query::SequenceQuery;
use xee_xpath::{Documents, Item, Queries, Query};

use super::{Finding, Severity, business_terms};

const SCHEMATRON_NS: &str = "http://purl.oclc.org/dsdl/schematron";

/// A compiled rule set.
pub(super) struct RuleSet {
    /// Namespace prefixes declared by the rule set, used for locations.
    prefixes: Vec<(String, String)>,
    patterns: Vec<Vec<Rule>>,
    /// IDs of asserts that could not be compiled (none for the official
    /// artefacts; see the unit test). They are skipped.
    #[cfg_attr(not(test), allow(dead_code, reason = "checked by unit tests"))]
    uncompiled: Vec<String>,
}

struct Rule {
    context: Option<SequenceQuery>,
    asserts: Vec<Assert>,
}

struct Assert {
    id: String,
    severity: Severity,
    message: String,
    test: SequenceQuery,
    /// The test does not depend on the context node.
    absolute: bool,
}

impl RuleSet {
    /// Compiles a Schematron file. Panics only if the bundled file is not
    /// well-formed XML, which the unit tests rule out.
    pub(super) fn compile(schematron: &str) -> Self {
        let doc = roxmltree::Document::parse(schematron).expect("bundled rules are well-formed");
        let root = doc.root_element();
        let prefixes: Vec<(String, String)> = children(root, "ns")
            .filter_map(|ns| {
                Some((
                    ns.attribute("prefix")?.to_string(),
                    ns.attribute("uri")?.to_string(),
                ))
            })
            .collect();

        let mut builder = StaticContextBuilder::default();
        for (prefix, uri) in &prefixes {
            builder.add_namespace(prefix, uri);
        }
        let queries = Queries::new(builder);

        let mut uncompiled = Vec::new();
        let patterns = children(root, "pattern")
            .map(|pattern| {
                children(pattern, "rule")
                    .map(|rule| {
                        let context = rule
                            .attribute("context")
                            .and_then(|context| compile(&queries, &context_expression(context)));
                        let asserts = children(rule, "assert")
                            .filter_map(|assert| {
                                let id = assert.attribute("id").unwrap_or_default().to_string();
                                let source = assert.attribute("test").unwrap_or_default();
                                let Some(test) = compile(&queries, source) else {
                                    uncompiled.push(id);
                                    return None;
                                };
                                Some(Assert {
                                    severity: severity(assert.attribute("flag")),
                                    message: normalize_space(&text_content(assert)),
                                    absolute: source.trim_start().starts_with('/'),
                                    id,
                                    test,
                                })
                            })
                            .collect();
                        if context.is_none() {
                            uncompiled.extend(
                                children(rule, "assert")
                                    .filter_map(|a| a.attribute("id").map(str::to_string)),
                            );
                        }
                        Rule { context, asserts }
                    })
                    .collect()
            })
            .collect();
        Self {
            prefixes,
            patterns,
            uncompiled,
        }
    }

    #[cfg(test)]
    pub(super) fn uncompiled(&self) -> Vec<&str> {
        self.uncompiled.iter().map(String::as_str).collect()
    }

    #[cfg(test)]
    pub(super) fn assert_count(&self) -> usize {
        self.patterns
            .iter()
            .flatten()
            .map(|rule| rule.asserts.len())
            .sum()
    }

    /// Evaluates all rules against an invoice and returns the violations.
    pub(super) fn evaluate(&self, xml: &str) -> Vec<Finding> {
        let mut documents = Documents::new();
        let Ok(document) = documents.add_string_without_uri(xml) else {
            // The invoice was parsed successfully by detection before; an
            // engine that cannot load it has nothing to report.
            return Vec::new();
        };
        let mut findings = Vec::new();
        let mut absolute_results: HashMap<*const Assert, bool> = HashMap::new();
        for rules in &self.patterns {
            let mut checked: HashSet<xot::Node> = HashSet::new();
            for rule in rules {
                let Some(context) = &rule.context else {
                    continue;
                };
                let Some(nodes) = guarded(|| context.execute(&mut documents, document).ok())
                    .flatten()
                    .map(|sequence| sequence.iter().collect::<Vec<Item>>())
                else {
                    continue;
                };
                for item in nodes {
                    let Ok(node) = item.to_node() else { continue };
                    if !checked.insert(node) {
                        continue;
                    }
                    for assert in &rule.asserts {
                        let key: *const Assert = assert;
                        let holds = match absolute_results.get(&key) {
                            Some(holds) => *holds,
                            None => {
                                let holds = test_holds(&assert.test, &mut documents, &item);
                                if assert.absolute {
                                    absolute_results.insert(key, holds);
                                }
                                holds
                            }
                        };
                        if !holds {
                            findings.push(Finding {
                                rule_id: assert.id.clone(),
                                severity: assert.severity,
                                business_terms: business_terms(&assert.message),
                                message: assert.message.clone(),
                                location: location(documents.xot(), node, &self.prefixes),
                            });
                        }
                    }
                }
            }
        }
        findings
    }
}

/// Whether an assert test holds for a context item; errors and engine
/// panics count as "does not hold".
fn test_holds(test: &SequenceQuery, documents: &mut Documents, item: &Item) -> bool {
    guarded(|| {
        test.execute(documents, item)
            .ok()
            .and_then(|sequence| sequence.effective_boolean_value().ok())
    })
    .flatten()
    .unwrap_or(false)
}

/// Runs engine code, turning a panic into `None`.
fn guarded<T>(f: impl FnOnce() -> T) -> Option<T> {
    catch_unwind(AssertUnwindSafe(f)).ok()
}

fn compile(queries: &Queries, expression: &str) -> Option<SequenceQuery> {
    guarded(|| queries.sequence(expression).ok()).flatten()
}

fn children<'a, 'input>(
    node: roxmltree::Node<'a, 'input>,
    name: &'static str,
) -> impl Iterator<Item = roxmltree::Node<'a, 'input>> {
    node.children().filter(move |child| {
        child.is_element()
            && child.tag_name().namespace() == Some(SCHEMATRON_NS)
            && child.tag_name().name() == name
    })
}

fn severity(flag: Option<&str>) -> Severity {
    match flag {
        Some("warning") => Severity::Warning,
        Some("information") => Severity::Information,
        _ => Severity::Error,
    }
}

fn text_content(node: roxmltree::Node) -> String {
    node.descendants()
        .filter(roxmltree::Node::is_text)
        .filter_map(|text| text.text())
        .collect()
}

fn normalize_space(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Turns an XSLT match pattern into an XPath expression selecting all nodes
/// it matches.
fn context_expression(pattern: &str) -> String {
    let mut branches = Vec::new();
    let (mut depth, mut quote, mut start) = (0i32, None, 0);
    for (index, c) in pattern.char_indices() {
        match (quote, c) {
            (None, '\'' | '"') => quote = Some(c),
            (Some(open), _) if c == open => quote = None,
            (None, '[' | '(') => depth += 1,
            (None, ']' | ')') => depth -= 1,
            (None, '|') if depth == 0 => {
                branches.push(&pattern[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    branches.push(&pattern[start..]);
    branches
        .iter()
        .map(|branch| {
            let branch = branch.trim();
            if branch.starts_with('/') {
                branch.to_string()
            } else {
                format!("//{branch}")
            }
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

/// 1-based location path of the element a rule was checked on, e.g.
/// `/rsm:CrossIndustryInvoice/rsm:SupplyChainTradeTransaction/ram:IncludedSupplyChainTradeLineItem[2]`.
/// For an attribute, its element.
fn location(xot: &xot::Xot, node: xot::Node, prefixes: &[(String, String)]) -> String {
    let mut steps = Vec::new();
    let mut current = Some(node);
    while let Some(node) = current {
        if let Some(element) = xot.element(node) {
            let name = element.name();
            let (local, namespace) = xot.name_ns_str(name);
            let prefix = prefixes
                .iter()
                .find(|(_, uri)| uri == namespace)
                .map_or("unknown", |(prefix, _)| prefix.as_str());
            let same_name: Vec<xot::Node> = xot
                .parent(node)
                .map(|parent| {
                    xot.children(parent)
                        .filter(|sibling| xot.element(*sibling).is_some_and(|e| e.name() == name))
                        .collect()
                })
                .unwrap_or_default();
            if same_name.len() > 1 {
                let position = same_name
                    .iter()
                    .position(|sibling| *sibling == node)
                    .map_or(1, |p| p + 1);
                steps.push(format!("{prefix}:{local}[{position}]"));
            } else {
                steps.push(format!("{prefix}:{local}"));
            }
        }
        current = xot.parent(node);
    }
    steps.reverse();
    format!("/{}", steps.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_union_branches_get_a_descendant_prefix() {
        assert_eq!(
            context_expression("cac:A[x = 'a|b'] | /ubl:Invoice | cac:B[(1|2)]"),
            "//cac:A[x = 'a|b'] | /ubl:Invoice | //cac:B[(1|2)]"
        );
    }
}
