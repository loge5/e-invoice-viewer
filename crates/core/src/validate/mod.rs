//! Validation against the official EN 16931 business rules (spec 0009) and,
//! for XRechnung invoices, the XRechnung CIUS rules (spec 0010).
//!
//! The rules are the unchanged official Schematron artefacts in `rules/`,
//! evaluated with an XPath 3.1 engine (ADR 0002).

mod scenarios;
mod schematron;

use std::cell::OnceCell;

use xee_xpath::Documents;

use crate::detect::{DetectError, Detection, Profile, Syntax, detect};
use crate::xml_view::decode;

use scenarios::Scenarios;
use schematron::RuleSet;

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
    /// Why no rules were applied, if so.
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
    scenarios: OnceCell<Scenarios>,
}

impl Validator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates an invoice XML. Fails only if the input is not a supported
    /// invoice (see [`detect`]); rule violations are findings.
    pub fn validate(&self, xml: &[u8]) -> Result<Validation, DetectError> {
        let detection = detect(xml)?;
        if let Some(reason) = not_applicable_reason(detection.profile) {
            return Ok(Validation {
                not_applicable_reason: Some(reason.to_string()),
                ..Validation::default()
            });
        }
        let text = decode(xml)?;
        let mut documents = Documents::new();
        let Ok(document) = documents.add_string_without_uri(&text) else {
            // Detection parsed the invoice before; an engine that cannot load
            // it has nothing to report.
            return Ok(Validation::default());
        };

        let (rules, syntax) = match detection.syntax {
            Syntax::Cii => (
                self.cii.get_or_init(|| RuleSet::compile(CEN_CII, &[])),
                "CII",
            ),
            Syntax::Ubl => (
                self.ubl.get_or_init(|| RuleSet::compile(CEN_UBL, &[])),
                "UBL",
            ),
        };
        let mut validation = Validation {
            applied_rule_sets: vec![format!("EN 16931 {syntax} {CEN_VERSION}")],
            not_applicable_reason: None,
            version_note: None,
            findings: rules.evaluate(&mut documents, document),
        };

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
            RuleSet::compile(rules, &[XRECHNUNG_COMMON])
        });
        validation
            .findings
            .extend(xrechnung.evaluate(&mut documents, document));
        validation
            .applied_rule_sets
            .push(format!("XRechnung {XRECHNUNG_VERSION}"));
        if declared != XRECHNUNG_IDENTIFIER_VERSION {
            validation.version_note = Some(format!(
                "Geprüft nach XRechnung {XRECHNUNG_VERSION}, Rechnung erklärt Version {declared}"
            ));
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

/// The XRechnung version an invoice declares in BT-24, if the XRechnung
/// rules apply to it (profile `xrechnung`, including the Extension and CVD).
fn declared_xrechnung_version(detection: &Detection) -> Option<String> {
    (detection.profile == Profile::XRechnung)
        .then(|| detection.profile_version.clone())
        .flatten()
}

fn not_applicable_reason(profile: Profile) -> Option<&'static str> {
    match profile {
        Profile::Minimum => Some("Profil Minimum: keine vollständige Rechnung nach EN 16931"),
        Profile::BasicWl => Some("Profil Basic WL: keine vollständige Rechnung nach EN 16931"),
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
            let compiled = RuleSet::compile(rules, &[]);
            assert_eq!(compiled.uncompiled(), Vec::<&str>::new());
            assert!(compiled.assert_count() > 800);
        }
    }

    #[test]
    fn xrechnung_rule_sets_compile_completely() {
        for (rules, asserts) in [(XRECHNUNG_CII, 74), (XRECHNUNG_UBL, 77)] {
            let compiled = RuleSet::compile(rules, &[XRECHNUNG_COMMON]);
            assert_eq!(compiled.uncompiled(), Vec::<&str>::new());
            assert_eq!(compiled.assert_count(), asserts);
        }
    }
}
