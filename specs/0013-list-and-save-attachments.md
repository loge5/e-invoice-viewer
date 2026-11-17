---
id: 0013
title: List and save attachments of an invoice
status: reviewed
created: 2026-11-16
depends_on: [0002, 0007]
---

## Context

Invoices often carry supporting documents such as delivery notes or
timesheets. They can be embedded in two places: as further files in the
ZUGFeRD PDF, or inside the invoice XML as additional supporting documents
(BG-24 with embedded binary content, BT-125). Users need to see and save them.

## Requirement

For an opened document, the field pane has a section "Anhänge" after the last
invoice section and before the validation summary. It lists all attachments
from both sources:

- **PDF:** every file embedded in the PDF except the one 0002 selected as the
  invoice XML. Further XML files (e.g. an `xrechnung.xml` next to the
  selected `factur-x.xml`) are listed. A PDF without an invoice XML (0007)
  still lists its embedded files.
- **Invoice (BG-24):** every additional supporting document of the invoice
  XML, with embedded content (BT-125) or with an external address (BT-124).

Each entry shows:

- the file name;
- the source: `PDF` or `Rechnung (BG-24)`;
- the MIME type, or `unbekannt` if none is given;
- the size of the decoded content, the way the GNOME file manager shows sizes
  (decimal units, German decimal comma), e.g. `24,6 kB` for 24 576 bytes;
- the description, if one is given (BT-122 for BG-24, the file description
  for PDF files).

Without attachments the section shows only `Keine Anhänge`.

The user can save an attachment through a save dialog. The dialog proposes a
safe file name: the attachment's name without any directory part (`/` and
`\`), control characters or characters that change the text direction. If no
name is given or nothing is left, it proposes `Anhang-<n>`, where n is the
entry's position in the list. The file is written only where the user chooses
in the dialog. Cancelling writes nothing. If writing fails, the reason is
shown and no partial file is left behind.

An attachment that cannot be decoded, or whose decoded content is larger than
64 MB (the limit of 0002), is listed with the reason instead of a size, and
without a save action.

The app does not open, preview or execute attachments, and it never opens or
fetches external addresses (BT-124).

## Acceptance criteria

- [ ] Given a Factur-X PDF that embeds `factur-x.xml` and `lieferschein.pdf`
      (24 576 bytes, type `application/pdf`), then the section "Anhänge"
      shows exactly one entry: `lieferschein.pdf`, source `PDF`, type
      `application/pdf`, size `24,6 kB`.
  - If `lieferschein.pdf` carries the description `Lieferschein 4711`, it is
    shown with the entry.
  - Given the same PDF without `factur-x.xml`, then the PDF and the note of
    0007 are shown, and "Anhänge" lists `lieferschein.pdf`.
  - Given a PDF that embeds `factur-x.xml` and `xrechnung.xml`, then
    `xrechnung.xml` is listed and `factur-x.xml` is not.
- [ ] Given CII sample A with a BG-24 entry with BT-122 `Stundennachweis` and
      embedded content with file name `stunden.csv` and MIME type `text/csv`,
      then the list shows `stunden.csv`, source `Rechnung (BG-24)`, type
      `text/csv` and the description `Stundennachweis`.
- [ ] Given an invoice without attachments, then the section "Anhänge" shows
      only `Keine Anhänge`.
- [ ] Given the user saves `stunden.csv`, then the save dialog proposes the
      name `stunden.csv`, and the saved file is byte-identical to the decoded
      embedded content. Cancelling the dialog writes no file.
- [ ] Proposed save names:

      | attachment name | proposed name |
      |---|---|
      | `../../.bashrc` | `.bashrc` |
      | `C:\Temp\stunden.csv` | `stunden.csv` |
      | `rechnung` + U+202E + `fdp.exe` | `rechnungfdp.exe` |
      | `..`, or no name, as the 2nd entry | `Anhang-2` |

  - The list shows the name without control and text-direction characters
    as well.
- [ ] Given a BG-24 entry with only an external address
      `https://example.com/stunden.csv` (BT-124) and no embedded content,
      then it is listed with the address as text and without a save action.
      The address is not opened or fetched.
- [ ] Given a BG-24 entry whose embedded content is not valid Base64, or a
      PDF attachment that cannot be decompressed, then the entry is listed
      with the reason `Inhalt nicht lesbar` and without a save action.
      Opening the invoice still succeeds.
- [ ] Given an attachment whose decoded content exceeds 64 MB, then it is
      listed with `Zu groß (über 64 MB)` and without a save action.
- [ ] Given an attachment without MIME type, then its type reads
      `unbekannt`.
- [ ] Given saving fails (e.g. the chosen folder is not writable), then the
      message `Der Anhang <name> konnte nicht gespeichert werden: <reason>`
      is shown, no partial file remains, and the invoice stays shown.

## Out of scope

- Opening attachments with another application, previews of attachments.
- Saving all attachments at once.
- Attachments of UBL invoices: the app cannot open UBL before 0006. Their
  BG-24 entries are listed like CII entries once 0006 is implemented.
- Validating or scanning attachment content.

## Notes

- Listing, name sanitising, size formatting and decoding are core logic and
  unit-tested; the list and the save dialog are verified manually.
- Test PDFs with attachments are built synthetically, like the PDFs of 0002.
- ZUGFeRD 1.0 files are rejected by 0003, so they never show attachments.
- Review on 2026-11-17: the author decided on the section placement before
  the validation summary, listing attachments of PDFs without an invoice,
  the GNOME size format and showing descriptions of PDF attachments.
