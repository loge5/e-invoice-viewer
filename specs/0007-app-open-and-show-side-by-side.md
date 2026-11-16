---
id: 0007
title: App - open an invoice and show PDF and fields side by side
status: draft
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
on the command line. The window then shows the PDF pages and the field model
side by side, with a clearly visible way to open another file and to close the
current one. Plain XML invoices show only the fields. Errors are shown as
understandable messages in the window, never as a crash. On start without a
file argument, the app shows an empty start view and does not reopen any
previous file.

## Acceptance criteria

- [ ] Given the app is started without arguments, then the window shows a start
      view with the title "E-Invoice Viewer", the text "Rechnung öffnen" and a
      button "Öffnen…"; no file is opened automatically.
- [ ] Given the user clicks "Öffnen…" (or presses Ctrl+O) and selects the
      sample Factur-X PDF `RE-2026-0042.pdf`, then within 2 s the left pane
      shows page 1 of the PDF, the right pane shows the sections from 0004
      with BT-1 `RE-2026-0042`, and the window title contains
      `RE-2026-0042.pdf`.
- [ ] Given the app is started as `e-invoice-viewer /path/RE-2026-0042.xml`
      (a plain CII invoice), then only the field pane is shown, filling the
      window, with a note "Reine XML-Rechnung – kein PDF enthalten".
- [ ] Given a multi-page PDF (3 pages), the user can scroll through all pages
      in the left pane, and zoom in and out (Ctrl++ / Ctrl+- / Ctrl+0 for
      "fit width").
- [ ] Given an invoice is shown, when the user chooses "Schließen" (header bar
      button or Ctrl+W), then the start view is shown again; the window title
      no longer contains the file name.
- [ ] Error paths, each shown as an in-window message naming the file:
      - PDF without invoice → the PDF is still shown, and the field pane shows
        "Diese PDF enthält keine elektronische Rechnung";
      - not a readable PDF / malformed XML / not an invoice → the start view
        is shown with the message "Die Datei <name> kann nicht als Rechnung
        gelesen werden" and the reason from 0002/0003; a previously shown
        invoice is closed.
- [ ] Layout stability: the window cannot be made smaller than 900 × 600
      logical pixels; at that size both panes, the header bar and all field
      labels are fully visible (scrollbars where needed). The divider between
      the panes can be dragged; neither pane can become narrower than 300
      logical pixels. Manual check: resize the window 10 times between the
      minimum size and maximized, and toggle maximize 5 times; no part of the
      window is ever cut off or left unpainted.

## Out of scope

- XML view (0008), validation display (0011), drag & drop, "Open with",
  recent files (0012), attachments (0013), saving window geometry (0015).
- Multiple invoices per window (tabs).
- Printing, text selection or search in the PDF.

## Notes

- Verification: criteria 1 to 7 are GUI-bound and are verified manually by
  running the app with the documented steps and the synthetic sample files;
  the result is recorded in the spec's Progress section.
