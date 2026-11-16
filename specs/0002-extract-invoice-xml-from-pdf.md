---
id: 0002
title: Extract the invoice XML from a ZUGFeRD / Factur-X PDF
status: draft
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

The invoice XML is the embedded file with one of these names, checked in this
order (case-insensitive): `factur-x.xml`, `xrechnung.xml`,
`zugferd-invoice.xml`, `ZUGFeRD-invoice.xml`.

## Acceptance criteria

- [ ] Given a Factur-X PDF with an embedded `factur-x.xml` whose content starts
      with `<?xml version="1.0" encoding="UTF-8"?><rsm:CrossIndustryInvoice`,
      when it is read, then the result is that exact XML (byte-identical) and
      the source name `factur-x.xml`.
- [ ] Given a ZUGFeRD 1.0 PDF with an embedded `ZUGFeRD-invoice.xml`, when it
      is read, then its XML is returned with source name `ZUGFeRD-invoice.xml`.
- [ ] Given a ZUGFeRD PDF with an embedded `xrechnung.xml`, when it is read,
      then its XML is returned with source name `xrechnung.xml`.
- [ ] Given a PDF that embeds both `factur-x.xml` and `lieferschein.pdf`, when
      it is read, then the XML of `factur-x.xml` is returned (other attachments
      are ignored by this function).
- [ ] Given a valid PDF without any embedded file (e.g. a plain scanned
      invoice), when it is read, then the error is "no embedded invoice" and
      no XML is returned.
- [ ] Given a file that is not a PDF (the bytes of a PNG image) or a PDF
      truncated to its first 1024 bytes, when it is read, then the error is
      "not a readable PDF", and the process does not crash or hang (returns
      within 2 s).
- [ ] Given a PDF whose embedded `factur-x.xml` is compressed in the file, when
      it is read, then the decompressed XML is returned.

## Out of scope

- Detecting syntax or profile of the XML (0003).
- Listing or saving other attachments (0013).
- Validating the PDF itself (PDF/A conformance).
- Encrypted / password-protected PDFs: they return "not a readable PDF".

## Notes

- Test fixtures must be synthetic or taken from Apache-2.0 licensed sample
  sets (e.g. Mustang project test resources), never real invoices.
