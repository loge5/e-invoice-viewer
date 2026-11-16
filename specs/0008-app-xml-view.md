---
id: 0008
title: App - show the raw invoice XML
status: draft
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
display must not change the copied or saved content.

## Acceptance criteria

- [ ] Given an opened invoice, the field pane offers two views "Felder" and
      "XML"; switching to "XML" shows the invoice XML; switching back restores
      the field view at the previous scroll position.
- [ ] Given a CII invoice stored on a single line (no line breaks, 40 KB), then
      the XML view shows it indented, one element per line, with line numbers
      and syntax highlighting (element names, attributes, text in distinct
      colours), in both light and dark appearance.
- [ ] Given the user presses Ctrl+F in the XML view and types `DE02120300`,
      then the first match is highlighted and scrolled into view, and the
      number of matches is shown (e.g. `1 von 1`); Enter jumps to the next
      match.
- [ ] Given the user chooses "XML kopieren", then the clipboard contains the
      original XML byte-for-byte (not the indented display version).
- [ ] Given the user chooses "XML speichern unter…" for an invoice extracted
      from `RE-2026-0042.pdf`, then the save dialog proposes the file name
      `RE-2026-0042.xml`, and the saved file is byte-identical to the
      extracted XML.
- [ ] Given a 2 MB invoice XML, switching to the XML view takes less than 1 s.

## Out of scope

- Editing the XML.
- Linking fields and XML elements (0014).

## Notes

- Criteria 1 to 5 are GUI-bound and verified manually; the "original bytes"
  property of copy/save is additionally covered by a test of the export
  function.
