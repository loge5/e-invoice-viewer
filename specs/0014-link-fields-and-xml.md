---
id: 0014
title: Link fields and XML elements in both directions
status: draft
created: 2026-11-16
depends_on: [0008]
---

## Context

When a value looks wrong, users want to see where it comes from in the XML,
and when reading the XML, which business term an element represents. The
field model already knows the location of every value (0004 to 0006).

## Requirement

From a field the user can jump to its element in the XML view, which is then
highlighted. In the XML view, placing the cursor in an element that belongs to
a field shows that field's BT/BG number and label, with an action to jump back
to the field.

## Acceptance criteria

- [ ] Given an opened invoice, when the user chooses "Im XML zeigen" on field
      BT-84 (context menu or the field's action button), then the pane
      switches to the XML view, the element holding
      `DE02120300000000202051` is scrolled into view and highlighted.
- [ ] Given sample B ([fixtures](../docs/fixtures.md)), when the user chooses "Im XML
      zeigen" on BT-119 of the second "Umsatzsteuer (BG-23)" section, then the
      rate element of the second breakdown is highlighted, not the first.
- [ ] Given the XML view, when the cursor is placed inside the text of the
      invoice number element, then the status line shows
      `BT-1 Rechnungsnummer`; choosing "In Feldern zeigen" switches to the
      field view with BT-1 visible and highlighted.
- [ ] Given the cursor is placed in an element that is not part of the field
      model, then the status line shows `Kein Feld zugeordnet` and "In Feldern
      zeigen" is disabled.

## Out of scope

- Editing, linking PDF text positions to fields.

## Notes

- GUI-bound; verified manually. The lookup from a cursor position to a field
  is core logic and unit-tested.
