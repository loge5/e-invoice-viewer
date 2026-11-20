//! Validation against the XML schema (spec 0023), the official EN 16931
//! business rules (spec 0009), for XRechnung invoices the XRechnung CIUS
//! rules (spec 0010), and for ZUGFeRD/Factur-X invoices the schema and rules
//! of their profile (spec 0027).
//!
//! The rules are the unchanged official Schematron artefacts in `rules/`,
//! evaluated with an XPath 3.1 engine (ADR 0002).

mod profile;
mod scenarios;
mod schematron;
mod xsd;

use std::cell::OnceCell;

use xee_xpath::Documents;

use crate::detect::{DetectError, Detection, Profile, Syntax, detect};
use crate::i18n::{Language, tr};
use crate::xml_view::decode;

use profile::FacturX;
use scenarios::Scenarios;
use schematron::{Files, RuleSet};

/// Version of the bundled CEN rules (`rules/cen-1.3.17/SOURCE.md`).
const CEN_VERSION: &str = "1.3.17";
const CEN_CII: &str =
    include_str!("../../rules/cen-1.3.17/EN16931-CII-validation-preprocessed.sch");
const CEN_UBL: &str =
    include_str!("../../rules/cen-1.3.17/EN16931-UBL-validation-preprocessed.sch");

/// Version of the bundled XRechnung rules (`rules/xrechnung-*/SOURCE.md`)
/// and the identifier version they check.
const XRECHNUNG_VERSION: &str = "3.0.2";
const XRECHNUNG_IDENTIFIER_VERSION: &str = "3.0";
const XRECHNUNG_CII: &str =
    include_str!("../../rules/xrechnung-2.6.0/cii/XRechnung-CII-validation.sch");
const XRECHNUNG_UBL: &str =
    include_str!("../../rules/xrechnung-2.6.0/ubl/XRechnung-UBL-validation.sch");
/// Included by both XRechnung rule sets as `../common.sch`.
const XRECHNUNG_COMMON: (&str, &str) = (
    "../common.sch",
    include_str!("../../rules/xrechnung-2.6.0/common.sch"),
);
/// KoSIT validator configuration for XRechnung (severity changes).
const KOSIT_SCENARIOS: &str = include_str!("../../rules/kosit-config-2026-08-31/scenarios.xml");

/// Severity of a finding, as flagged in the official rule artefacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Official flag `fatal`.
    Error,
    Warning,
    Information,
}

impl Severity {
    /// All severities, the most severe first.
    pub const ALL: [Self; 3] = [Self::Error, Self::Warning, Self::Information];
}

/// One violated rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Rule ID, e.g. `BR-CO-15`.
    pub rule_id: String,
    pub severity: Severity,
    /// Official text of the rule, whitespace normalised.
    pub message: String,
    /// 1-based XPath of the element the rule was checked on.
    pub location: String,
    /// BT/BG ids named in the message, in order of appearance.
    pub business_terms: Vec<String>,
}

/// Result of validating one invoice.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Validation {
    /// E.g. `EN 16931 CII 1.3.17`; empty if the rules do not apply.
    pub applied_rule_sets: Vec<String>,
    /// Why the EN 16931 rules were not applied, if so (profiles Minimum and
    /// Basic WL).
    pub not_applicable_reason: Option<String>,
    /// Set if the invoice declares an XRechnung version other than the
    /// bundled one, e.g. `Geprüft nach XRechnung 3.0.2, Rechnung erklärt
    /// Version 2.3`.
    pub version_note: Option<String>,
    pub findings: Vec<Finding>,
}

impl Validation {
    /// The severity of the most severe finding, `None` without findings.
    pub fn worst_severity(&self) -> Option<Severity> {
        Severity::ALL
            .into_iter()
            .find(|severity| self.findings.iter().any(|f| f.severity == *severity))
    }
}

