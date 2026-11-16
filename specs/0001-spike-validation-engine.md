---
id: 0001
title: Spike - validation engine for EN 16931 and XRechnung rules
status: draft
created: 2026-11-16
depends_on: []
---

## Context

The viewer must show validation findings for invoices (see
[requirements](../docs/requirements.md), decisions 5 and 6). The official rules
are published by CEN (EN 16931) and KoSIT (XRechnung CIUS) as Schematron files
whose assertions are XPath 2.0 expressions, normally executed via XSLT 2.0 in
Java (Saxon). The viewer has to be a self-contained binary without Java and
without network access.

It is unknown whether these rules can be evaluated reliably without a Java
runtime. This spike answers that question before the validation specs (0009,
0010) are sized and reviewed.

## Requirement

Investigate how the official EN 16931 rules (CII and UBL) and the XRechnung
CIUS rules can be evaluated inside a self-contained binary, and record a
decision with evidence. The spike produces knowledge and a decision record,
not production code. Throwaway prototype code is allowed and is not merged.

## Acceptance criteria

- [ ] An ADR `docs/adr/0002-validation-engine.md` exists and states the chosen
      approach, the alternatives considered, and the reasons.
- [ ] The ADR evaluates at least these options:
      (a) evaluating the official Schematron assertions directly with an
      XPath engine available in the project's language;
      (b) re-implementing the rules natively;
      (c) delegating to an external validator as an optional fallback.
- [ ] For the chosen approach, the ADR reports a prototype run of the official
      EN 16931 CII rules against at least 10 invoices, among them at least 3
      that violate rules, and compares the result per invoice with the
      reference result of the official KoSIT validator (same rule IDs fired:
      yes/no, list of differences).
- [ ] The ADR states coverage: how many of the official EN 16931 CII asserts
      (absolute number and percentage) could be evaluated without errors in
      the prototype, and lists the ones that could not, with the reason.
- [ ] The ADR states the measured time to validate one typical invoice (1 to
      5 line items) on the author's laptop, and whether it stays below 1 s.
- [ ] The ADR states the licence of every rule artefact and engine involved,
      and confirms compatibility with GPL-3.0-or-later.
- [ ] The ADR lists the follow-up consequences for specs 0009 and 0010
      (e.g. scope, rule versions to bundle, how rule updates are shipped).

## Out of scope

- Production-quality implementation of validation (that is 0009 and 0010).
- ZUGFeRD-specific profile rules, PDF/A conformance checks.
- Any UI work.

## Notes

- The reference results may be produced once at development time with the
  KoSIT validator (Java). This does not violate decision 6, since it is a
  development tool and not a runtime dependency.
- Candidate starting points: the CEN `eInvoicing-EN16931` validation
  artefacts, the KoSIT `xrechnung-schematron` and `xrechnung-testsuite`
  repositories (Apache 2.0).
- Time box: one focused session. If no conclusion is reached, the ADR
  documents the findings so far and the open questions.
