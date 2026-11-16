---
id: 0006
title: Map UBL invoices and credit notes to the field model
status: draft
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
`raw` values that are syntax-specific and in the `location` values.

## Acceptance criteria

- [ ] Given the UBL counterpart of the 0004/0005 sample invoice (same business
      content: `RE-2026-0042`, `Muster GmbH`, two lines, totals `1190.00`
      `EUR`), when it is mapped, then the list of (section title, field id,
      display) is identical to the CII result.
- [ ] Given the UBL invoice, BT-2 raw is `2026-11-17` (ISO format, as in UBL)
      and display `17.11.2026`.
- [ ] Given a UBL `CreditNote` with BT-3 `381`, then BT-3 display is
      `Gutschrift (381)` and the invoice lines are read from the credit-note
      line elements, i.e. `Position 1` and `Position 2` exist with their
      values.
- [ ] Given a UBL invoice with seller VAT ID `DE123456789`, then BT-31 is
      `DE123456789` and its `location` selects the VAT scheme's company ID
      element of the seller party.
- [ ] Given a UBL invoice without payment instructions, then no section
      "Zahlung (BG-16)" exists and no error occurs.

## Out of scope

- UBL-specific elements without an EN 16931 business term.
- Peppol BIS specifics beyond EN 16931.