/// Validates invoices. The rule sets are compiled on first use and kept.
///
/// Not `Send`: create one per thread that validates.
#[derive(Default)]
pub struct Validator {
    cii: OnceCell<RuleSet>,
    ubl: OnceCell<RuleSet>,
    xrechnung_cii: OnceCell<RuleSet>,
    xrechnung_ubl: OnceCell<RuleSet>,
    facturx: [OnceCell<RuleSet>; FacturX::ALL.len()],
    scenarios: OnceCell<Scenarios>,
}

impl Validator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates an invoice XML file. Fails only if the input is not a
    /// supported invoice (see [`detect`]); rule violations are findings. The
    /// app's own notes are in `language`; rule messages stay as published
    /// (spec 0017).
    pub fn validate(&self, xml: &[u8], language: Language) -> Result<Validation, DetectError> {
        self.check(xml, false, language)
    }

    /// Validates the invoice XML embedded in a PDF (ZUGFeRD/Factur-X): as
    /// [`Self::validate`], and an invoice with the plain EN 16931 identifier
    /// gets the rules of the Factur-X profile EN 16931 too (spec 0027).
    pub fn validate_embedded(
        &self,
        xml: &[u8],
        language: Language,
    ) -> Result<Validation, DetectError> {
        self.check(xml, true, language)
    }

    fn check(
        &self,
        xml: &[u8],
        embedded: bool,
        language: Language,
    ) -> Result<Validation, DetectError> {
        let detection = detect(xml)?;
        let text = decode(xml)?;
        let facturx = FacturX::of(detection.syntax, detection.profile, embedded);
        let (schema, schema_findings) = schema_check(&text, detection.syntax, facturx);
        let mut validation = Validation {
            applied_rule_sets: vec![
                tr(language, "XML schema {name}").replace("{name}", &schema.name()),
            ],
            not_applicable_reason: not_applicable_reason(detection.profile, language)
                .map(str::to_string),
            version_note: None,
            findings: schema_findings,
        };
        let with_en16931_rules = match facturx {
            Some(profile) => profile.with_en16931_rules(),
            None => validation.not_applicable_reason.is_none(),
        };
        if !with_en16931_rules && facturx.is_none() {
            return Ok(validation);
        }
        let mut documents = Documents::new();
        let Ok(document) = documents.add_string_without_uri(&text) else {
            // Detection parsed the invoice before; an engine that cannot load
            // it has nothing to report.
            return Ok(Validation::default());
        };

        if let Some(profile) = facturx {
            let rules = self.facturx[profile.index()].get_or_init(|| {
                // Rules the CEN rules check too are left to them, so that no
                // rule is reported twice.
                let skip_ids = if profile.with_en16931_rules() {
                    self.cen_rules(Syntax::Cii).rule_ids()
                } else {
                    Vec::new()
                };
                RuleSet::compile(
                    profile.schematron(),
                    &Files {
                        documents: &[profile.code_lists()],
                        skip_ids: &skip_ids,
                        ..Files::default()
                    },
                )
            });
            validation
                .findings
                .extend(
                    rules
                        .evaluate(&mut documents, document)
                        .into_iter()
                        .map(|mut finding| {
                            if finding.rule_id.is_empty() {
                                finding.rule_id = profile::UNNAMED_RULE_ID.to_string();
                            }
                            finding
                        }),
                );
            validation.applied_rule_sets.push(profile.rules_name());
        }
        if !with_en16931_rules {
            return Ok(validation);
        }
        validation.findings.extend(
            self.cen_rules(detection.syntax)
                .evaluate(&mut documents, document),
        );
        let syntax = match detection.syntax {
            Syntax::Cii => "CII",
            Syntax::Ubl => "UBL",
        };
        validation
            .applied_rule_sets
            .push(format!("EN 16931 {syntax} {CEN_VERSION}"));

        let Some(declared) = declared_xrechnung_version(&detection) else {
            return Ok(validation);
        };
        let xrechnung = match detection.syntax {
            Syntax::Cii => &self.xrechnung_cii,
            Syntax::Ubl => &self.xrechnung_ubl,
        };
        let xrechnung = xrechnung.get_or_init(|| {
            let rules = match detection.syntax {
                Syntax::Cii => XRECHNUNG_CII,
                Syntax::Ubl => XRECHNUNG_UBL,
            };
            RuleSet::compile(
                rules,
                &Files {
                    includes: &[XRECHNUNG_COMMON],
                    ..Files::default()
                },
            )
        });
        validation
            .findings
            .extend(xrechnung.evaluate(&mut documents, document));
        validation
            .applied_rule_sets
            .push(format!("XRechnung {XRECHNUNG_VERSION}"));
        if declared != XRECHNUNG_IDENTIFIER_VERSION {
            validation.version_note = Some(
                tr(
                    language,
                    "Checked against XRechnung {bundled}, invoice declares version {declared}",
                )
                .replace("{bundled}", XRECHNUNG_VERSION)
                .replace("{declared}", &declared),
            );
        }
        let scenarios = self
            .scenarios
            .get_or_init(|| Scenarios::compile(KOSIT_SCENARIOS));
        if let Some(levels) = scenarios.levels(&mut documents, document) {
            for finding in &mut validation.findings {
                if let Some(severity) = levels.get(&finding.rule_id) {
                    finding.severity = *severity;
                }
            }
        }
        Ok(validation)
    }
}

