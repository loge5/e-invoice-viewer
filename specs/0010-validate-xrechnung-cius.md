---
id: 0010
title: Validate XRechnung invoices against the XRechnung CIUS rules
status: implemented
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
XRechnung CVD and its temporary rules (`BR-TMP-*`, `BR-DE-TMP-*`), and the
PEPPOL BIS rules (`PEPPOL-EN16931-R*`) that KoSIT merges into the released
rule set; which of them apply is decided by the specification identifier
(BT-24), as in the official rule set. For all other profiles these rules are not applied.

For XRechnung invoices the severities of EN 16931 findings follow the
official KoSIT validator configuration for XRechnung, which raises or lowers
the severity of a few EN 16931 rules (e.g. `CII-SR-465` becomes `error`,
`BR-CL-23` becomes `warning`, `CII-SR-475` becomes `information`).

If the invoice declares an XRechnung version different from the bundled rule
version, the rules of the bundled version are applied and the result states
both versions.

The rules are those of the official KoSIT `xrechnung-schematron` release
`v2.6.0` (XRechnung 3.0.2, release asset including the merged PEPPOL rules),
unchanged, and the severity changes are those of
the KoSIT validator configuration for XRechnung 3.0.2, release
`2026-08-31`. Both ship with the app and are updated like the rules of 0009
([ADR 0002](../docs/adr/0002-validation-engine.md)).

## Acceptance criteria

- [x] Given CII sample B ([fixtures](../docs/fixtures.md), XRechnung 3.0,
      buyer reference `04011000-12345-34`), when it is validated, then no
      finding with severity `error` is returned and the applied rule sets are
      `EN 16931 CII 1.3.17` and `XRechnung 3.0.2`.
- [x] Given CII sample B without BT-10, then the findings contain `BR-DE-15`
      with severity `error`; its message is the official German text and
      mentions `BT-10`.
- [x] Given CII sample B without payment instructions (BG-16, every
      `ram:SpecifiedTradeSettlementPaymentMeans` removed), then the findings
      contain `BR-DE-1` with severity `error`.
- [x] Given CII sample A (profile `en16931`, no BT-10), then no finding with
      a rule ID from the XRechnung rule set is returned and the applied rule
      sets do not list XRechnung.
- [x] Given UBL sample B, then no finding with severity `error` is returned
      and the applied rule sets are `EN 16931 UBL 1.3.17` and
      `XRechnung 3.0.2`. The UBL sample B fixture is created by this spec and
      confirmed once with the KoSIT validator.
- [x] Given CII sample B with BT-24 declaring version 2.3
      (`urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_2.3`),
      then the result states that the bundled version was applied
      (`Geprüft nach XRechnung 3.0.2, Rechnung erklärt Version 2.3`), and the
      findings contain the warning `BR-DE-21`.
- [x] Given the KoSIT test instance `cii-peppol-en16931-r040.xml` (XRechnung
      3.0; the KoSIT validator reports `CII-SR-465` for it, see spike 0001),
      then the finding `CII-SR-465` has severity
      `error`; given the same invoice with BT-24 set to
      `urn:cen.eu:en16931:2017`, the finding `CII-SR-465` has severity
      `warning`.
- [x] For every invoice of the reference set of 0009 that the KoSIT validator
      assigns to an XRechnung scenario, the set of fired rule IDs from the
      XRechnung rule set (`BR-DE-*`, `BR-DEX-*`, `BR-TMP-*`, `BR-DE-TMP-*`,
      `PEPPOL-EN16931-R*`)
      and the severities of all findings equal the expected result produced
      with the official KoSIT configuration for XRechnung 3.0.2.

## Out of scope

- Validating Leitweg-IDs against a register.
- XRechnung versions other than the bundled one (older versions are checked
  with the bundled rules, see requirement).
- Showing findings in the UI (0011).

## Progress

- **Done**:
  - `Validator::validate` also applies the XRechnung rules when BT-24 is
    an XRechnung identifier: profile `xrechnung`, or the 3.0 Extension and
    CVD identifiers, which detection (0003) reports as `unknown`.
  - Severities follow the matching KoSIT scenario. The result names
    `XRechnung 3.0.2`. A new field `version_note` reports a declared version
    other than 3.0.
  - The Schematron runner handles `include`, `let` (global and rule level),
    `xsl:function`, and `<name/>` / `<value-of>` in messages. It works around
    three further engine errors, see the ADR 0002 addendum of the same day.
  - Bundled unchanged:
    - `rules/xrechnung-2.6.0/` (release asset including the PEPPOL rules,
      Apache-2.0)
    - `rules/kosit-config-2026-08-31/scenarios.xml` (Apache-2.0)
  - Fixtures:
    - New `ubl/sample-b.xml`.
    - Sample B (CII and UBL) gets BT-23. The UBL fixture keeps one line
      note. Both are accepted by KoSIT, with only `BR-DE-TMP-32` reported
      as information.
    - Variants for BT-10, BG-16, version 2.3, Extension, CVD and r040 as
      EN 16931.
  - `scripts/kosit-reference.sh` also writes `expected-xrechnung.tsv`, from
    the configuration's own scenarios with the CEN 1.3.17 rules.
  - Test restructuring:
    - The reference comparisons of 0009 and 0010 share one validation pass
      in `tests/validate_reference.rs`.
    - The variants live in `tests/common/validation.rs`.
- **Verified** (2026-11-17):
  - `cargo test --workspace`: 102 passed, including 9 tests for this spec, 3
    reference tests and the new unit tests.
  - EN 16931 rule IDs match KoSIT on all 370 reference invoices.
  - XRechnung rule IDs and the severities of all findings match KoSIT on
    the 359 invoices that KoSIT assigns to an XRechnung scenario.
  - The reference test takes about 26–28 s.
  - Release timing tests of 0009 are green. Sample B takes about 50 ms
    (CII) and 90 ms (UBL), plus one-off compiling.
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
    warnings` and the architecture check are clean.
  - `cargo audit` reports the same two "unmaintained" notices as in 0009.
- **Next** — none; set to `implemented` by the author on 2026-11-17. The
  open point about detecting the Extension and CVD identifiers was fixed
  in 0003 the same day. Validation now relies on the detected profile.
