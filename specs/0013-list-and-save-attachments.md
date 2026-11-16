---
id: 0013
title: List and save attachments of an invoice
status: draft
created: 2026-11-16
depends_on: [0002, 0007]
---

## Context

Invoices often carry supporting documents such as delivery notes or
timesheets. They can be embedded in two places: as further files in the
ZUGFeRD PDF, or inside the invoice XML as additional supporting documents
(BG-24 with embedded binary content, BT-125). Users need to see and save them.

## Requirement

For an opened invoice, the app lists all attachments from both sources, each
with file name, source, MIME type and size, and lets the user save a selected
attachment through a save dialog. The invoice XML itself is not listed. The
app does not open or execute attachments itself.

## Acceptance criteria

- [ ] Given a Factur-X PDF that embeds `factur-x.xml` and `lieferschein.pdf`
      (24 KB), then the attachment list shows exactly one entry:
      `lieferschein.pdf`, source `PDF`, type `application/pdf`, size `24 KB`.
- [ ] Given an invoice XML with a BG-24 entry with BT-122 `Stundennachweis`,
      an embedded binary object with file name `stunden.csv` and MIME type
      `text/csv`, then the list shows `stunden.csv`, source `Rechnung (BG-24)`,
      type `text/csv`, and the description `Stundennachweis`.
- [ ] Given an invoice without attachments, then the attachment area shows
      `Keine Anhänge` and takes no more than one line of space.
- [ ] Given the user saves `stunden.csv`, then the save dialog proposes the
      name `stunden.csv` and the saved file is byte-identical to the decoded
      embedded content.
- [ ] Given an attachment whose name contains a path (`../../.bashrc`), then
      the proposed save name is `.bashrc` without any directory part, and the
      file is only written where the user chooses in the dialog.
- [ ] Given a BG-24 entry with only an external URL (BT-124) and no embedded
      content, then it is listed with the URL as text and without a save
      action; the URL is not opened or fetched.

## Out of scope

- Opening attachments with another application, previews of attachments.
- Attachments of ZUGFeRD 1.0 files.

## Notes

- Listing is core logic (unit-tested); the list widget and the save dialog are
  verified manually.
