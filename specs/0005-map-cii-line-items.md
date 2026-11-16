---
id: 0005
title: Map CII invoice line items to the field model
status: draft
created: 2026-11-16
depends_on: [0004]
---

## Context

Invoice lines (BG-25) are what users check most often against the PDF:
quantities, prices, VAT rates. They extend the field model from 0004 and must
stay readable for invoices with many lines.

## Requirement

Given a CII invoice, the field model additionally contains one section per
invoice line, in document order, placed after "Käufer (BG-7)" and before
"Zahlung (BG-16)". Each line section uses the same field structure as 0004
(id, German label, raw, display, location).

## Contracts (optional)

Line section title: `Position <BT-126>` (e.g. `Position 1`). Fields:
BT-126, BT-127 (repeatable), BT-153, BT-154, BT-155, BT-129, BT-130, BT-146,
BT-149, BT-131, BT-151, BT-152, BT-128, BT-133.

## Acceptance criteria

- [ ] Given a CII invoice with two lines, when it is mapped, then the model
      contains sections `Position 1` and `Position 2` in that order, located
      between "Käufer (BG-7)" and "Zahlung (BG-16)".
- [ ] Given line `1` with item name `Beratung`, quantity `8` with unit code
      `HUR`, net price `125.00` and line net amount `1000.00` in currency
      `EUR`, then section `Position 1` contains BT-153 `Beratung`, BT-129
      display `8 Stunden (HUR)`, BT-146 display `125,00 EUR`, BT-131 display
      `1.000,00 EUR`.
- [ ] Given line `2` with VAT category `S` and rate `19`, then section
      `Position 2` contains BT-151 display `Normalsatz (S)` and BT-152 display
      `19 %`.
- [ ] Given a quantity `2.5` with unit code `KGM`, then BT-129 display is
      `2,5 Kilogramm (KGM)`; given a unit code not in the supported list,
      e.g. `XPP`, then the display is `2,5 XPP`.
- [ ] Given a line with two notes (BT-127) `Lieferung KW 46` and `inkl.
      Anfahrt`, then both appear as separate BT-127 fields in document order.
- [ ] Given an invoice with 500 lines, when it is mapped, then the model
      contains 500 line sections and mapping takes less than 1 s on the
      author's laptop.
- [ ] The `location` of BT-131 in `Position 2` selects the line net amount
      element of the second line and no other element.

## Out of scope

- Line-level allowances and charges (BG-27, BG-28), item attributes (BG-32),
  sub-lines of the Extended profile.
- UBL lines (0006).

## Notes

- Unit names are required for at least: `C62` (Stück), `HUR` (Stunden), `DAY`
  (Tage), `KGM` (Kilogramm), `MTR` (Meter), `LTR` (Liter), `H87` (Stück),
  `MON` (Monate), `LS` (pauschal). `XPP` stays deliberately unsupported so the
  fallback can be tested.
