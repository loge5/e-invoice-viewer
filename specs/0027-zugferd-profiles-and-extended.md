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

**Open until the package is there**: the licence of the ZUGFeRD 2.5 technical
artefacts (2.2 was Apache 2.0). If it does not allow redistribution, the
profile rules stay out and only the Extended content is shown.

## Requirement

- **Profile check**: for a CII invoice that declares a ZUGFeRD/Factur-X
  profile (BT-24, detected by 0003), the app also checks the profile's XML
  schema and Schematron of the bundled ZUGFeRD version (2.5.x). Findings
  carry the rule IDs of the FeRD artefacts and are shown and attributed like
  all findings; the summary names the profile rules, e.g. `Checked against XML
  schema CII D16B, ZUGFeRD 2.5.2 EN 16931 and EN 16931 CII 1.3.17`.
- **Minimum and Basic WL**: these profiles are not complete invoices (0009);
  with this spec they are checked against their own profile rules instead of
  only being marked as not applicable.
- **Extended content**: elements of the invoice that are not mapped to a
  business term (Extended profile, XRechnung extension, or anything else the
  schema allows) are not hidden. A section `Weitere Angaben` / `Other data`
  per place (document, party, line) lists them with their element names and
  values, in document order, so that nothing in the invoice is invisible.
- Versions: the bundled ZUGFeRD version is named in the summary; an invoice
  that declares another ZUGFeRD version gets a note like the XRechnung
  version note of 0010.

## Acceptance criteria

To be completed once the FeRD package is available; drafts:

- [ ] Given the FeRD sample invoices of each profile (PDF and XML), then each
      is checked against its profile and shows no finding where FeRD declares
      it valid.
- [ ] Given an EN 16931-profile sample with an Extended-only element added,
      then the profile schema check reports it, and `Weitere Angaben` shows
      it.
- [ ] Given a Minimum-profile sample, then the profile rules are applied and
      the summary names them.
- [ ] Given an Extended sample, then every element not mapped to a business
      term appears under `Weitere Angaben` with its value.
- [ ] Licence notes for the bundled FeRD artefacts are in the README and next
      to the files.

## Out of scope

- ZUGFeRD 1.0 (different syntax).
- PDF/A conformance (decision 7 of the requirements).

## Notes

- Blocked until the author provides the FeRD ZUGFeRD 2.5.x package; the
  review of this spec happens then.

## Test list (proposed)

Follows with the review of this spec, once the FeRD package is available.
