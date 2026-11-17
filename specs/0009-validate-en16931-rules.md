---
id: 0009
title: Validate an invoice against the EN 16931 business rules
status: implemented
created: 2026-11-16
depends_on: [0001, 0003, 0004]
---

## Context

Users want to know whether an invoice is formally correct before booking or
forwarding it, and exactly which value is wrong. The European standard
EN 16931 defines the business rules (e.g. BR-01, BR-CO-15) that every
compliant invoice must satisfy, in CII and UBL. The findings must be precise
enough to be shown next to the affected fields (0011).

## Requirement

Given an invoice XML (CII or UBL) and its detected profile (0003), the system
evaluates the official EN 16931 business rules and returns a list of findings.
Each finding carries the rule ID, the severity, the official message text, the
location of the element the rule was checked on, and the BT/BG numbers the
rule refers to, so that it can be matched with the field model. Validation
never reaches out to the network and never crashes; a rule that cannot be
evaluated for a given invoice counts as violated, so that no problem is hidden.

Rules are applied for the profiles `basic`, `en16931`, `extended`, `xrechnung`
and `unknown`. For `minimum` and `basic-wl` no EN 16931 findings are produced;
instead the result states that EN 16931 does not apply to this profile.

The rules are those of the official CEN validation artefacts, release
`1.3.17`, unchanged. They ship with the app; there is no download at run
time. The result names the applied rule set and its version. How the rules
are bundled and updated is described in
[ADR 0002](../docs/adr/0002-validation-engine.md).

## Contracts (optional)

Validation result:

| Field | Meaning |
|---|---|
| applied rule sets | e.g. `EN 16931 CII 1.3.17`; empty if not applicable |
| not applicable reason | e.g. `Profil Minimum: keine vollständige Rechnung nach EN 16931` |
| findings | list, see below |

Finding:

| Field | Meaning |
|---|---|
| `rule_id` | e.g. `BR-CO-15` |
| `severity` | `error` (official flag `fatal`), `warning` or `information`, as flagged in the official artefacts |
| `message` | official text of the rule, whitespace normalised |
| `location` | 1-based XPath of the element the rule was checked on; always an element that exists in the document |
| `business_terms` | BT/BG ids named in the official message (e.g. `BT-112`, `BT-109`, `BT-110`), in order of appearance; may be empty |

## Acceptance criteria

- [x] Given CII sample A ([fixtures](../docs/fixtures.md)), when it is
      validated, then the result has zero findings with severity `error`, and
      the applied rule sets are `EN 16931 CII 1.3.17`.
- [x] Given CII sample A with the BT-24 element
      (`ram:GuidelineSpecifiedDocumentContextParameter/ram:ID`) removed, then
      the findings contain `BR-01` with severity `error`, a message mentioning
      `BT-24`, and a location that selects the root element
      `rsm:CrossIndustryInvoice`.
- [x] Given CII sample A with BT-112 changed to `1300.00` (BT-109 stays
      `1050.00`, BT-110 `199.50`), then the findings contain `BR-CO-15` with
      severity `error` and `business_terms` `BT-112`, `BT-109`, `BT-110`.
- [x] Given UBL sample A, then the result has zero findings with severity
      `error` and the applied rule sets are `EN 16931 UBL 1.3.17`. The UBL
      sample A fixture is created by this spec and confirmed once with the
      KoSIT validator, as [fixtures](../docs/fixtures.md) requires.
- [x] Given UBL sample A with BT-1 (`cbc:ID` of the invoice) removed, then the
      findings contain `BR-02` with severity `error`.
- [x] Given CII sample A with BT-24 set to `urn:factur-x.eu:1p0:minimum`, then
      no findings are returned, the applied rule sets are empty, and the
      not-applicable reason is set.
- [x] Given the KoSIT test instance `ubl-inv-peppol-en16931-r053-2.xml`
      (two VAT totals in the document currency, which makes the rule's
      arithmetic impossible), then the findings contain `BR-CO-15`.
- [x] For every invoice of the reference set, the set of fired rule IDs equals
      the expected result. The reference set consists of all KoSIT
      `xrechnung-schematron` v2.6.0 test instances (CII and UBL, Apache-2.0)
      that are valid against the XML schema, plus samples A and B and the
      variants of this spec. The expected results are produced once with the
      KoSIT validator 1.6.3 by a development script that is part of the
      repository, and are committed. Running the comparison needs no Java.
- [x] Validating CII sample A takes less than 1 s; validating sample L
      (500 lines) takes less than 5 s (release build, reference machine, see
      [requirements](../docs/requirements.md)).

## Out of scope

- XRechnung CIUS rules (0010), ZUGFeRD-specific profile rules, XML schema
  (XSD) validation, PDF/A checks.
- Showing findings in the UI and running validation without blocking it
  (0011).
- Automatic download or update of rule sets.

## Progress

- **Done** — core module `validate`:
  - `Validator::validate` returns applied rule sets, a not-applicable reason
    (`minimum`, `basic-wl`) and findings (`rule_id`, `severity`, `message`,
    `location`, `business_terms`).
  - `validate/schematron.rs` is a minimal Schematron runner on `xee-xpath`,
    as described in ADR 0002: first-match per pattern, a dynamic error counts
    as violated, absolute tests are evaluated once per document, and engine
    panics are guarded.
  - The CEN 1.3.17 rules are embedded unchanged from
    `crates/core/rules/cen-1.3.17/`, together with the EUPL licence and
    `SOURCE.md`.
  - New fixtures:
    - `ubl/sample-a.xml`, confirmed by KoSIT without any message.
    - 353 XSD-valid KoSIT instances under `tests/fixtures/kosit/`
      (Apache-2.0, about 5 MB).
    - `expected-en16931.tsv` with 363 invoices, 147 of them with findings.
  - `scripts/kosit-reference.sh` regenerates the expected results.
  - New dependencies: `xee-xpath` =0.1.5 and `xot` =0.31.2 (MIT).
  - Dev profile: dependencies are built with `opt-level = 2`.
- **Verified** — 2026-11-17:
  - `cargo test --workspace`: 83 passed, among them 13 for this spec and 3
    unit tests.
  - The reference test matches KoSIT on all 362 schema-valid invoices. It
    takes about 19 s in parallel, because a UBL invoice needs about 150–200 ms
    in the engine.
  - Release: the timing tests for sample A (< 1 s; validation about 37 ms,
    plus a one-off 150 ms compile) and sample L (< 5 s) are green.
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
    warnings` and the architecture check are clean.
  - `cargo audit` (advisory) finds no vulnerability. It reports two
    "unmaintained" notices, both transitive through xee/xot: `json` 0.12.4
    and `proc-macro-error` 0.4.12. `cargo deny` is not installed.
- **Next** — none; set to `implemented` by the author on 2026-11-17.
