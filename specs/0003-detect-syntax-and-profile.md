---
id: 0003
title: Detect the syntax and profile of an invoice XML
status: implemented
created: 2026-11-16
depends_on: []
---

## Context

An invoice XML comes in one of two syntaxes, UN/CEFACT CII or OASIS UBL, is
either an invoice or a credit note, and declares a profile (Minimum,
Basic WL, Basic, EN 16931, Extended, XRechnung) in its specification
identifier (BT-24). The field mapping, the applicable rules and the hints
shown to the user depend on these. The input may also be an XML file that is
not an invoice at all, or a deliberately malicious XML file.

## Requirement

Given the bytes of an XML document, the system determines the syntax, the
document kind and the profile, or returns a specific error. The raw value of
BT-24 is always kept so that unknown profiles can still be shown.

- **Syntax** follows from the root element and its namespace.
- **Document kind** follows from the invoice type code (BT-3): `381` and
  `261` mean credit note, every other value means invoice. For UBL, a
  `CreditNote` root also means credit note.
- **Profile** follows from BT-24 after removing leading and trailing
  whitespace. The XRechnung Extension identifier counts as profile
  `xrechnung` with the extension flag set. So does the CIUS identifier
  followed by the Extension suffix (`#conformant#…extension:xrechnung_<v>`),
  the official form since XRechnung 2.x. The CIUS identifier followed by the
  XRechnung CVD suffix (`#compliant#…xrechnung:cvd_<v>`) counts as profile
  `xrechnung` without the extension flag.
- Documents containing a DOCTYPE declaration are rejected. Real invoices never
  contain one, and this rules out external entities and entity expansion
  attacks.

## Contracts (optional)

Result of a successful detection:

| Field | Values |
|---|---|
| syntax | `CII`, `UBL` |
| document kind | `invoice`, `credit-note` |
| profile | `minimum`, `basic-wl`, `basic`, `en16931`, `extended`, `xrechnung`, `unknown` |
| profile version | for XRechnung the version from the identifier (e.g. `3.0`), otherwise empty |
| extension | `true` for the XRechnung Extension, otherwise `false` |
| specification identifier | raw value of BT-24 (untrimmed), or empty if missing |

## Acceptance criteria

- [ ] Given these root elements, the syntax is detected as follows:

      | root element | namespace | syntax |
      |---|---|---|
      | `CrossIndustryInvoice` | `urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100` | `CII` |
      | `Invoice` | `urn:oasis:names:specification:ubl:schema:xsd:Invoice-2` | `UBL` |
      | `CreditNote` | `urn:oasis:names:specification:ubl:schema:xsd:CreditNote-2` | `UBL` |

- [ ] Given a CII document with BT-3 `380`, the kind is `invoice`; with BT-3
      `381`, `credit-note`. Given a UBL `Invoice` with BT-3 `380`, the kind is
      `invoice`; a UBL `CreditNote` with BT-3 `381`, `credit-note`.
- [ ] Given a CII invoice with these BT-24 values, the profile is detected as
      follows:

      | BT-24 | profile | version | extension |
      |---|---|---|---|
      | `urn:factur-x.eu:1p0:minimum` | `minimum` | | `false` |
      | `urn:factur-x.eu:1p0:basicwl` | `basic-wl` | | `false` |
      | `urn:cen.eu:en16931:2017#compliant#urn:factur-x.eu:1p0:basic` | `basic` | | `false` |
      | `urn:cen.eu:en16931:2017` | `en16931` | | `false` |
      | `␠urn:cen.eu:en16931:2017␠` (leading and trailing space) | `en16931` | | `false` |
      | `urn:cen.eu:en16931:2017#conformant#urn:factur-x.eu:1p0:extended` | `extended` | | `false` |
      | `urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0` | `xrechnung` | `3.0` | `false` |
      | `urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_2.3` | `xrechnung` | `2.3` | `false` |
      | `urn:cen.eu:en16931:2017#conformant#urn:xeinkauf.de:kosit:extension:xrechnung_3.0` | `xrechnung` | `3.0` | `true` |
      | `urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0#conformant#urn:xeinkauf.de:kosit:extension:xrechnung_3.0` | `xrechnung` | `3.0` | `true` |
      | `urn:cen.eu:en16931:2017#compliant#urn:xoev-de:kosit:standard:xrechnung_2.3#conformant#urn:xoev-de:kosit:extension:xrechnung_2.3` | `xrechnung` | `2.3` | `true` |
      | `urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0#compliant#urn:xeinkauf.de:kosit:xrechnung:cvd_0.9` | `xrechnung` | `3.0` | `false` |
      | `urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0#compliant#urn:example:other` | `unknown` | | `false` |
      | `urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0` | `unknown` | | `false` |

- [ ] Given a UBL invoice whose BT-24 is
      `urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0`,
      then the profile is `xrechnung` with version `3.0`.
- [ ] Given a CII invoice with BT-24 `urn:example:custom-profile`, then the
      profile is `unknown` and the specification identifier is
      `urn:example:custom-profile`. Given a CII invoice without BT-24, then the
      profile is `unknown` and the specification identifier is empty; this is
      not an error (the missing BT-24 is reported later by validation).
- [ ] Error cases:

      | input | error |
      |---|---|
      | well-formed XML with root `<order>` | "not an electronic invoice" |
      | `<a><b></a>` | "XML is not well-formed", line `1` |
      | root `CrossIndustryDocument` in namespace `urn:ferd:CrossIndustryDocument:invoice:1p0` | "unsupported format: ZUGFeRD 1.0" |

- [ ] Given a document with a DOCTYPE declaration, either with an external
      entity (`<!DOCTYPE x [<!ENTITY e SYSTEM "file:///etc/passwd">]>`) or
      with nested internal entities expanding to more than 1 GB ("billion
      laughs"), then the error is "XML contains a DOCTYPE declaration – not
      supported", returned within 1 s, without resolving any entity.

## Out of scope

- Mapping fields (0004 to 0006), validation (0009, 0010).
- Support for ZUGFeRD 1.0 content (only recognised to give a clear error).
- Peppol BIS as a separate profile (shown as `unknown` with its identifier).

## Progress

- **Done** — `e_invoice_viewer_core::detect::detect` implemented
  (`crates/core/src/detect.rs`); 17 tests in
  `crates/core/tests/detect_invoice.rs`. New dependency: `roxmltree` 0.21
  (MIT OR Apache-2.0); its default options reject any DOCTYPE.
- **Verified** — 2026-11-16: `cargo test --workspace` 30 passed (17 new);
  `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
  warnings`, architecture check and `cargo audit` clean. Manual smoke run on
  two real Factur-X invoices: both detected as CII / invoice / `en16931`.
- **Next** — none; set to `implemented` by the author on 2026-11-16.
- **Correction** — 2026-11-17: the official identifiers of the XRechnung
  Extension and XRechnung CVD were detected as `unknown`. They append a
  suffix to the CIUS identifier, and the spec only listed a form without the
  CIUS part. Found while implementing 0010. Four rows were added to the
  BT-24 table, and detection and tests were adjusted.
  - `cargo test --workspace`: 102 passed.
  - The BT-24 display test covers the official Extension identifier.
