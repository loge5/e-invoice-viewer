---
id: 0004
title: Map CII invoice header data to the field model
status: implemented
created: 2026-11-16
depends_on: [0003]
---

## Context

The core value of the viewer is a readable, structured view of the invoice
content, where every value is labelled with its EN 16931 business term (BT)
or group (BG) number. Users compare the fields with the PDF and with
validation findings, which refer to BT numbers. CII is the syntax of all
ZUGFeRD / Factur-X PDFs, so it comes first.

## Requirement

Given a CII invoice, the system produces a field model: an ordered list of
sections, each containing fields. Every field carries its BT number, the
German label, the raw value from the XML, a display value for humans, and the
location of its element in the XML. This spec covers the document header, the
parties, payment, VAT breakdown and totals. Line items follow in 0005.

- Fields that are absent in the invoice are not included. A section without
  any field is not included either.
- Sub-groups of a party (postal address, contact) are not separate sections;
  their fields appear directly in the party's section, in the order of the
  contract table.
- Display rules:
  - **Dates** in format 102 (YYYYMMDD) are shown as `DD.MM.YYYY`. Dates in any
    other format, and invalid dates, are shown as the raw value.
  - **Amounts** are shown with German separators, all decimal places present
    in the XML but at least two, followed by the invoice currency (BT-5).
  - **Codes** of BT-3, BT-81 and BT-118 are shown as `<name> (<code>)`;
    unknown codes and all other codes are shown raw.
  - **IBAN** (BT-84) is shown in groups of four characters.
  - **BT-24** is shown as the profile name detected by 0003 (e.g.
    `EN 16931`, `XRechnung 3.0`, `XRechnung 3.0 (Extension)`); for profile
    `unknown` the raw value is shown.

## Contracts (optional)

Sections in this order, with their fields in this order:

| Section (German title) | Fields |
|---|---|
| Rechnung | BT-1, BT-2, BT-3, BT-5, BT-9, BT-10, BT-24, BT-22 (repeatable) |
| Verkäufer (BG-4) | BT-27, BT-29, BT-31, BT-32, BT-35, BT-36, BT-38, BT-37, BT-40, BT-41, BT-42, BT-43 |
| Käufer (BG-7) | BT-44, BT-46, BT-48, BT-50, BT-51, BT-53, BT-52, BT-55 |
| Zahlung (BG-16) | BT-81, BT-83, BT-84, BT-85, BT-86, BT-20 |
| Umsatzsteuer (BG-23), repeatable | BT-116, BT-117, BT-118, BT-119, BT-120 |
| Summen (BG-22) | BT-106, BT-107, BT-108, BT-109, BT-110, BT-112, BT-113, BT-114, BT-115 |

Each field: `id` (e.g. `BT-1`), `label` (German), `raw` (string as in the
XML), `display` (formatted string), `location` (XPath of the element, with
position predicates for repeated elements).

## Acceptance criteria

All sample invoices are defined in [fixtures.md](../docs/fixtures.md).

- [ ] Given CII sample A, when it is mapped, then section "Rechnung" contains
      exactly, in this order: BT-1 `RE-2026-0042`; BT-2 raw `20261117`,
      display `17.11.2026`; BT-3 display `Rechnung (380)`; BT-5 `EUR`; BT-9
      display `01.12.2026`; BT-24 display `EN 16931`. BT-10 and BT-22 are not
      present.
- [ ] Given CII sample A, then section "Verkäufer (BG-4)" contains, in this
      order: BT-27 `Muster GmbH`, BT-31 `DE123456789`, BT-35
      `Hauptstraße 1`, BT-38 `12345`, BT-37 `Berlin`, BT-40 `DE`, BT-41
      `Erika Mustermann`, BT-42 `+49 30 1234567`, BT-43
      `rechnung@muster.example`.
- [ ] Given CII sample A, then section "Zahlung (BG-16)" contains BT-81
      display `SEPA-Überweisung (58)` and BT-84 display
      `DE02 1203 0000 0000 2020 51`; section "Summen (BG-22)" contains BT-106
      `1.050,00 EUR`, BT-110 `199,50 EUR`, BT-112 `1.249,50 EUR`, BT-115
      `1.249,50 EUR` (display values).
- [ ] Given CII sample B, then there are two "Umsatzsteuer (BG-23)" sections in
      document order, the first with BT-118 display `Normalsatz (S)` and BT-119
      `19`, the second with BT-118 display `Steuerbefreit (E)`, BT-119 `0`
      and BT-120 `Steuerbefreit nach § 4 Nr. 21 UStG`; section "Rechnung"
      contains BT-10 `04011000-12345-34` and BT-22
      `Vielen Dank für Ihren Auftrag.`
- [ ] Given sample A with these values, the display is:

      | element | raw | display |
      |---|---|---|
      | BT-24 | `urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0` | `XRechnung 3.0` |
      | BT-24 | `urn:cen.eu:en16931:2017#conformant#urn:xeinkauf.de:kosit:extension:xrechnung_3.0` | `XRechnung 3.0 (Extension)` |
      | BT-24 | `urn:example:custom-profile` | `urn:example:custom-profile` |
      | BT-115 | `12.3456` | `12,3456 EUR` |
      | BT-115 | `-50.00` | `-50,00 EUR` |
      | BT-115 | `1000` | `1.000,00 EUR` |
      | BT-115 with BT-5 `CHF` | `1000.00` | `1.000,00 CHF` |
      | BT-2 with format `610` | `202611` | `202611` |
      | BT-2 with format `102` | `20261345` | `20261345` |

- [ ] Given sample A without payment instructions, then the model contains no
      section "Zahlung (BG-16)" and all other sections are unchanged.
- [ ] Every field's `location` selects exactly the element its raw value was
      read from (checked for BT-1 and BT-84 in sample A, and for BT-119 of the
      second BG-23 in sample B).

## Out of scope

- Line items (0005), UBL (0006).
- Fields of the Extended profile beyond EN 16931.
- Business terms not listed in the contract table; they can be added in later
  specs. They must not cause errors when present.
- Validation of values (0009).

## Notes

- German labels: the KoSIT XRechnung visualization ships German BT names under
  Apache 2.0 and can serve as the reference for wording.

## Progress

- **Done** — `e_invoice_viewer_core::fields::map_invoice` implemented for CII
  (`crates/core/src/fields/`: `mod.rs` mapping engine and locations,
  `cii.rs` business-term table, `display.rs` display rules and code lists).
  CII samples A and B added under `crates/core/tests/fixtures/cii/`; 13 tests
  in `crates/core/tests/map_cii_header.rs`. No new dependency. UBL returns
  `MapError::UnsupportedSyntax` until 0006.
- **Verified** — 2026-11-16: `cargo test --workspace` 43 passed (13 new);
  `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
  warnings` and architecture check clean. Manual smoke run on a real
  Factur-X invoice: all six sections mapped in < 1 ms.
- **Next** — none; set to `implemented` by the author on 2026-11-16.
