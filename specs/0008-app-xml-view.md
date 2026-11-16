---
id: 0008
title: App - show the raw invoice XML
status: reviewed
created: 2026-11-16
depends_on: [0007]
---

## Context

Developers and accountants regularly need to see the original XML: to check a
value the structured view does not show, to compare with the sender's system,
or to copy it into a support request.

## Requirement

For an opened invoice the user can switch the field pane to an XML view that
shows the invoice XML read-only, readable and searchable, and can copy it or
save it as a file. The shown XML is the original content; any reformatting for
display must not change the copied or saved content. Search is
case-insensitive.

## Acceptance criteria

Sample invoices are defined in [fixtures.md](../docs/fixtures.md); time limits
refer to the reference machine in [requirements](../docs/requirements.md).

- [ ] Given an opened invoice, the field pane offers two views "Felder" and
      "XML"; switching to "XML" shows the invoice XML; switching back restores
      the field view at the previous scroll position. Given a PDF without
      embedded invoice, the "XML" view cannot be selected.
- [ ] Given the single-line variant of sample L, then the XML view shows it
      indented, one element per line, with line numbers and syntax
      highlighting (element names, attributes and text in distinct colours),
      readable in both light and dark appearance.
- [ ] Given a CII sample A saved with `encoding="ISO-8859-1"`, then the XML
      view shows `Hauptstraße` and `München` correctly (no replacement
      characters).
- [ ] Given sample A in the XML view, when the user presses Ctrl+F and types
      `de02120300`, then the match `DE02120300…` is highlighted and scrolled
      into view and `1 von 1` is shown; Enter jumps to the next match; for the
      search text `xyz-nicht-vorhanden` the view shows `Keine Treffer`.
- [ ] Given the user chooses "XML kopieren", then the clipboard contains the
      original XML byte-for-byte (not the indented display version).
- [ ] Given the user chooses "XML speichern unter…" for the invoice from
      `RE-2026-0042.pdf`, then the save dialog proposes the file name
      `RE-2026-0042.xml`, and the saved file is byte-identical to the
      extracted XML.
- [ ] Given a 2 MB invoice XML, switching to the XML view takes less than 1 s.

## Out of scope

- Editing the XML.
- Linking fields and XML elements (0014).

## Notes

- GUI-bound criteria are verified manually; the "original bytes" property of
  copy/save is additionally covered by a test of the export function.
