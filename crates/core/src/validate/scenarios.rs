//! Severity changes of the KoSIT validator configuration for XRechnung
//! (spec 0010): the bundled `scenarios.xml`, unchanged. The first scenario
//! whose match expression holds for the invoice decides which `customLevel`s
//! apply, as in the KoSIT validator.

use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

use xee_xpath::context::StaticContextBuilder;
use xee_xpath::query::SequenceQuery;
use xee_xpath::{DocumentHandle, Documents, Queries, Query};

use super::Severity;

const SCENARIOS_NS: &str = "http://www.xoev.de/de/validator/framework/1/scenarios";

/// Scenario used when no scenario matches (e.g. an older declared XRechnung
/// version), by local name of the root element.
const FALLBACK_SCENARIOS: [(&str, &str); 3] = [
    ("CrossIndustryInvoice", "EN16931 XRechnung (CII)"),
    ("Invoice", "EN16931 XRechnung (UBL Invoice)"),
    ("CreditNote", "EN16931 XRechnung (UBL CreditNote)"),
];

pub(super) struct Scenarios {
    scenarios: Vec<Scenario>,
    root_name: SequenceQuery,
}

struct Scenario {
    name: String,
    matches: Option<SequenceQuery>,
    levels: HashMap<String, Severity>,
}

impl Scenarios {
    /// Compiles the bundled configuration. Panics only if it is not
    /// well-formed XML, which the unit tests rule out.
    pub(super) fn compile(xml: &str) -> Self {
        let doc = roxmltree::Document::parse(xml).expect("bundled configuration is well-formed");
        let scenarios = children(doc.root_element(), "scenario")
            .map(|scenario| {
                let mut builder = StaticContextBuilder::default();
                for namespace in children(scenario, "namespace") {
                    if let (Some(prefix), Some(uri)) =
                        (namespace.attribute("prefix"), namespace.text())
                    {
                        builder.add_namespace(prefix, uri.trim());
                    }
                }
                let queries = Queries::new(builder);
                let matches = children(scenario, "match")
                    .next()
                    .and_then(|node| node.text())
                    .and_then(|expression| compile(&queries, expression.trim()));
                let levels = children(scenario, "createReport")
                    .flat_map(|report| children(report, "customLevel"))
                    .filter_map(|level| {
                        let severity = match level.attribute("level")? {
                            "error" => Severity::Error,
                            "warning" => Severity::Warning,
                            "information" => Severity::Information,
                            _ => return None,
                        };
                        Some((level.text()?.trim().to_string(), severity))
                    })
                    .collect();
                Scenario {
                    name: children(scenario, "name")
                        .next()
                        .and_then(|node| node.text())
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                    matches,
                    levels,
                }
            })
            .collect();
        let root_name = compile(
            &Queries::new(StaticContextBuilder::default()),
            "local-name(/*)",
        )
        .expect("local-name(/*) compiles");
        Self {
            scenarios,
            root_name,
        }
    }

    /// The severity changes for an XRechnung invoice: those of the first
    /// matching scenario, else those of the XRechnung scenario for its
    /// document type.
    pub(super) fn levels(
        &self,
        documents: &mut Documents,
        document: DocumentHandle,
    ) -> Option<&HashMap<String, Severity>> {
        let matching = self.scenarios.iter().find(|scenario| {
            scenario.matches.as_ref().is_some_and(|query| {
                guarded(|| {
                    query
                        .execute(documents, document)
                        .ok()
                        .and_then(|sequence| sequence.effective_boolean_value().ok())
                })
                .flatten()
                .unwrap_or(false)
            })
        });
        let scenario = match matching {
            Some(scenario) => scenario,
            None => {
                let root = guarded(|| {
                    let sequence = self.root_name.execute(documents, document).ok()?;
                    sequence.iter().next()?.try_into_value::<String>().ok()
                })
                .flatten()?;
                let (_, name) = FALLBACK_SCENARIOS
                    .iter()
                    .find(|(local, _)| *local == root)?;
                self.scenario(name)?
            }
        };
        Some(&scenario.levels)
    }

    fn scenario(&self, name: &str) -> Option<&Scenario> {
        self.scenarios.iter().find(|scenario| scenario.name == name)
    }
}

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
            && child.tag_name().namespace() == Some(SCENARIOS_NS)
            && child.tag_name().name() == name
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundled() -> Scenarios {
        Scenarios::compile(super::super::KOSIT_SCENARIOS)
    }

    #[test]
    fn every_scenario_has_a_compiled_match() {
        let scenarios = bundled();
        assert!(scenarios.scenarios.len() >= 11);
        for scenario in &scenarios.scenarios {
            assert!(scenario.matches.is_some(), "{}", scenario.name);
        }
        for (_, name) in FALLBACK_SCENARIOS {
            assert!(scenarios.scenario(name).is_some(), "{name}");
        }
    }

    #[test]
    fn custom_levels_are_read_per_scenario() {
        let scenarios = bundled();
        let cii = &scenarios
            .scenario("EN16931 XRechnung (CII)")
            .expect("CII")
            .levels;
        assert_eq!(cii.get("CII-SR-465"), Some(&Severity::Error));
        assert_eq!(cii.get("CII-SR-475"), Some(&Severity::Information));
        assert_eq!(cii.get("BR-CL-23"), Some(&Severity::Warning));
        let extension = &scenarios
            .scenario("EN16931 XRechnung Extension (UBL Invoice)")
            .expect("UBL Extension")
            .levels;
        assert_eq!(extension.get("BR-CO-16"), Some(&Severity::Information));
        assert_eq!(extension.get("CII-SR-465"), None);
    }
}
