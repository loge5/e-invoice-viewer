---
id: 0027
title: ZUGFeRD/Factur-X - profile schemas, profile rules, Extended content
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

- **When the profile rules apply** ("Factur-X invoice"): a CII invoice that
  is embedded in a PDF (ZUGFeRD/Factur-X, 0003), or whose BT-24 contains
  `urn:factur-x.eu:1p0:`. The profile follows from BT-24 (the plain CEN
  identifier is the EN 16931 profile). A plain CII XML file with the CEN
  identifier and every UBL invoice are checked as before; XRechnung
  identifiers keep the XRechnung rules (0010) and get no profile rules.
- **Profile schema**: a Factur-X invoice is checked against the XSD of its
  profile instead of the general CII D16B schema of 0023; findings carry the
  rule ID `XSD` as in 0023.
- **Profile rules**: in addition, the Schematron of its profile, with its
  code list file, is evaluated. Findings carry the rule ID from the file;
  an assertion without `id` gets the rule ID `FX` and keeps its message.
  Severities: flag `warning` → warning, no flag → error. A finding that the
  CEN rules report with the same rule ID at the same element is shown once.
- **Minimum and Basic WL**: are checked against their profile schema and
  rules; the note that they are not complete invoices (0009) stays, but the
  validation is no longer "not applicable".
- **Summary**: names what was checked, e.g. `Checked against Factur-X
  1.09.2 EN 16931 (XML schema and rules) and EN 16931 CII 1.3.17`; the
  ZUGFeRD version 2.5.2 is named in the details/about text.
- **Other data**: elements of the invoice that are not mapped to a business
  term (Extended profile, XRechnung extension, anything else the schema
  allows) are not hidden: a section `Weitere Angaben` / `Other data` per place
  (document, each party, each line) lists them with element name and value,
  in document order. This applies to every CII and UBL invoice, not only to
  Factur-X.
- **Bundling**: the five XSD sets, Schematron files and code list files are
  copied unchanged to `crates/core/rules/facturx-1.09.2/` with `SOURCE.md`
  (source, version, file list) and the Apache 2.0 licence text; the README
  lists them under the bundled third-party material.

## Acceptance criteria

- [ ] Given each FeRD sample invoice of the profiles Minimum to Extended (XML
      and, where present, PDF), when it is opened, then it is checked against
      its profile schema and rules, and the profile check reports no error
      (local test against the package, see Notes).
- [ ] Given a committed synthetic EN 16931-profile PDF invoice with an
      element that only the Extended profile allows, then the profile schema
      check reports it, and `Other data` shows the element and its value.
- [ ] Given a committed synthetic Minimum invoice in a PDF, then the profile
      schema and rules are applied, the summary names them, and a violated
      Minimum rule is shown as a finding with its rule ID.
- [ ] Given a committed synthetic Basic invoice with two preceding invoices
      (BG-3), then no schema finding is reported (D22B), while the same XML
      with the plain CEN identifier as XML file still gets the D16B finding
      of 0023.
- [ ] Given an assertion without `id` that is violated (e.g. an attribute
      marked as not used), then the finding has rule ID `FX` and the message
      from the Schematron.
- [ ] Given an invoice that violates a rule present in both the CEN rules and
      the profile rules, then the finding is shown once.
- [ ] Given an Extended sample, then every element not mapped to a business
      term appears under `Other data` with its value, at the right place.
- [ ] A plain CII XML invoice with the CEN identifier and the XRechnung and
      KoSIT tests give the same results as before (no regression).
- [ ] Opening and validating the largest Extended sample takes no longer
      than 2 s in a release build on the author's machine.
- [ ] The bundled files are unchanged copies (checksums in `SOURCE.md`);
      licence notes are in the README and next to the files.

## Out of scope

- ZUGFeRD 1.0 (different syntax).
- PDF/A conformance and the XMP metadata of the PDF (decision 7 of the
  requirements).
- Checking that the profile in the XMP metadata matches BT-24.
- A version note for other ZUGFeRD versions: the identifiers carry no
  version.

## Notes

- Sample invoices: not committed. A test reads them from the directory in
  `FERD_PACKAGE` (the unpacked package) and is skipped without it; the CI
  does not run it. Committed tests use synthetic invoices.
- Engine work: the Schematron engine needs `report`, `document()` for the
  code list file (resolved from the bundled files, never from disk or
  network), and assertions without `id`.
- If the Extended rule set is too slow to compile on every start, it is
  compiled lazily, like the other rule sets.

## Test list (proposed)

- T1 `ferd_samples` (local, `FERD_PACKAGE`): every sample of the profiles
  Minimum to Extended, XML and PDF: profile detected, profile schema and
  rules applied, no error from the profile check (AC1).
- T2 `extended_element_in_en16931_profile` (synthetic PDF) (AC2).
- T3 `minimum_profile_rules` (synthetic PDF, valid and with one violation)
  (AC3).
- T4 `two_preceding_invoices_d22b` (synthetic Basic; plain XML control)
  (AC4).
- T5 `assertion_without_id` (engine unit test and an invoice) (AC5).
- T6 `same_rule_shown_once` (AC6).
- T7 `other_data_for_extended` (synthetic Extended with unmapped elements in
  document, party and line) (AC7).
- T8 existing KoSIT, CEN and XRechnung tests unchanged (AC8).
- T9 `bundled_facturx_files_unchanged` (checksums from `SOURCE.md`) and
  packaging test for README and licence (AC10).
- P1 manual: largest Extended sample, release build, time to show (AC9).
