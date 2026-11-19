---
id: 0028
title: Accessibility - keyboard and screen reader review
status: draft
created: 2026-11-19
depends_on: [0007, 0011, 0014, 0016, 0019]
---

## Context

GTK and libadwaita provide accessibility, and several specs already require
accessible names and keyboard use for single features. A complete review
with a screen reader and keyboard only has not been done. For 1.0 the app
should be usable without a mouse and with a screen reader (author,
2026-11-19).

## Requirement

- **Keyboard only**: every function can be reached and used with the
  keyboard: open, recent files, main menu and language, view switch, fields
  and their menu (copy value, show in XML), findings, XML view with search and
  "show in fields", PDF pages and zoom, attachments (save), close and quit.
  The focus is always visible and moves in a sensible order; no keyboard
  trap.
- **Screen reader** (Orca on Linux, NVDA on Windows): every control has a
  name; icon-only buttons are named by their action; a field is read as label
  and value (and its business term); the status line is read when it changes
  after the validation; findings are read with severity, rule and message;
  sections are recognisable as groups with their title.
- **Visual**: texts and icons have enough contrast in light and dark style
  (WCAG 2.1 AA, 4.5:1 for text); severity is never shown by colour alone
  (icon or word as well); the app stays usable at 200 % text scaling.
- Problems found in the review are fixed, or recorded in the spec with a
  reason if they are limitations of GTK/libadwaita.

## Acceptance criteria

- [ ] An automated test walks the widget tree of a window showing sample C
      and fails for an interactive widget without an accessible name or
      without being reachable by the keyboard (focusable).
- [ ] A test checks the contrast of the app's own colours (CSS of
      `load_css`) in light and dark style against 4.5:1.
- [ ] Manual check by the author with Orca: open sample C with the keyboard
      only, read the status line, move through the fields of `Lieferung
      (BG-13)`, open the field menu, jump to the XML, search for `BT-73`,
      save the attachment of the Factur-X sample, switch the language.
- [ ] The same with NVDA in the Windows VM (a shorter path: open, status,
      fields, save attachment).
- [ ] At 200 % text scaling (GNOME "Large text" or Windows text size) all
      texts are fully visible and the window stays usable at its minimum
      size.
- [ ] Every problem found is fixed or listed under Notes with its reason.

## Out of scope

- Accessibility of the PDF content itself (Poppler renders pages as images;
  the text of the PDF is not exposed).
- High-contrast themes beyond what libadwaita provides.

## Notes

- The automated widget tree test needs a display; it runs under a virtual
  display (`xvfb-run`) in the CI Linux job of 0026 and is skipped where no
  display is available.
- The review is done in two rounds: first an automated pass and a keyboard
  pass by Claude (through AT-SPI in a container), then the manual screen
  reader checks by the author.

## Test list (proposed)

- T1 `every_control_is_named_and_focusable` (app, needs a display; runs under
  `xvfb-run` in the CI Linux job): window with sample C, walk the widget
  tree (AC1).
- T2 `own_colours_have_enough_contrast`: the colours in the app's CSS against
  the libadwaita backgrounds, light and dark, 4.5:1 (AC2).
- P1 keyboard pass through AT-SPI in a container (Claude), findings fixed.
- Manual: Orca, NVDA, 200 % text scaling (author) (AC3–AC5).