impl Validator {
    /// The CEN rules for a syntax, compiled on first use.
    fn cen_rules(&self, syntax: Syntax) -> &RuleSet {
        match syntax {
            Syntax::Cii => self
                .cii
                .get_or_init(|| RuleSet::compile(CEN_CII, &Files::default())),
            Syntax::Ubl => self
                .ubl
                .get_or_init(|| RuleSet::compile(CEN_UBL, &Files::default())),
        }
    }
}

/// Checks the invoice against its XML schema (spec 0023): the schema used and
/// its errors as findings, located at the element concerned.
fn schema_check(
    text: &str,
    syntax: Syntax,
    facturx: Option<FacturX>,
) -> (xsd::Schema, Vec<Finding>) {
    let Ok(doc) = roxmltree::Document::parse(text) else {
        // Detection parsed the invoice before.
        return (xsd::Schema::Cii, Vec::new());
    };
    let root = doc.root_element();
    let schema = match (syntax, facturx) {
        (_, Some(profile)) => xsd::Schema::FacturX(profile),
        (Syntax::Cii, None) => xsd::Schema::Cii,
        (Syntax::Ubl, None) if root.tag_name().name() == "CreditNote" => xsd::Schema::UblCreditNote,
        (Syntax::Ubl, None) => xsd::Schema::UblInvoice,
    };
    let prefixes = crate::fields::location_prefixes(syntax);
    let locations = crate::location::Locations::new(&doc, prefixes);
    let root_location = locations.of(root).to_string();
    let findings = xsd::check(text, schema)
        .into_iter()
        .map(|error| {
            let element = error.element.and_then(|positions| {
                let (first, rest) = positions.split_first()?;
                (*first == 0).then_some(())?;
                rest.iter().try_fold(root, |node, position| {
                    node.children()
                        .filter(roxmltree::Node::is_element)
                        .nth(*position)
                })
            });
            Finding {
                rule_id: SCHEMA_RULE_ID.to_string(),
                severity: Severity::Error,
                message: schema_message(&error.message, prefixes),
                location: element.map_or_else(
                    || root_location.clone(),
                    |element| locations.of(element).to_string(),
                ),
                business_terms: Vec::new(),
            }
        })
        .collect();
    (schema, findings)
}

/// The rule ID of schema findings.
pub const SCHEMA_RULE_ID: &str = "XSD";

/// libxml2's message with namespace URIs replaced by the usual prefixes,
/// e.g. `{urn:…:ReusableAggregateBusinessInformationEntity:100}Foo` →
/// `ram:Foo`, and whitespace normalised.
fn schema_message(message: &str, prefixes: &[(&str, &str)]) -> String {
    let mut text = message.split_whitespace().collect::<Vec<_>>().join(" ");
    for (prefix, namespace) in prefixes {
        text = text.replace(&format!("{{{namespace}}}"), &format!("{prefix}:"));
    }
    text
}

