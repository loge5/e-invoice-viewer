---
id: 0014
title: Link fields and XML elements in both directions
status: reviewed
created: 2026-11-16
depends_on: [0008, 0011]
---

## Context

When a value looks wrong, users want to see where it comes from in the XML,
and when reading the XML, which business term an element represents. The
field model already knows the location of every value (0004, 0005), and every
validation finding has a location (0009). Findings that belong to no field
(`Dokument` in 0011) can only be found in the XML.

## Requirement

**Field → XML.** The context menu of a field value offers "Im XML zeigen",
next to "Wert kopieren" (0007). It is also reachable by keyboard (menu key or
Shift+F10). Choosing it switches the pane to the XML view, scrolls the
field's element into view and selects the whole element, from start tag to
end tag. The cursor is placed in the element's text, or in the attribute value
for a field that is an attribute (e.g. BT-130 unit).

**Finding → XML.** Every entry of the validation summary (0011), including
`Dokument` entries, has "Im XML zeigen" in its context menu. It goes to the
finding's location in the same way. Clicking an entry keeps jumping to the
field or section as in 0011.

**XML → field.** Below the XML view is an assignment line. It shows what the
element at the cursor belongs to, and a button "In Feldern zeigen":

1. If the cursor is in an element of a field, the line shows that field's
   number and label, e.g. `BT-1 Rechnungsnummer`. If two fields share the
   element, the attribute field counts when the cursor is in its attribute
   value, otherwise the text field (BT-129 `Menge` and BT-130 `Einheit` share
   the quantity element). In a repeated section, the section title is added,
   e.g. `BT-131 Nettobetrag · Position 2`.
2. Otherwise, if the cursor is within the element of a section, the deepest
   such section is shown, e.g. `Verkäufer (BG-4)`. The section of the root
   element (`Rechnung`) does not count, as in 0011.
3. Otherwise the line shows `Kein Feld zugeordnet`, and "In Feldern zeigen"
   is disabled.

"In Feldern zeigen" switches to the field view, scrolls to the field or
section header and highlights it for at least 2 s, as a summary entry does
in 0011. The assignment line is separate from the status line of 0011, which
keeps showing the validation result.

The XML view shows the XML reformatted (0008). The linking works on the
content shown there, also for invoices stored on a single line.

## Acceptance criteria

Sample invoices are defined in [fixtures.md](../docs/fixtures.md); time limits
refer to the reference machine in [requirements](../docs/requirements.md).

- [ ] Given sample A, when the user chooses "Im XML zeigen" in the context
      menu of BT-84, then the pane switches to the XML view, and the element
      holding `DE02120300000000202051` is scrolled into view and selected
      from start tag to end tag.
- [ ] Given sample B, when the user chooses "Im XML zeigen" on BT-119 of the
      second "Umsatzsteuer (BG-23)" section, then the rate element of the
      second breakdown is selected, not the first.
- [ ] Given sample A, "Im XML zeigen" on BT-131 of "Position 2" selects the
      net amount of the second line. "Im XML zeigen" on BT-130 places the
      cursor in the value of the `unitCode` attribute.
- [ ] Given the XML view of sample A, when the cursor is placed inside the
      text of the invoice number element, then the assignment line shows
      `BT-1 Rechnungsnummer`; choosing "In Feldern zeigen" switches to the
      field view with BT-1 visible and highlighted for at least 2 s.
- [ ] Given the XML view of sample A:

      | cursor in | assignment line |
      |---|---|
      | the text of the quantity of line 1 | `BT-129 Menge · Position 1` |
      | the `unitCode` value of line 1 | `BT-130 Einheit · Position 1` |
      | the net amount of line 2 | `BT-131 Nettobetrag · Position 2` |
      | the seller element, outside its fields | `Verkäufer (BG-4)` |
      | the root element, outside all sections | `Kein Feld zugeordnet` |

  - For `Verkäufer (BG-4)`, "In Feldern zeigen" scrolls to the section
    header and highlights it.
  - For `Kein Feld zugeordnet`, "In Feldern zeigen" is disabled.
- [ ] Given CII sample B declaring XRechnung version 2.3 and without BT-10
      (variants of 0010), when the user chooses "Im XML zeigen" on the
      summary entry `BR-DE-15` (`Dokument`), then the XML view selects the
      element at the finding's location.
  - Clicking the entry `BR-DE-21` still goes to BT-24 in the field view.
- [ ] Given the single-line variant of sample L, "Im XML zeigen" on BT-131
      of "Position 500" selects the net amount of the 500th line, and the
      cursor there shows `BT-131 Nettobetrag · Position 500`.
- [ ] Given sample L in the XML view, the assignment line is updated within
      100 ms after the cursor moves (manual check: hold an arrow key; the
      line follows without visible lag).
- [ ] Given the XML view, the status line of 0011 still shows the validation
      headline, unchanged by the cursor position.

## Out of scope

- Editing; linking PDF text positions to fields.
- UBL invoices: the app cannot open them before 0006. They are linked like
  CII invoices once 0006 is implemented.
- Attachments (0013): their BG-24 elements show `Kein Feld zugeordnet`.
- Highlighting findings inside the XML view.

## Notes

- GUI-bound; verified manually. The lookup from a cursor position to a field
  or section, and from a location to a text range in the reformatted XML, is
  core logic and unit-tested, including the single-line variant of sample L.
- Review on 2026-11-17: the author decided on a separate assignment line
  below the XML view, "Im XML zeigen" for every summary entry (requested by
  0011), the context menu as the only trigger, and section assignment for
  elements outside fields.
