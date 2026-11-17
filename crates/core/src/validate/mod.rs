//! Validation against the official EN 16931 business rules (spec 0009).
//!
//! The rules are the unchanged CEN Schematron artefacts in `rules/`, evaluated
//! with an XPath 3.1 engine (ADR 0002).

mod schematron;

use std::cell::OnceCell;

use crate::detect::{DetectError, Profile, Syntax, detect};
use crate::xml_view::decode;

use schematron::RuleSet;

/// Version of the bundled CEN rules (`rules/cen-1.3.17/SOURCE.md`).
const CEN_VERSION: &str = "1.3.17";
const CEN_CII: &str =
    include_str!("../../rules/cen-1.3.17/EN16931-CII-validation-preprocessed.sch");
const CEN_UBL: &str =
    include_str!("../../rules/cen-1.3.17/EN16931-UBL-validation-preprocessed.sch");

/// Severity of a finding, as flagged in the official rule artefacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Official flag `fatal`.
    Error,
    Warning,
    Information,
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
    pub findings: Vec<Finding>,
}

/// Validates invoices. The rule sets are compiled on first use and kept.
///
/// Not `Send`: create one per thread that validates.
#[derive(Default)]
pub struct Validator {
    cii: OnceCell<RuleSet>,
    ubl: OnceCell<RuleSet>,
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
        let (rules, syntax) = match detection.syntax {
            Syntax::Cii => (self.cii.get_or_init(|| RuleSet::compile(CEN_CII)), "CII"),
            Syntax::Ubl => (self.ubl.get_or_init(|| RuleSet::compile(CEN_UBL)), "UBL"),
        };
        Ok(Validation {
            applied_rule_sets: vec![format!("EN 16931 {syntax} {CEN_VERSION}")],
            not_applicable_reason: None,
            findings: rules.evaluate(&text),
        })
    }
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
            let compiled = RuleSet::compile(rules);
            assert_eq!(compiled.uncompiled(), Vec::<&str>::new());
            assert!(compiled.assert_count() > 800);
        }
    }
}
