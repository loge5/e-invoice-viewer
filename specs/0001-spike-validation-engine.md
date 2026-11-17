---
id: 0001
title: Spike - validation engine for EN 16931 and XRechnung rules
status: implemented
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

- [x] An ADR `docs/adr/0002-validation-engine.md` exists and states the chosen
      approach, the alternatives considered, and the reasons.
- [x] The ADR evaluates at least these options:
      (a) evaluating the official Schematron assertions directly with an
      XPath engine available in the project's language;
      (b) re-implementing the rules natively;
      (c) delegating to an external validator as an optional fallback.
- [x] For the chosen approach, the ADR reports a prototype run of the official
      EN 16931 CII rules against at least 10 CII invoices, among them at least
      3 that violate rules. Per invoice it compares the fired EN 16931 rule IDs
      (XRechnung `BR-DE-*` IDs excluded) with the reference result of the
      official KoSIT validator: identical yes/no, and the list of differences.
- [x] The ADR states coverage for each of the three rule sets EN 16931 CII,
      EN 16931 UBL and XRechnung CIUS (CII): the number of asserts in the
      official artefact, how many can be evaluated with the chosen approach
      (absolute and percentage), and the ones that cannot, with the reason.
      For CII this is measured with the prototype; for UBL and XRechnung a
      static analysis of the artefacts is sufficient.
- [x] The ADR states the measured time to validate one typical invoice (1 to
      5 line items) on the author's laptop (Intel Core Ultra 7 155H, 30 GB
      RAM), and whether it stays below 1 s.
- [x] The ADR states the licence of every rule artefact and engine involved,
      and confirms compatibility with GPL-3.0-or-later.
- [x] The ADR lists the follow-up consequences for specs 0009 and 0010
      (e.g. scope, rule versions to bundle, how rule updates are shipped).

## Out of scope

- Production-quality implementation of validation (that is 0009 and 0010).
- Prototype runs for UBL and XRechnung (covered by static analysis only, see
  the coverage criterion).
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

## Progress

- **Done** — ADR [0002](../docs/adr/0002-validation-engine.md) written
  (accepted by the author on 2026-11-16): option (a), the official Schematron asserts evaluated
  with `xee-xpath` 0.1.5 (XPath 3.1, MIT). Throwaway prototype in the session
  scratchpad, not merged. Rule versions: CEN `validation-1.3.17`, KoSIT
  `xrechnung-schematron` `v2.6.0`. Reference: KoSIT validator 1.6.3.
- **Verified** — 2026-11-16 on the reference machine:
  - EN 16931 CII: 191 of 191 invoices (48 with findings) give the same rule
    IDs as KoSIT.
  - Extra runs beyond the criteria: EN 16931 UBL 287/287, XRechnung CII
    170/170.
  - Coverage: 810/810 asserts for CII, 982/982 for UBL, 52/52 for XRechnung
    CII.
  - Typical invoice: 38–150 ms, below 1 s.
  - Large invoices grow superlinearly: 500 lines take 2.7 s, 3000 lines
    take 37 s. Levers for this are listed in the ADR.
  - Licences checked: CEN rules are EUPL-1.2, not Apache 2.0; they are
    GPL-3.0-compatible.
  - Side finding: `sample_cii_with_lines(3000)` produces inconsistent totals
    (`BR-CO-10` and `BR-S-08` fire, in KoSIT as well).
- **Next** — none for the spike; set to `implemented` by the author on 2026-11-17. The NEEDS CLARIFICATION in 0009 and 0010
  are resolved following the ADR; both specs still need their review.