/// The XRechnung version an invoice declares in BT-24, if the XRechnung
/// rules apply to it (profile `xrechnung`, including the Extension and CVD).
fn declared_xrechnung_version(detection: &Detection) -> Option<String> {
    (detection.profile == Profile::XRechnung)
        .then(|| detection.profile_version.clone())
        .flatten()
}

fn not_applicable_reason(profile: Profile, language: Language) -> Option<&'static str> {
    match profile {
        Profile::Minimum => Some(tr(
            language,
            "Profile Minimum: not a complete invoice according to EN 16931",
        )),
        Profile::BasicWl => Some(tr(
            language,
            "Profile Basic WL: not a complete invoice according to EN 16931",
        )),
        _ => None,
    }
}

/// The BT/BG ids named in a message, e.g. `BT-112`, without duplicates.
fn business_terms(message: &str) -> Vec<String> {
    let mut terms: Vec<String> = Vec::new();
    for (index, _) in message.match_indices('B') {
        let rest = &message[index..];
        let preceded_by_word = message[..index]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric);
        if preceded_by_word || !(rest.starts_with("BT-") || rest.starts_with("BG-")) {
            continue;
        }
        let digits = rest[3..].chars().take_while(char::is_ascii_digit).count();
        if digits > 0 {
            let term = rest[..3 + digits].to_string();
            if !terms.contains(&term) {
                terms.push(term);
            }
        }
    }
    terms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worst_severity_is_the_most_severe_finding() {
        let finding = |severity| Finding {
            rule_id: String::new(),
            severity,
            message: String::new(),
            location: String::new(),
            business_terms: Vec::new(),
        };
        let validation = |findings| Validation {
            applied_rule_sets: Vec::new(),
            not_applicable_reason: None,
            version_note: None,
            findings,
        };
        assert_eq!(validation(Vec::new()).worst_severity(), None);
        assert_eq!(
            validation(vec![
                finding(Severity::Information),
                finding(Severity::Warning)
            ])
            .worst_severity(),
            Some(Severity::Warning)
        );
        assert_eq!(
            validation(vec![finding(Severity::Warning), finding(Severity::Error)]).worst_severity(),
            Some(Severity::Error)
        );
    }

    #[test]
    fn business_terms_in_order_without_duplicates() {
        assert_eq!(
            business_terms(
                "[BR-CO-15]-Invoice total amount with VAT (BT-112) = Invoice total amount without VAT (BT-109) + Invoice total VAT amount (BT-110). See BT-112."
            ),
            ["BT-112", "BT-109", "BT-110"]
        );
        assert_eq!(
            business_terms("[BR-16]-An Invoice shall have at least one Invoice line (BG-25)"),
            ["BG-25"]
        );
        assert_eq!(
            business_terms("no terms, not XBT-1 or BT-"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn official_rule_sets_compile_completely() {
        for rules in [CEN_CII, CEN_UBL] {
            let compiled = RuleSet::compile(rules, &Files::default());
            assert_eq!(compiled.uncompiled(), Vec::<&str>::new());
            assert!(compiled.assert_count() > 800);
        }
    }

    #[test]
    fn xrechnung_rule_sets_compile_completely() {
        for (rules, asserts) in [(XRECHNUNG_CII, 74), (XRECHNUNG_UBL, 77)] {
            let compiled = RuleSet::compile(
                rules,
                &Files {
                    includes: &[XRECHNUNG_COMMON],
                    ..Files::default()
                },
            );
            assert_eq!(compiled.uncompiled(), Vec::<&str>::new());
            assert_eq!(compiled.assert_count(), asserts);
        }
    }

    #[test]
    fn facturx_rule_sets_compile_completely() {
        for (profile, asserts) in FacturX::ALL.into_iter().zip([66, 337, 472, 627, 1464]) {
            let compiled = RuleSet::compile(
                profile.schematron(),
                &Files {
                    documents: &[profile.code_lists()],
                    ..Files::default()
                },
            );
            assert_eq!(compiled.uncompiled(), Vec::<&str>::new(), "{profile:?}");
            assert_eq!(compiled.assert_count(), asserts, "{profile:?}");
        }
    }
}
