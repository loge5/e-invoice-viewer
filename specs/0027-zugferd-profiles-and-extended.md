---
id: 0027
title: ZUGFeRD/Factur-X - profile schemas and profile rules
status: draft
created: 2026-11-19
depends_on: [0003, 0009, 0023, 0024]
---

## Context

ZUGFeRD 2.x / Factur-X defines five profiles (Minimum, Basic WL, Basic,
EN 16931, Extended), each with its own XML schema and Schematron. The app
applies the EN 16931 rules (and XRechnung) and, with 0023, the general CII
schema, but not the profile-specific checks; content of the Extended profile
beyond EN 16931 is not shown. `docs/requirements.md` (decision 5) names the
ZUGFeRD profile rules as a later extension. For 1.0 the author decided on
2026-11-19 to include them, using the official FeRD package, which the author
downloads (FeRD asks for contact details).

**Package** (provided by the author 2026-11-20): `ZUGFeRD_2.5.2_EN`,
Factur-X 1.09.2, released 2026-08-04, effective 2026-09-01. Per profile
(Minimum, Basic WL, Basic, EN 16931, Extended) it contains an XSD set based on
CII D22B, one Schematron (`queryBinding="xslt2"`) with its code list file
`FACTUR-X_<profile>_codedb.xml`, read by the Schematron through
`document(...)`, and an XSLT generated from it; also the full CII D22B schema
and sample invoices (Minimum 2, Basic WL 2, Basic 3, EN 16931 25, Extended
25, XRechnung 4 XML, mostly with PDF).

Facts from the package that shape the requirement:

- Profile identifiers (BT-24) of the samples: `urn:factur-x.eu:1p0:minimum`,
  `urn:factur-x.eu:1p0:basicwl`,
  `urn:cen.eu:en16931:2017#compliant#urn:factur-x.eu:1p0:basic`,
  `urn:cen.eu:en16931:2017` (EN 16931, the plain CEN identifier) and
  `urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended`. The
  identifiers carry no ZUGFeRD version.
- Factur-X is based on CII D22B, which is backward compatible with D16B but
  allows more than one preceding invoice (BG-3); the D16B check of 0023 would
  report such a Factur-X invoice wrongly.
- The Schematron files repeat many EN 16931 rules under the same IDs (e.g.
  EN 16931 profile: 193 of 433 IDs are also in CEN 1.3.17), use `report` as
  well as `assert` (the bundled CEN and XRechnung files use only `assert`),
  and have assertions without `id` (194 in EN 16931, 521 in Extended),
  mostly profile restrictions such as `Attribute @listID marked as not used
  in the given context`. Flags are `warning` or none.
- Size: 66 (Minimum) to 1464 (Extended) assertions.

