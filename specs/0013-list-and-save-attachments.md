---
id: 0013
title: List and save attachments of an invoice
status: implemented
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

- [x] Given a Factur-X PDF that embeds `factur-x.xml` and `lieferschein.pdf`
      (24 576 bytes, type `application/pdf`), then the section "Anhänge"
      shows exactly one entry: `lieferschein.pdf`, source `PDF`, type
      `application/pdf`, size `24,6 kB`.
  - If `lieferschein.pdf` carries the description `Lieferschein 4711`, it is
    shown with the entry.
  - Given the same PDF without `factur-x.xml`, then the PDF and the note of
    0007 are shown, and "Anhänge" lists `lieferschein.pdf`.
  - Given a PDF that embeds `factur-x.xml` and `xrechnung.xml`, then
    `xrechnung.xml` is listed and `factur-x.xml` is not.
- [x] Given CII sample A with a BG-24 entry with BT-122 `Stundennachweis` and
      embedded content with file name `stunden.csv` and MIME type `text/csv`,
      then the list shows `stunden.csv`, source `Rechnung (BG-24)`, type
      `text/csv` and the description `Stundennachweis`.
- [x] Given an invoice without attachments, then the section "Anhänge" shows
      only `Keine Anhänge`.
- [x] Given the user saves `stunden.csv`, then the save dialog proposes the
      name `stunden.csv`, and the saved file is byte-identical to the decoded
      embedded content. Cancelling the dialog writes no file.
- [x] Proposed save names:

      | attachment name | proposed name |
      |---|---|
      | `../../.bashrc` | `.bashrc` |
      | `C:\Temp\stunden.csv` | `stunden.csv` |
      | `rechnung` + U+202E + `fdp.exe` | `rechnungfdp.exe` |
      | `..`, or no name, as the 2nd entry | `Anhang-2` |

  - The list shows the name without control and text-direction characters
    as well.
- [x] Given a BG-24 entry with only an external address
      `https://example.com/stunden.csv` (BT-124) and no embedded content,
      then it is listed with the address as text and without a save action.
      The address is not opened or fetched.
- [x] Given a BG-24 entry whose embedded content is not valid Base64, or a
      PDF attachment that cannot be decompressed, then the entry is listed
      with the reason `Inhalt nicht lesbar` and without a save action.
      Opening the invoice still succeeds.
- [x] Given an attachment whose decoded content exceeds 64 MB, then it is
      listed with `Zu groß (über 64 MB)` and without a save action.
- [x] Given an attachment without MIME type, then its type reads
      `unbekannt`.
- [x] Given saving fails (e.g. the chosen folder is not writable), then the
      message `Der Anhang <name> konnte nicht gespeichert werden: <reason>`
      is shown, no partial file remains, and the invoice stays shown.

## Out of scope

- Opening attachments with another application, previews of attachments.
- Saving all attachments at once.
- Validating or scanning attachment content.

## Notes

- Listing, name sanitising, size formatting and decoding are core logic and
  unit-tested; the list and the save dialog are verified manually.
- Test PDFs with attachments are built synthetically, like the PDFs of 0002.
- ZUGFeRD 1.0 files are rejected by 0003, so they never show attachments.
- Review on 2026-11-17: the author decided on the section placement before
  the validation summary, listing attachments of PDFs without an invoice,
  the GNOME size format and showing descriptions of PDF attachments.
- Corrections (2026-11-18, during implementation):
  - In EN 16931 the description of a supporting document is BT-123; BT-122
    is its reference. The list shows BT-123, or BT-122 if there is no
    description, so AC2 holds as written.
  - 0006 was implemented first, so UBL BG-24 entries are listed too; the
    out-of-scope item for UBL is dropped.
  - BG-24 entries with neither embedded content nor an external address are
    not listed (the requirement lists only those two kinds).

## Progress

- **Done**:
  - Core `attachments.rs`: `list(bytes)` returns the PDF's embedded files
    (except the invoice XML chosen by 0002), then the BG-24 entries of the
    invoice XML (CII `AdditionalReferencedDocument` with type code 916, UBL
    `AdditionalDocumentReference` except type code 130). Each has name,
    source, MIME type, description and content (embedded, external address,
    unreadable, too large). `display_name` and `proposed_file_name` clean
    the name; base64 is decoded by an own decoder (no new dependency).
  - `pdf.rs` reads description (`/Desc`) and MIME type (`/Subtype`) and
    tells unreadable from too large. lopdf decodes damaged Flate streams
    leniently (truncated or empty, no error); a mismatch with the declared
    size (`/Params /Size`) or an empty result from a non-empty stream is
    treated as unreadable. A damaged stream without a declared size whose
    damage lopdf hides cannot be detected without a stricter decoder.
  - App: section "Anhänge" (`attachments_view.rs`) before the validation
    summary, also below the note of a PDF without invoice. Names and
    descriptions are shown as plain text, never as markup. The save button
    opens a save dialog with the proposed name; the file is written via
    temporary file and rename; a failure shows the message as a note.
- **Verified** (2026-11-18):
  - `cargo test --workspace`: 152 passed; new: 12 tests in
    `list_attachments.rs`, a base64 unit test, `size_texts` and
    `attachment_texts`.
  - `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --
    -D warnings` are clean.
  - Smoke run with the samples in `target/manual-0013/` (written by `cargo
    test -p e-invoice-viewer-core --test list_attachments -- --ignored
    write_manual_samples`), read through AT-SPI: the attachment of the PDF,
    the PDF without invoice with note and list, the three BG-24 entries
    (size, external address, `Inhalt nicht lesbar`) and `Keine Anhänge`.
- **Manually verified** by the author (2026-11-18):
  - AC1/AC3: `a-mit-lieferschein.pdf` and `a.xml` list the attachments as
    expected. AC4/AC5: saving `stunden.csv` works. AC10: saving into a
    write-protected folder shows the message.
  - The manual samples now show text on the page and embed a real PDF as
    `lieferschein.pdf`; before, the page was empty, which looked like an
    error.
