---
id: 0002
title: Extract the invoice XML from a ZUGFeRD / Factur-X PDF
status: implemented
created: 2026-11-16
depends_on: []
---

## Context

A ZUGFeRD / Factur-X invoice is a PDF/A-3 file with the structured invoice
embedded as an XML attachment. Everything the viewer shows besides the PDF
pages (fields, validation, XML view) needs that XML. Users also receive PDFs
that are not e-invoices at all, or broken files, and must get a clear answer
instead of a crash.

## Requirement

Given the bytes of a PDF file, the system returns the embedded invoice XML, or
a specific, understandable error if there is none or the file cannot be read.
It also reports which embedded file name the XML came from.

The invoice XML is the embedded file with one of these names, compared
case-insensitively. If several are present, the first one in this order wins:

1. `factur-x.xml`
2. `xrechnung.xml`
3. `zugferd-invoice.xml`

Embedded XML files with any other name are not treated as the invoice
(standard-conformant behaviour). If no invoice is found, the error lists the
names of the embedded files that are present, so the user can see what the
PDF contains.

## Acceptance criteria

- [ ] Given a PDF that embeds exactly one of the following files, when it is
      read, then the result is that file's content, byte-identical, and the
      source name as stored in the PDF:

      | embedded file name | returned source name |
      |---|---|
      | `factur-x.xml` | `factur-x.xml` |
      | `Factur-X.xml` | `Factur-X.xml` |
      | `xrechnung.xml` | `xrechnung.xml` |
      | `ZUGFeRD-invoice.xml` | `ZUGFeRD-invoice.xml` |

- [ ] Given a PDF that embeds `xrechnung.xml`, `factur-x.xml` and
      `lieferschein.pdf`, when it is read, then the content of `factur-x.xml`
      is returned with source name `factur-x.xml`.
- [ ] Given a PDF whose embedded `factur-x.xml` is stored as a compressed
      (Flate-encoded) stream, when it is read, then the decompressed XML is
      returned, byte-identical to the original file before embedding.
- [ ] Given a valid PDF without any embedded file (e.g. a plain scanned
      invoice), when it is read, then the error is "no embedded invoice" with
      an empty list of embedded files.
- [ ] Given a valid PDF whose only embedded file is `rechnung.xml`, when it is
      read, then the error is "no embedded invoice" and the list of embedded
      files is `["rechnung.xml"]`.
- [ ] Given a PDF with an embedded `factur-x.xml` of 0 bytes, when it is read,
      then the error is "embedded invoice is empty" with source name
      `factur-x.xml`.
- [ ] Given each of these inputs: the bytes of a PNG image, a valid PDF
      truncated to its first 1024 bytes, a password-protected PDF, then the
      error is "not a readable PDF", and the call returns within 2 s without
      crashing.

## Out of scope

- Detecting syntax or profile of the XML (0003).
- Listing or saving other attachments (0013).
- Validating the PDF itself (PDF/A conformance).
- Opening password-protected PDFs (asking for a password).

## Notes

- Test fixtures must be synthetic or taken from Apache-2.0 licensed sample
  sets (e.g. Mustang project test resources), never real invoices.
- `zugferd-invoice.xml` (in any letter case) is the name used by ZUGFeRD 1.0
  and 2.0; ZUGFeRD 1.0 content is rejected later by 0003.

## Progress

- **Done** — `e_invoice_viewer_core::pdf::extract_invoice_xml` implemented
  (`crates/core/src/pdf.rs`); 13 tests in
  `crates/core/tests/extract_invoice_xml.rs`, synthetic PDFs built in
  `crates/core/tests/common/mod.rs`. New dependency: `lopdf` 0.47 (MIT).
- **Verified** — 2026-11-16: `cargo test --workspace` 13 passed;
  `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
  warnings`, architecture check and `cargo audit` clean. Manual smoke run
  against two real Factur-X invoices (extracted `factur-x.xml`, 7142 and
  55871 bytes) and 42 non-invoice PDFs (all "no embedded invoice", no crash).
- **Next** — none; set to `implemented` by the author on 2026-11-16.
