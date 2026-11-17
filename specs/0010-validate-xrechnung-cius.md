---
id: 0010
title: Validate XRechnung invoices against the XRechnung CIUS rules
status: reviewed
created: 2026-11-16
depends_on: [0009]
---

## Context

Invoices to German public authorities must follow XRechnung, which adds
national rules (BR-DE-*, e.g. buyer reference, mandatory payment instructions)
on top of EN 16931. Users who send or receive XRechnung need to see
violations of these rules as well, judged the same way as by the official
KoSIT validator that authorities use.

## Requirement

For invoices with profile `xrechnung` (0003), in CII and UBL, the validation
result additionally contains the findings of the official XRechnung rules,
with the same finding structure as in 0009. The complete official rule set
is applied, including its rules for the XRechnung Extension (`BR-DEX-*`), for
XRechnung CVD and its temporary rules (`BR-TMP-*`, `BR-DE-TMP-*`); which of
them apply is decided by the specification identifier (BT-24), as in the
official rule set. For all other profiles these rules are not applied.

For XRechnung invoices the severities of EN 16931 findings follow the
official KoSIT validator configuration for XRechnung, which raises or lowers
the severity of a few EN 16931 rules (e.g. `CII-SR-465` becomes `error`,
`BR-CL-23` becomes `warning`, `CII-SR-475` becomes `information`).

If the invoice declares an XRechnung version different from the bundled rule
version, the rules of the bundled version are applied and the result states
both versions.

The rules are those of the official KoSIT `xrechnung-schematron` release
`v2.6.0` (XRechnung 3.0.2), unchanged, and the severity changes are those of
the KoSIT validator configuration for XRechnung 3.0.2, release
`2026-08-31`. Both ship with the app and are updated like the rules of 0009
([ADR 0002](../docs/adr/0002-validation-engine.md)).

## Acceptance criteria

- [ ] Given CII sample B ([fixtures](../docs/fixtures.md), XRechnung 3.0,
      buyer reference `04011000-12345-34`), when it is validated, then no
      finding with severity `error` is returned and the applied rule sets are
      `EN 16931 CII 1.3.17` and `XRechnung 3.0.2`.
- [ ] Given CII sample B without BT-10, then the findings contain `BR-DE-15`
      with severity `error`; its message is the official German text and
      mentions `BT-10`.
- [ ] Given CII sample B without payment instructions (BG-16, every
      `ram:SpecifiedTradeSettlementPaymentMeans` removed), then the findings
      contain `BR-DE-1` with severity `error`.
- [ ] Given CII sample A (profile `en16931`, no BT-10), then no finding with
      a rule ID from the XRechnung rule set is returned and the applied rule
      sets do not list XRechnung.
- [ ] Given UBL sample B, then no finding with severity `error` is returned
      and the applied rule sets are `EN 16931 UBL 1.3.17` and
      `XRechnung 3.0.2`. The UBL sample B fixture is created by this spec and
      confirmed once with the KoSIT validator.
- [ ] Given CII sample B with BT-24 declaring version 2.3
      (`urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_2.3`),
      then the result states that the bundled version was applied
      (`Geprüft nach XRechnung 3.0.2, Rechnung erklärt Version 2.3`), and the
      findings contain the warning `BR-DE-21`.
- [ ] Given the KoSIT test instance `cii-peppol-en16931-r040.xml` (XRechnung
      3.0; the KoSIT validator reports `CII-SR-465` for it, see spike 0001),
      then the finding `CII-SR-465` has severity
      `error`; given the same invoice with BT-24 set to
      `urn:cen.eu:en16931:2017`, the finding `CII-SR-465` has severity
      `warning`.
- [ ] For every invoice of the reference set of 0009 that the KoSIT validator
      assigns to an XRechnung scenario, the set of fired rule IDs from the
      XRechnung rule set (`BR-DE-*`, `BR-DEX-*`, `BR-TMP-*`, `BR-DE-TMP-*`)
      and the severities of all findings equal the expected result produced
      with the official KoSIT configuration for XRechnung 3.0.2.

## Out of scope

- Validating Leitweg-IDs against a register.
- XRechnung versions other than the bundled one (older versions are checked
  with the bundled rules, see requirement).
- Showing findings in the UI (0011).
