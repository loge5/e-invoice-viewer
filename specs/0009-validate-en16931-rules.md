---
id: 0009
title: Validate an invoice against the EN 16931 business rules
status: draft
created: 2026-11-16
depends_on: [0001, 0003, 0004]
---

## Context

Users want to know whether an invoice is formally correct before booking or
forwarding it, and exactly which value is wrong. The European standard
EN 16931 defines the business rules (e.g. BR-01, BR-CO-15) that every
compliant invoice must satisfy, in CII and UBL. The findings must be precise
enough to be shown next to the affected field (0011).

## Requirement

Given an invoice XML (CII or UBL) and its detected profile (0003), the system
evaluates the official EN 16931 business rules and returns a list of findings.
Each finding carries the rule ID, the severity, the official message text, the
location of the affected element and, where possible, the BT/BG number of the
affected field so it can be matched with the field model. Validation never
reaches out to the network and never crashes on malformed content.

Rules are applied for the profiles `basic`, `en16931`, `extended`, `xrechnung`
and `unknown`. For `minimum` and `basic-wl` no EN 16931 findings are produced;
instead the result states that EN 16931 does not apply to this profile.

The rule set version bundled with the app:
[NEEDS CLARIFICATION: exact CEN artefact version to bundle and how it is
updated, to be decided by ADR 0002 from spike 0001].

## Contracts (optional)

Validation result:

| Field | Meaning |
|---|---|
| applied rule sets | e.g. `EN 16931 CII 1.3.x`, empty if not applicable |
| not applicable reason | e.g. `Profil Minimum: keine vollständige Rechnung nach EN 16931` |
| findings | list, see below |

Finding: `rule_id` (e.g. `BR-CO-15`), `severity` (`error` or `warning`, as
flagged in the official artefacts), `message` (official text), `location`
(XPath of the affected element, or of the closest existing parent if the
element is missing), `business_terms` (BT/BG ids mentioned by the rule that
are present in the field model, may be empty).

## Acceptance criteria

- [ ] Given CII sample A ([fixtures](../docs/fixtures.md)), when it is
      validated, then the result has zero findings with severity
      `error`, and the applied rule sets name EN 16931 CII.
- [ ] Given sample A with BT-24 removed, then exactly one finding with
      `rule_id` `BR-01`, severity `error`, a message mentioning `BT-24`, and a
      location that selects an existing element of the document (never the
      missing BT-24 element) is returned.
- [ ] Given sample A with BT-112 changed to `1300.00` while BT-109 is
      `1050.00` and BT-110 `199.50`, then a finding `BR-CO-15` with severity
      `error` is returned whose `business_terms` contain `BT-112`, and whose
      location selects the BT-112 element.
- [ ] Given UBL sample A with BT-1 removed, then a finding
      `BR-02` with severity `error` is returned.
- [ ] Given a CII invoice with profile `minimum`, then no findings are
      returned and the not-applicable reason is set.
- [ ] For every invoice of the reference set chosen in spike 0001, the set of
      fired rule IDs equals the reference result of the official KoSIT
      validator.
- [ ] Validating a typical invoice (1 to 5 lines) takes less than 1 s;
      validating sample L takes less than 5 s (reference machine, see
      [requirements](../docs/requirements.md)).

## Out of scope

- XRechnung CIUS rules (0010), ZUGFeRD-specific profile rules, XML schema
  (XSD) validation, PDF/A checks.
- Showing findings in the UI (0011).
