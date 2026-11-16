---
id: 0016
title: App - navigate and zoom multi-page PDFs
status: reviewed
created: 2026-11-16
depends_on: [0007]
---

## Context

Many invoices have several pages (cover letter, line item pages, terms), and
small print must be readable. Split off from 0007 to keep that spec small.

## Requirement

In the PDF pane the user can scroll through all pages of the PDF, see which
page is visible, and zoom in and out. A newly opened PDF is shown fitted to
the pane width.

## Acceptance criteria

Time limits refer to the reference machine in
[requirements](../docs/requirements.md).

- [ ] Given a 3-page invoice PDF is opened, then all 3 pages are shown one
      below the other with a visible gap, and scrolling reaches the end of
      page 3.
- [ ] Given the 3-page PDF, a page indicator shows `Seite 1 von 3` initially,
      and `Seite 2 von 3` once page 2 fills most of the visible area.
- [ ] Given a newly opened PDF, then its pages are scaled to fit the pane
      width; when the divider is dragged, the pages are re-scaled to the new
      width while fit-width is active.
- [ ] Given the PDF pane has focus, Ctrl++ zooms in and Ctrl+- zooms out in
      steps (100 %, 125 %, 150 %, 200 %, 300 %, 400 % and 75 %, 50 %, 25 %),
      Ctrl+0 returns to fit width; the zoom level is shown (e.g. `150 %`), and
      zooming never goes beyond 25 % or 400 %.
- [ ] Given page 2 is visible and the user zooms to 200 %, then page 2 stays
      in view (the visible page does not jump).
- [ ] Given a 50-page PDF, opening it shows page 1 within 2 s, and scrolling to
      page 50 shows its content within 1 s after scrolling stops; text at
      400 % is rendered sharply (not an enlarged low-resolution image).

## Out of scope

- Text selection, search, thumbnails, printing, rotation.
- Remembering zoom level between files or sessions.

## Notes

- GUI-bound; verified manually with a synthetic 3-page and 50-page PDF.
