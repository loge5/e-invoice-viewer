---
id: 0031
title: Other data - elements without a business term
status: implemented
created: 2026-11-20
depends_on: [0004, 0005, 0006, 0013, 0014, 0024]
---

## Context

The field model (0004 to 0006, 0024) shows every EN 16931 business term.
Content beyond them — the Extended profile of ZUGFeRD/Factur-X, the
XRechnung extension, or anything else the schema allows — is not shown, so
part of an invoice can be invisible. Split off from 0027 at its review
(2026-11-20).

A scan at the review (2026-11-20) listed every element with text that no
field shows, in the FeRD 2.5.2 samples and the committed KoSIT test
invoices:

- **Extended content**: e.g. `ram:ExchangedDocument/ram:Name`, delivery
  notes and order references per line, product classifications, advance
  payments, ship-to parties per line; FeRD Extended samples have 2 to 99
  such elements each; UBL `cac:SubInvoiceLine` (XRechnung extension).
- **Elements the app already uses** without showing them as a field: type
  codes that decide the business term (`ram:TypeCode` 50/130/916 of a
  referenced document), charge indicators, the tax scheme `VAT`
  (`ram:TypeCode`, `cac:TaxScheme/cbc:ID`), the names of item attributes
  (BT-160, used as label of BT-161), and the supporting documents listed as
  attachments (BG-24, 0013).
- **Further occurrences of a single business term**: e.g. BT-7 in every
  VAT breakdown of CII (only the first is shown), a second payment means.
- **Gaps of the field model**: BT-29, BT-46 and BT-71 given as
  `ram:GlobalID` with `schemeID` (only `ram:ID` is mapped).
- **Not business terms in EN 16931 invoices**: e.g. the project name
  `ram:SpecifiedProcuringProject/ram:Name` (required by CII next to BT-11),
  the base quantity of the gross price.

## Requirement

- **Other data**: an element with text that no field shows, and that is
  not *used* (see below), is listed as *other data*: at the end of the
  section it belongs to (the deepest section whose element contains it, as
  findings are placed, 0014), after the business terms; elements outside
  every section below the root go to a last section `Weitere Angaben` /
  `Other data`.
  - Label: the path from the section's element to the element, in local
    names, e.g. `ExchangedDocument › Name`; no business term number. The
    row shows only the last element after `… ` (`… Name`), with a button
    next to it that shows the whole path (author, 2026-11-20, after the
    first check in the app).
  - Value: the text, whitespace normalised; attributes of the element
    follow in brackets, e.g. `4012345000016 (schemeID: 0160)`; binary
    content is shown as `binary data, 12 kB`.
  - Like fields, a row has *Show in XML* (0014), and findings at its
    element are shown at the row.
- **Used elements** are not listed: elements that decide which business
  term a value is (conditions of the mapping, e.g. type codes, charge
  indicators), the tax scheme `VAT`, the names of item attributes, and the
  supporting documents of the attachment list (0013). The mapping declares
  them; there is no list of names in the code.
- **Further occurrences** of a single business term are listed only if
  their value differs from the shown one.
- **Gaps**: BT-29, BT-46 and BT-71 are also read from `ram:GlobalID`, with
  the scheme (`schemeID`) like BT-30. Gaps found while implementing are
  mapped, not listed as other data, and added to the progress notes.
- Applies to every CII and UBL invoice, whatever its profile.
- New texts are translated into all 24 languages.

## Acceptance criteria

- [x] For every committed fixture invoice (samples, KoSIT, public samples)
      and, locally, every FeRD sample: every element with text is shown as
      a field, used, or listed as other data — nothing is invisible
      (coverage check over the XML).
- [x] Given sample A with an Extended element in the document
      (`ram:ExchangedDocument/ram:Name`), one in the seller party and one
      in the first line, then each is listed at the end of the seller and
      first line section, and in the last section `Other data` for the
      document, with label, value and attributes.
- [x] Given samples A to D, then no other data is listed, except elements
      that are not business terms (CII project name in C and D, the card
      network in UBL D).
- [x] Given the committed KoSIT EN 16931 and XRechnung test invoices without
      extension, then other data contains only elements that are not
      business terms, each listed in the notes of this spec.
- [x] Given a CII invoice whose seller identifier is
      `<ram:GlobalID schemeID="0088">…`, then BT-29 shows it with the scheme;
      the same for BT-46 and BT-71.