**Licence** (checked 2026-11-20): FeRD's page *Disclaimer and rights of use
ZUGFeRD* puts the technical artefacts, "Schemata and Schematron", under the
Apache License 2.0, which is compatible with GPL-3.0. The code list files are
read by the Schematron and lie next to it; they are treated as part of the
Schematron artefacts. The page names no licence for the sample invoices;
they are not committed (author's decision 2026-11-20), see Notes. The XSLT
is not needed and not bundled.

## Requirement

- **When the profile rules apply** ("Factur-X invoice"): a CII invoice whose
  profile (BT-24, detected by 0003) is Minimum, Basic WL, Basic or Extended,
  or EN 16931 (the plain CEN identifier) when the invoice is embedded in a
  PDF. A plain CII XML file with the CEN identifier, every UBL invoice and
  every XRechnung invoice (0010) are checked as before.
- **Profile schema**: a Factur-X invoice is checked against the XSD of its
  profile instead of the general CII D16B schema of 0023; findings carry the
  rule ID `XSD` as in 0023.
- **Profile rules**: in addition, the Schematron of its profile, with its
  code list file, is evaluated:
  - findings carry the rule ID from the file; an assertion or report
    without `id` gets the rule ID `FX` and keeps its message;
  - flag `warning` → warning, no flag → error;
  - where the CEN rules are applied too (Basic, EN 16931, Extended), profile
    rules whose ID is also a rule ID of the bundled CEN rules are not
    evaluated: the CEN rules are the reference for them, and a rule is never
    reported twice.
- **Minimum and Basic WL**: are checked against their profile schema and
  rules; the CEN rules stay not applicable to them, and the note that they
  are not complete invoices (0009) stays, without claiming that no rules
  were applied.
- **Summary**: lists what was checked in the existing form, e.g. `XML schema
  Factur-X 1.09.2 EN 16931`, `Factur-X 1.09.2 (ZUGFeRD 2.5.2) EN 16931`,
  `EN 16931 CII 1.3.17`.
- **Texts**: new texts of the app are translated into all 24 languages
  (the catalogue check of 0030 enforces it).
- **Bundling**: the five XSD sets, Schematron files and code list files are
  copied unchanged to `crates/core/rules/facturx-1.09.2/` with `SOURCE.md`
  (source, version, file list with checksums) and the Apache 2.0 licence
  text; the README lists them under the bundled third-party material. The
  code list file is read only from the bundled files, never from disk or the
  network.

## Acceptance criteria

- [ ] Given each FeRD sample invoice of the profiles Minimum to Extended (XML
      and, where present, PDF), when it is validated, then its profile
      schema and rules are applied and the profile check reports no error
      (local test against the package, see Notes).
- [ ] Given the FeRD XRechnung samples, then no profile rules are applied
      and the XRechnung rules are applied as before (local test).
- [ ] Given a committed synthetic EN 16931-profile PDF invoice with an
      element that only the Extended profile allows, then the profile schema
      check reports it with rule ID `XSD`.
- [ ] Given a committed synthetic Minimum invoice in a PDF, then the profile
      schema and rules are applied, the summary names them together with
      the note of 0009, and a violated Minimum rule is shown as a finding
      with its rule ID.
- [ ] Given a committed synthetic Basic invoice with two preceding invoices
      (BG-3) in a PDF, then no schema finding is reported (D22B), while the
      same XML as a plain XML file with the CEN identifier still gets the
      D16B finding of 0023.
- [ ] Given a violated assertion without `id` (e.g. an attribute marked as
      not used), then the finding has rule ID `FX` and the message from the
      Schematron.
- [ ] Given an EN 16931-profile PDF invoice that violates a rule present in
      both the CEN and the profile rules (e.g. `BR-CO-15`), then exactly one
      finding with that rule ID is reported.
- [ ] A plain CII XML invoice with the CEN identifier, UBL invoices and the
      XRechnung and KoSIT tests give the same results as before.
- [ ] Validating the largest Extended sample takes no longer than 2 s in a
      release build on the author's machine, including compiling the rules
      on first use.
- [ ] The bundled files are unchanged copies (checksums in `SOURCE.md`);
      licence notes are in the README and next to the files.

## Out of scope

- Showing elements that are not mapped to a business term (`Other data`):
  split off into spec 0031 at the review (2026-11-20).
- ZUGFeRD 1.0 (different syntax).
- PDF/A conformance and the XMP metadata of the PDF (decision 7 of the
  requirements); checking that the XMP profile matches BT-24.
- A version note for other ZUGFeRD versions: the identifiers carry no
  version.

## Notes

- Sample invoices: not committed. A test reads them from the directory in
  `FERD_PACKAGE` (the unpacked package) and is skipped without it; the CI
  does not run it. Committed tests use synthetic invoices.
- Engine work: the Schematron engine needs `report`, `document()` for the
  code list file and assertions without `id`. Apart from `document()` the
  files use only XPath functions (`count`, `decimal`, `normalize-space`,
  `round`, `sum`, `tokenize`, `matches`, …).
- `Validator::validate` needs to know whether the XML came from a PDF; this
  changes its signature (core API, callers in the app and the tests).
- The rule sets are compiled lazily on first use, like the others.
- Review on 2026-11-20 (Claude):
  - the condition uses the detected profile instead of matching BT-24 text;
  - doubled rules are not evaluated instead of de-duplicated by location
    (FeRD and CEN check some rules on different elements);
  - Minimum and Basic WL keep the CEN rules not applicable;
  - summary in the existing list form; translation of new texts;
  - XRechnung samples as a regression check; the PDF flag for `validate`;
  - `Other data` split off into 0031: it applies to every invoice, not
    only to Factur-X, and is tested independently.

## Test list (proposed)

- T1 `ferd_samples` (local, `FERD_PACKAGE`): every sample of the profiles
  Minimum to Extended, XML and PDF: profile schema and rules applied, no
  error from the profile check (AC1); XRechnung samples without profile
  rules (AC2).
- T2 `extended_element_in_en16931_profile` (synthetic PDF) (AC3).
- T3 `minimum_profile_rules` (synthetic PDF, valid and with one violation;
  summary and note) (AC4).
- T4 `two_preceding_invoices_d22b` (synthetic Basic PDF; plain XML control)
  (AC5).
- T5 `report_and_assertion_without_id` (engine unit tests: `report`,
  missing `id`, `document()` from bundled files) and one invoice (AC6).
- T6 `rule_in_cen_and_profile_reported_once` (AC7).
- T7 existing KoSIT, CEN, XRechnung and schema tests unchanged (AC8).
- T8 `bundled_facturx_files_unchanged` (checksums from `SOURCE.md`) and a
  packaging test for README and licence (AC10).
- P1 manual: largest Extended sample, release build, time from opening to
  findings shown (AC9).
