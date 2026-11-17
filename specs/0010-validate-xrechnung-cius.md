---
id: 0010
title: Validate XRechnung invoices against the XRechnung CIUS rules
status: draft
created: 2026-11-16
depends_on: [0009]
---

## Context

Invoices to German public authorities must follow XRechnung, which adds
national rules (BR-DE-*, e.g. Leitweg-ID as buyer reference, mandatory payment
instructions) on top of EN 16931. Users who send or receive XRechnung need to
see violations of these rules as well.

## Requirement

For invoices with profile `xrechnung` (0003), the validation result
additionally contains the findings of the official XRechnung CIUS rules, with
the same finding structure as in 0009. For all other profiles these rules are
not applied. If the invoice declares an XRechnung version different from the
bundled rule version, the result states which version was used.

The bundled XRechnung rule version is KoSIT `xrechnung-schematron`
`v2.6.0` (XRechnung 3.0.2, Apache-2.0), stored unchanged with its licence
text and embedded into the binary; updates work as in 0009
([ADR 0002](../docs/adr/0002-validation-engine.md)).

## Acceptance criteria

- [ ] Given CII sample B ([fixtures](../docs/fixtures.md), XRechnung 3.0,
      buyer reference `04011000-12345-34`), when it is validated, then
      no finding with severity `error` is returned and the applied rule sets
      list both EN 16931 and XRechnung.
- [ ] Given sample B without BT-10, then a finding `BR-DE-15` with
      severity `error` is returned; its message is the official German text
      and mentions `BT-10`.
- [ ] Given sample B without payment instructions (BG-16), then a
      finding `BR-DE-1` with severity `error` is returned.
- [ ] Given CII sample A (profile `en16931`, no BT-10), then no
      `BR-DE-*` finding is returned.
- [ ] Given an XRechnung invoice declaring version `2.3`, then the result
      states that the rules of the bundled version were applied (e.g.
      `Geprüft nach XRechnung 3.0.2, Rechnung erklärt Version 2.3`).
- [ ] For every XRechnung invoice in the KoSIT test suite set chosen in spike
      0001, the set of fired `BR-DE-*` rule IDs equals the reference result of
      the official KoSIT validator.

## Out of scope

- Extension rules of the XRechnung Extension profile.
- Validating Leitweg-IDs against a register.
