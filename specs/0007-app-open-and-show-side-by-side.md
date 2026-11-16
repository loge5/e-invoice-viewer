---
id: 0007
title: App - open an invoice and show PDF and fields side by side
status: implemented
created: 2026-11-16
depends_on: [0002, 0003, 0004]
---

## Context

This is the first user-visible slice: open a file, see the PDF on the left and
the structured fields on the right. The previous tool (Quba) had an unstable
layout (sidebar cut off after resizing, parts of the window left unpainted)
and always reopened the last file. The new app must feel solid from the first
version.

## Requirement

The user can open an invoice file through a file dialog or by passing its path
on the command line. The window then shows the first PDF page and the field
model side by side, with a clearly visible way to open another file and to
close the current one. Plain XML invoices show only the fields. Field values
can be selected and copied. Errors are shown as understandable messages in the
window, never as a crash. On start without a file argument, the app shows an
empty start view and does not reopen any previous file.

The field pane shows whatever field model the core produces; it shows each
field as one row with BT number, German label and display value.

## Acceptance criteria

Sample invoices are defined in [fixtures.md](../docs/fixtures.md); time limits
refer to the reference machine in [requirements](../docs/requirements.md).

- [ ] Given the app is started without arguments, then the window shows a start
      view with the title "E-Invoice Viewer", the text "Rechnung öffnen" and a
      button "Öffnen…"; no file is opened automatically.
- [ ] Given the user clicks "Öffnen…" (or presses Ctrl+O) and selects
      `RE-2026-0042.pdf` (sample A), then within 2 s the left pane shows page 1
      of the PDF, the right pane shows the sections of the field model with
      the row `BT-1 · Rechnungsnummer · RE-2026-0042`, and the window title
      contains `RE-2026-0042.pdf`.
- [ ] Given the app is started as `e-invoice-viewer /path/RE-2026-0042.xml`
      (CII sample A without PDF), then only the field pane is shown, filling
      the window, with the note "Reine XML-Rechnung – kein PDF enthalten".
- [ ] Given an invoice is shown, when the user clicks "Schließen" in the header
      bar or presses Ctrl+W, then the start view is shown and the window title
      no longer contains the file name. Given the start view, Ctrl+W closes the
      window. Ctrl+Q quits the application with all its windows.
- [ ] Given sample A is shown, when the user selects part of the BT-27 value
      with the mouse and presses Ctrl+C, then the clipboard contains the
      selected text; when the user chooses "Wert kopieren" in the context menu
      of BT-84, then the clipboard contains the raw value
      `DE02120300000000202051`.
- [ ] Error paths, each shown as an in-window message naming the file:

      | situation | result |
      |---|---|
      | PDF without embedded invoice | PDF is shown; the field pane shows "Diese PDF enthält keine elektronische Rechnung" |
      | invoice XML readable, but the PDF pages cannot be rendered | field pane is shown; the PDF pane shows "Die PDF-Seiten können nicht angezeigt werden" |
      | not a readable PDF, malformed XML, not an invoice | start view with "Die Datei <name> kann nicht als Rechnung gelesen werden" and the reason from 0002/0003; a previously shown invoice is closed |

- [ ] Layout stability: the window cannot be made smaller than 900 × 600
      logical pixels; at that size both panes, the header bar and all field
      labels are fully visible (scrollbars where needed). The divider between
      the panes can be dragged; neither pane can become narrower than 300
      logical pixels. Manual check: resize the window 10 times between the
      minimum size and maximized, and toggle maximize 5 times; no part of the
      window is ever cut off or left unpainted.

## Out of scope

- Multi-page navigation and zoom (0016), XML view (0008), validation display
  (0011), drag & drop, "Open with", recent files (0012), attachments (0013),
  saving window geometry (0015).
- Multiple invoices per window (tabs).
- Printing, text selection or search in the PDF.

## Notes

- All criteria are GUI-bound and are verified manually by running the app with
  the documented steps and the sample files; the result is recorded in the
  spec's Progress section.

## Progress

- **Done** — `core::document::open` decides what to show (PDF with fields,
  PDF without invoice, plain XML, or an error with reason); 4 tests in
  `crates/core/tests/open_document.rs`. App: `crates/app/src/` with
  `window.rs` (start view, header bar with "Öffnen…"/"Schließen", PDF and
  fields in a `GtkPaned`, minimum sizes, actions Ctrl+O/Ctrl+W/Ctrl+Q,
  one window per command-line file), `pdf_view.rs` (first page via
  Poppler), `fields_view.rs` (sections and rows, selectable values, "Wert
  kopieren" in the value's context menu), `texts.rs` (German texts, 3 unit
  tests). New dependency: `poppler-rs` 0.26 (MIT, links GPL Poppler per ADR
  0001); `rust-version` raised to 1.92.
- **Verified** — 2026-11-16: `cargo test --workspace` 59 passed;
  `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
  warnings`, architecture check and `cargo audit` clean. The app starts with
  each manual fixture without crash or GTK warnings. Manual checks M1–M7
  (start view, open PDF/XML, close/quit shortcuts, copy, error paths
  including a real Factur-X invoice, layout stability on resize) run by the
  author on 2026-11-16: all passed. Fixtures for the manual checks:
  `cargo test -p e-invoice-viewer-core --test open_document -- --ignored`
  (written to `target/manual-fixtures/`).
- **Next** — none; set to `implemented` by the author on 2026-11-16.
