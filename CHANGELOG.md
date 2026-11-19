# Changelog

All notable changes for users of E-Invoice Viewer. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- The invoice XML is checked against its XML schema (CII D16B, UBL 2.1) like
  the official KoSIT validator does: unknown or misplaced elements and wrong
  data types are reported as findings (rule `XSD`).
- Every business term of EN 16931 is shown, including the payee, the seller's
  tax representative, the buyer's contact, card payment, item attributes,
  allowances and charges of each line, and the VAT total in the VAT
  accounting currency.
- Every push and pull request is tested on Linux and Windows.

### Changed

- XML nested more than 256 levels deep is refused with a message instead of
  using up memory.

## [0.2.1] - 2026-11-19

### Added

- Windows 10/11 (x64): a portable ZIP and an installer for the current user,
  with *Open with* for PDF and XML files.
- More invoice fields: project, contract and order references, the preceding
  invoice, electronic addresses of seller and buyer, delivery information
  with the invoicing period, SEPA direct debit, document level allowances and
  charges, the period of each line.
- Findings about these fields are shown at the field instead of under
  "Document".

### Changed

- Proposed file names for saving attachments are valid on Windows as well
  (no `< > : " | ? *`, no device names such as `CON`).

## [0.1.0] - 2026-11-18

First public release.

### Added

- PDF and invoice fields side by side for ZUGFeRD/Factur-X PDFs and CII or
  UBL XML invoices and credit notes, every value labelled with its EN 16931
  business term.
- Validation against the EN 16931 business rules and XRechnung, with the
  findings at the affected field and a summary.
- Raw XML view with search, linked with the fields in both directions.
- Attachments listed and saved.
- Drag and drop, *Open with*, recent files, remembered window size.
- German and English, following the system language, switchable in the app.
- AppImage for x86_64 Linux; `scripts/install.sh` for an installation from
  source.

[Unreleased]: https://github.com/loge5/e-invoice-viewer/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/loge5/e-invoice-viewer/compare/v0.1.0...v0.2.1
[0.1.0]: https://github.com/loge5/e-invoice-viewer/releases/tag/v0.1.0
