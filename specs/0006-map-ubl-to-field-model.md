---
id: 0006
title: Map UBL invoices and credit notes to the field model
status: reviewed
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

- [ ] For each of the samples A, B and L: given the UBL and the CII version,
      when both are mapped, then the lists of (section title, field id,
      display) are identical.
- [ ] Given UBL sample A, then BT-2 raw is `2026-11-17` and display
      `17.11.2026`.
- [ ] Given UBL sample B, then the second "Umsatzsteuer (BG-23)" section
      contains BT-118 display `Steuerbefreit (E)` and BT-120
      `Steuerbefreit nach § 4 Nr. 21 UStG`, and its BT-119 `location` selects
      the percent element of the second VAT subtotal.
- [ ] Given sample A as a UBL `CreditNote` with BT-3 `381`, then BT-3 display
      is `Gutschrift (381)`, and sections `Position 1` and `Position 2` exist
      with the same display values as in the UBL invoice.
- [ ] Given UBL sample A, then BT-31 is `DE123456789` and its `location`
      selects the company ID of the seller's VAT tax scheme
      (`cac:PartyTaxScheme` whose tax scheme ID is `VAT`), not any other
      company ID.
- [ ] Given UBL sample A without payment means, then no section
      "Zahlung (BG-16)" exists and no error occurs.

## Out of scope

- UBL-specific elements without an EN 16931 business term.
- Peppol BIS specifics beyond EN 16931.
