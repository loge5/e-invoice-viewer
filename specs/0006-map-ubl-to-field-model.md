---
id: 0006
title: Map UBL invoices and credit notes to the field model
status: implemented
created: 2026-11-16
depends_on: [0004, 0005]
---

## Context

XRechnung is often sent as a plain UBL file. The viewer must show UBL invoices
in exactly the same structured view as CII invoices, so users do not need to
know which syntax they received.

## Requirement

Given a UBL invoice or credit note, the system produces the same field model
(sections, field ids, labels, display rules) as defined in 0004 and 0005. For
the same business content, the model differs from the CII result only in the
`raw` values whose format is syntax-specific (e.g. dates) and in the
`location` values.

## Acceptance criteria

All sample invoices are defined in [fixtures.md](../docs/fixtures.md); each
exists as CII and as UBL with the same business content.

- [x] For each of the samples A, B and L: given the UBL and the CII version,
      when both are mapped, then the lists of (section title, field id,
      display) are identical.
- [x] Given UBL sample A, then BT-2 raw is `2026-11-17` and display
      `17.11.2026`.
- [x] Given UBL sample B, then the second "Umsatzsteuer (BG-23)" section
      contains BT-118 display `Steuerbefreit (E)` and BT-120
      `Steuerbefreit nach § 4 Nr. 21 UStG`, and its BT-119 `location` selects
      the percent element of the second VAT subtotal.
- [x] Given sample A as a UBL `CreditNote` with BT-3 `381`, then BT-3 display
      is `Gutschrift (381)`, and sections `Position 1` and `Position 2` exist
      with the same display values as in the UBL invoice.
- [x] Given UBL sample A, then BT-31 is `DE123456789` and its `location`
      selects the company ID of the seller's VAT tax scheme
      (`cac:PartyTaxScheme` whose tax scheme ID is `VAT`), not any other
      company ID.
- [x] Given UBL sample A without payment means, then no section
      "Zahlung (BG-16)" exists and no error occurs.

## Out of scope

- UBL-specific elements without an EN 16931 business term.
- Peppol BIS specifics beyond EN 16931.

## Notes

- Correction (2026-11-18, during implementation): for sample B, the lists
  in AC1 cannot be identical, because UBL allows only one note per line
  (BT-127). UBL sample B carries only the first of the two notes of CII
  sample B, as documented in [fixtures](../docs/fixtures.md). AC1 compares
  UBL sample B with CII sample B without the second line note.
- Correction after the code review on 2026-11-18: BT-110 is taken from the
  VAT total that has a breakdown (`cac:TaxSubtotal`), so that a VAT total in
  accounting currency (BT-111) before it is not shown as BT-110 (test
  `vat_total_ignores_accounting_currency_total`).

## Progress

- **Done**:
  - `crates/core/src/fields/ubl.rs`: the UBL paths for all sections and
    fields of 0004 and 0005, for invoices and credit notes (`CreditNoteLine`,
    `CreditedQuantity`, `CreditNoteTypeCode`, BT-9 in the payment means).
    Location prefixes are those of the EN 16931 UBL rules (`ubl`, `cn`,
    `cac`, `cbc`), so that 0011 attributes UBL findings like CII findings.
  - The mapping machinery is shared: a step may accept a second element
    name and may be restricted by the text of a child (BT-31 from the
    `VAT` tax scheme, BT-32 from another one). A field path may start at
    the root (UBL BT-110 lies outside the monetary total). UBL dates are
    shown as `DD.MM.YYYY`.
  - App: the message "UBL-Rechnungen werden noch nicht unterstützt" is gone;
    UBL invoices open like CII invoices.
- **Verified** (2026-11-18):
  - `cargo test --workspace`: 137 passed; new: 8 tests in `map_ubl.rs`
    (AC1–AC6, the location contract, and BR-CO-15/BR-CO-16 of UBL sample A
    with BT-112 = 1300.00 going to BT-112/BT-115 as in 0011).
  - `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --
    -D warnings` are clean.
  - Smoke run: UBL sample A opens in the app with `BT-1 · Rechnungsnummer ·
    RE-2026-0042`, `Keine Befunde` and `Geprüft nach EN 16931 UBL 1.3.17`.
- **Manually verified** by the author (2026-11-18):
  - No manual checks needed; all criteria are covered by automated tests.
    The author decided on `implemented`.
