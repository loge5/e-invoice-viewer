---
id: 0024
title: Map every remaining EN 16931 business term
status: draft
created: 2026-11-19
depends_on: [0021]
---

## Context

After 0021 the field model covers the terms most reviewers look for, but not
all of EN 16931. Findings about the remaining terms appear only under
`Dokument`, and an invoice that uses them looks incomplete. For 1.0 every
business term of EN 16931 that an invoice carries is shown (author,
2026-11-19).

## Requirement

The field model of CII and UBL (invoices and credit notes) shows every
business term of EN 16931, with the rules of 0004–0006 and 0021 (absent
fields and empty sections left out, labels in German and English, official
English names, locations as in 0014, findings attributed by 0011).

**Added to existing sections:**

| Section | Terms |
|---|---|
| Rechnung | BT-6 VAT accounting currency code, BT-7 Value added tax point date, BT-8 Value added tax point date code, BT-23 Business process type, BT-21 Invoice note subject code (each before its note BT-22) |
| Verkäufer (BG-4) | BT-28 trading name, BT-30 legal registration identifier, BT-33 additional legal information, BT-162 address line 3, BT-39 country subdivision |
| Käufer (BG-7) | BT-45 trading name, BT-47 legal registration identifier, BT-163 address line 3, BT-54 country subdivision; buyer contact (BG-9): BT-56 contact point, BT-57 telephone, BT-58 e-mail |
| Lieferung (BG-13) | BT-165 address line 3, BT-79 country subdivision |
| Zahlung (BG-16) | BT-82 payment means text; card (BG-18): BT-87 card number, BT-88 card holder name |
| Umsatzsteuer (BG-23) | BT-121 VAT exemption reason code |
| Summen (BG-22) | BT-111 total VAT amount in accounting currency (with BT-6 as currency) |
| Position | BT-148 item gross price, BT-147 item price discount, BT-150 price base quantity unit, BT-156 buyer's item identifier, BT-157 item standard identifier, BT-158 item classification identifier (repeatable), BT-159 country of origin; item attributes (BG-32): one field per attribute, labelled with its name (BT-160) and showing its value (BT-161) |

**New sections:**

| Section (German / English) | Terms | Place |
|---|---|---|
| Zahlungsempfänger (BG-10) / Payee (BG-10) | BT-59, BT-60, BT-61 | after the buyer |
| Steuervertreter (BG-11) / Seller tax representative party (BG-11) | BT-62, BT-63, BT-64, BT-65, BT-164, BT-66, BT-67, BT-68, BT-69 | after the payee |
| Nachlass (BG-27) / Invoice line allowances (BG-27), repeatable | BT-136, BT-137, BT-138, BT-139, BT-140 | directly after its line, titled with the line, e.g. `Position 1 – Nachlass (BG-27)` |
| Zuschlag (BG-28) / Invoice line charges (BG-28), repeatable | BT-141, BT-142, BT-143, BT-144, BT-145 | after the line's allowances |

**Display rules** (in addition to 0004, 0006, 0017, 0021):

- Dates: BT-7 like BT-2.
- Amounts: BT-136, BT-137, BT-141, BT-142, BT-147, BT-148 with the invoice
  currency; BT-111 with the VAT accounting currency (BT-6).
- BT-150 like BT-130 (unit name and code).
- Codes shown raw: BT-6, BT-8, BT-21, BT-121, BT-140, BT-145, BT-159.
- Identifiers with a scheme (BT-30, BT-47, BT-60, BT-157, BT-158) like the
  electronic addresses of 0021: `<value> (<scheme>)`.
- BT-87: only what the invoice contains (EN 16931 allows only the last
  digits); shown as is.

**Coverage check**: a test lists the business terms of EN 16931 and fails if
one is neither mapped nor named as deliberately not shown. Deliberately not
shown as fields: BG-24 (BT-122 to BT-125, the supporting documents, which the
attachments section shows, 0013).

## Acceptance criteria

Sample D is described in [fixtures.md](../docs/fixtures.md): CII and UBL,
based on sample C, carrying every term of this spec at least once, including
two item attributes and one allowance and one charge on line 1; valid against
EN 16931 (confirmed once with the KoSIT validator).

- [ ] Given CII sample D in German, then:
  - section `Zahlungsempfänger (BG-10)` follows `Käufer (BG-7)` and
    `Steuervertreter (BG-11)` follows it, each with its fields in the order
    of the table;
  - `Käufer (BG-7)` contains BT-56, BT-57 and BT-58 after the address;
  - `Rechnung` contains BT-6, BT-7, BT-8 and BT-23, and BT-21 directly
    before its BT-22.
- [ ] Given CII sample D, then `Position 1 – Nachlass (BG-27)` and
      `Position 1 – Zuschlag (BG-28)` follow `Position 1` directly and come
      before `Position 2`; their amounts carry the currency.
- [ ] Given CII sample D, then `Position 1` contains BT-148, BT-147, BT-150
      with a unit name, BT-156 to BT-159, and two fields for the item
      attributes, labelled with the attribute names and showing their
      values.
- [ ] Given CII sample D, then `Summen (BG-22)` shows BT-111 with the
      currency of BT-6, and `Zahlung (BG-16)` shows BT-82, BT-87 and BT-88.
- [ ] Given UBL sample D, then the field model equals that of CII sample D
      (sections, fields, order, displays), in German and English.
- [ ] A finding about a line allowance (e.g. BR-41, amount missing) is
      attributed to its `Position 1 – Nachlass (BG-27)` section; a finding
      about the payee (e.g. BR-17, name missing) to `Zahlungsempfänger
      (BG-10)`.
- [ ] The coverage check passes: every business term BT-1 to BT-165 of EN
      16931 is mapped for CII and for UBL, except BT-122 to BT-125.
- [ ] Samples A, B, C and L give the same field model as before, and the
      time limits of 0007 still hold for sample L.
- [ ] New labels and titles are in German and English; the catalogue check
      passes.

## Out of scope

- Fields of the ZUGFeRD/Factur-X Extended profile beyond EN 16931 (0027).
- Names for the code lists shown raw.

## Notes

- German labels follow the style of the existing ones (short, the section
  gives the context) and the KoSIT visualisation as reference.
- BT-4 does not exist in EN 16931.
- Line allowances and charges are the first sections that belong to one
  line; the field model gets sections nested under a line section's place
  (they are still separate sections in the list, placed after their line).
- Sample D needs BT-6 different from BT-5 and therefore BT-111 (BR-53).

## Test list (proposed)

- T1–T4 (core, `map_all_terms.rs`): sample D CII in German: new sections and
  their order, buyer contact, document fields, line allowances and charges
  after their line, line fields with item attributes, totals and payment
  (AC1–AC4).
- T5 `ubl_and_cii_sample_d_give_same_fields`, German and English (AC5).
- T6 `findings_at_line_allowances_and_payee`: BR-41 and BR-17 variants
  (AC6).
- T7 `every_business_term_is_mapped`: the list BT-1 to BT-165 against the
  terms of the CII and UBL mappings, with BT-122 to BT-125 as the only
  exceptions (AC7).
- T8 existing tests for A, B, C, L unchanged; timing; catalogue (AC8, AC9).
- T9 `sample_d_has_no_errors` (validation) and the KoSIT check of sample D.
