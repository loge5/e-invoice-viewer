---
id: 0003
title: Detect the syntax and profile of an invoice XML
status: draft
created: 2026-11-16
depends_on: []
---

## Context

An invoice XML comes in one of two syntaxes, UN/CEFACT CII or OASIS UBL
(invoice or credit note), and declares a profile (Minimum, Basic WL, Basic,
EN 16931, Extended, XRechnung) in its specification identifier (BT-24). The
field mapping, the applicable rules and the hints shown to the user depend on
both. The input may also be an XML file that is not an invoice at all.

## Requirement

Given the bytes of an XML document, the system determines the syntax, the
document kind and the profile, or returns a specific error. The raw value of
BT-24 is always kept so that unknown profiles can still be shown.

## Contracts (optional)

Result of a successful detection:

| Field | Values |
|---|---|
| syntax | `CII`, `UBL` |
| document kind | `invoice`, `credit-note` |
| profile | `minimum`, `basic-wl`, `basic`, `en16931`, `extended`, `xrechnung`, `unknown` |
| profile version | for XRechnung the version from the identifier (e.g. `3.0`), otherwise empty |
| specification identifier | raw value of BT-24, or empty if missing |

## Acceptance criteria

- [ ] Given a document with root element `CrossIndustryInvoice` in namespace
      `urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100`, when it
      is detected, then syntax is `CII` and document kind is `invoice`.
- [ ] Given a document with root `Invoice` in namespace
      `urn:oasis:names:specification:ubl:schema:xsd:Invoice-2`, then syntax is
      `UBL`, kind `invoice`; given root `CreditNote` in namespace
      `urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2`, then syntax
      is `UBL`, kind `credit-note`.
- [ ] Given these BT-24 values, the profile is detected as follows:

      | BT-24 | profile | version |
      |---|---|---|
      | `urn:factur-x.eu:1p0:minimum` | `minimum` | |
      | `urn:factur-x.eu:1p0:basicwl` | `basic-wl` | |
      | `urn:cen.eu:en16931:2017#compliant#urn:factur-x.eu:1p0:basic` | `basic` | |
      | `urn:cen.eu:en16931:2017` | `en16931` | |
      | `urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended` | `extended` | |
      | `urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0` | `xrechnung` | `3.0` |
      | `urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_2.3` | `xrechnung` | `2.3` |

- [ ] Given a CII invoice with BT-24 `urn:example:custom-profile`, then the
      profile is `unknown` and the specification identifier is
      `urn:example:custom-profile`.
- [ ] Given a CII invoice without BT-24, then the profile is `unknown` and the
      specification identifier is empty (this is not an error; the missing
      BT-24 is reported later by validation).
- [ ] Given well-formed XML with root `<order>` (no invoice namespace), then the
      error is "not an electronic invoice"; given malformed XML (`<a><b></a>`),
      then the error is "XML is not well-formed" including the line number of
      the first problem.
- [ ] Given a document that declares an external entity
      (`<!DOCTYPE x [<!ENTITY e SYSTEM "file:///etc/passwd">]>`), when it is
      detected, then the entity is not resolved and no file is read.

## Out of scope

- Mapping fields (0004 to 0006), validation (0009, 0010).
- Support for ZUGFeRD 1.0 (root namespace
  `urn:ferd:CrossIndustryDocument:invoice:1p0`). Such documents return the
  error "unsupported format: ZUGFeRD 1.0" instead of "not an electronic
  invoice", so the user knows why.