- [x] Given a CII invoice with two VAT breakdowns and the same BT-7, then
      nothing is listed; with different dates, the second is listed.
- [x] Given a finding located at an other-data element (e.g. the `FX`
      finding of 0027 for an unused attribute's element), then it is shown
      at that row; *Show in XML* of the row selects the element.
- [x] The catalogue check passes for all languages.

## Out of scope

- Names from the FeRD technical appendix (`BT-X-…` numbers and labels): the
  appendix is a document of FeRD, not bundled; possibly a later spec.
- Attributes of shown elements that no field shows.
- Editing or exporting other data.

## Progress

- **Done**:
  - `fields/other.rs`: other data at the end of its section (document
    level in a last section `Other data`); label as path, attributes in
    brackets, binary content as size.
  - *Used* elements are derived from the mapping tables: what conditions,
    label names (BT-160), title suffixes and scheme elements read; fixed
    values declared per syntax (`fixed_values`: tax scheme `VAT`, the
    charge indicator `false` of a CII price discount, UBL document type
    `130` of BT-128); the supporting documents as a whole. Further
    occurrences of a single field with the same value are recorded while
    mapping.
  - Gaps mapped (found by the scan and the coverage test): BT-29, BT-46,
    BT-71 from `ram:GlobalID` with scheme; BT-41 and BT-56 from
    `ram:DepartmentName` (CEN counts it as the contact point, CII-SR-465); BT-29 of UBL repeatable; BG-17
    (BT-84, BT-85, BT-86) once per payment means in CII and UBL; the scheme
    of BT-128 (CII `ram:ReferenceTypeCode`, UBL `schemeID`).
  - App: rows without number (`field_name`, `field_row_text`); other data
    shows `…` and the last element, a button with a popover shows the
    whole path (`short_label`, text `Show full path` in all languages).
  - Texts `Other data` and `binary data, {size}` in all 24 languages.
  - README and CHANGELOG.
- **Verified** (2026-11-20): `other_data.rs` T1–T7 (coverage over all 400+
  committed fixtures, and locally over all FeRD samples with
  `FERD_PACKAGE`), catalogue check (T8), the full workspace tests, fmt and
  clippy. The UBL/CII comparison tests compare business terms only (other
  data differs between the syntaxes by nature).
  - P1 (author, 2026-11-20): X08 in the app; the short label with the
    full-path button fits, with a space after `…`.

## Notes

- Elements of EN 16931 invoices that are not business terms (AC4, list in
  `kosit_other_data_is_not_business_terms`): CII project name
  (`ram:SpecifiedProcuringProject/ram:Name`), base quantity of the gross price
  (`ram:GrossPriceProductTradePrice/ram:BasisQuantity`), the payer's bank
  of a CII direct debit (`ram:PayerSpecifiedDebtorFinancialInstitution`),
  UBL card network (`cac:CardAccount/cbc:NetworkID`), UBL credit account
  (`cac:CreditAccount/cbc:AccountID`), UBL order reference of a line
  (`cac:OrderLineReference/cac:OrderReference/cbc:ID`), and a second tax
  total in the invoice currency (negative test invoices).
- Review on 2026-11-20 (Claude): scanned the FeRD samples and the KoSIT
  invoices (see Context); added *used* elements, further occurrences and
  the gaps BT-29/46/71; other data placed at the end of its section instead
  of separate sections per place (no new section kinds, findings and
  *Show in XML* work as for fields); label as path; FeRD names out of
  scope; a coverage check over all fixtures.

## Test list

- T1 `nothing_is_invisible` (core): for every committed fixture invoice,
  every element with text is a field location, a used element or an
  other-data row (AC1); the same over the FeRD samples with
  `FERD_PACKAGE` (local).
- T2 `extended_elements_listed_at_their_place` (core, sample A variant)
  (AC2).
- T3 `samples_have_no_other_data` (core, samples A to D) (AC3).
- T4 `kosit_other_data_is_not_business_terms` (core): other data of the
  KoSIT invoices without extension ⊆ the list in the notes (AC4).
- T5 `party_identifiers_from_global_id` (core, CII) (AC5).
- T6 `further_occurrences_only_if_different` (core, BT-7) (AC6).
- T7 `finding_at_other_data_row` (core attribution) and the row's location
  in the XML view (`XmlLayout`) (AC7).
- T8 catalogue check (AC8).
- P1 manual: an Extended sample (e.g. X08) in the app: rows readable, *Show
  in XML* works.
