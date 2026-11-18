---
id: 0014
title: Link fields and XML elements in both directions
status: implemented
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

- [x] Given sample A, when the user chooses "Im XML zeigen" in the context
      menu of BT-84, then the pane switches to the XML view, and the element
      holding `DE02120300000000202051` is scrolled into view and selected
      from start tag to end tag.
- [x] Given sample B, when the user chooses "Im XML zeigen" on BT-119 of the
      second "Umsatzsteuer (BG-23)" section, then the rate element of the
      second breakdown is selected, not the first.
- [x] Given sample A, "Im XML zeigen" on BT-131 of "Position 2" selects the
      net amount of the second line. "Im XML zeigen" on BT-130 places the
      cursor in the value of the `unitCode` attribute.
- [x] Given the XML view of sample A, when the cursor is placed inside the
      text of the invoice number element, then the assignment line shows
      `BT-1 Rechnungsnummer`; choosing "In Feldern zeigen" switches to the
      field view with BT-1 visible and highlighted for at least 2 s.
- [x] Given the XML view of sample A:

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
- [x] Given CII sample B declaring XRechnung version 2.3 and without BT-10
      (variants of 0010), when the user chooses "Im XML zeigen" on the
      summary entry `BR-DE-15` (`Dokument`), then the XML view selects the
      element at the finding's location.
  - Clicking the entry `BR-DE-21` still goes to BT-24 in the field view.
- [x] Given the single-line variant of sample L, "Im XML zeigen" on BT-131
      of "Position 500" selects the net amount of the 500th line, and the
      cursor there shows `BT-131 Nettobetrag · Position 500`.
- [x] Given sample L in the XML view, the assignment line is updated within
      100 ms after the cursor moves (manual check: hold an arrow key; the
      line follows without visible lag).
- [x] Given the XML view, the status line of 0011 still shows the validation
      headline, unchanged by the cursor position.

## Out of scope

- Editing; linking PDF text positions to fields.
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
- Correction (2026-11-18, during implementation): 0006 was implemented
  first, so UBL invoices are linked too (same location format); the
  out-of-scope item for UBL is dropped.
- "Selects the whole element" is shown as a highlight of the element from
  start to end tag, because the text cursor must at the same time stand in
  the element's text or attribute value; a real selection could not do both.

## Progress

- **Done**:
  - Core `link.rs`: `XmlLayout` reformats the XML like 0008 and records,
    for every element, its location (same format as field and finding
    locations), its range from start to end tag, the start of its text and
    the value ranges of its attributes. `assignment(model, layout, offset)`
    applies the three rules of the requirement. `xml_view` reports where
    each piece of the input ends up in the reformatted text.
  - The field model now says which attribute a field is (`Field.attribute`,
    BT-130) and whether a section is repeated (`Section.repeated`).
  - App:
    - "Im XML zeigen" in the context menu of every field value (next to
      "Wert kopieren") and of every summary entry (right click, menu key or
      Shift+F10). It switches to the XML view, highlights the element and
      places the cursor in its text or attribute value.
    - The assignment line below the XML view follows the cursor;
      "In Feldern zeigen" switches to the field view and highlights the
      field or section header for 2.5 s, as in 0011.
- **Verified** (2026-11-18):
  - `cargo test --workspace`: 160 passed; new: 7 tests in
    `link_fields_xml.rs` and `assignment_texts`.
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
    warnings` and the architecture check are clean.
  - Smoke run of the release build with sample A: switching to the XML view
    shows `Kein Feld zugeordnet` with "In Feldern zeigen" disabled, and the
    status line still shows `Keine Befunde` (AC9). Placing the caret through
    AT-SPI gives `BT-1 Rechnungsnummer`, `BT-129 Menge · Position 1`,
    `BT-130 Einheit · Position 1` and `Verkäufer (BG-4)` (AC4, AC5).
  - Found and fixed in the smoke run: loading the XML view moved the cursor
    while the source was borrowed, which aborted the app.
- **Manually verified** by the author (2026-11-18):
  - AC1–AC3: "Im XML zeigen" works; AC4/AC5: "In Feldern zeigen" works;
    AC6 and AC8 as expected.
