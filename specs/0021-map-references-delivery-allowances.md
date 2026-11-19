---
id: 0021
title: Map references, delivery and invoicing period, allowances and charges
status: reviewed
created: 2026-11-19
depends_on: [0004, 0005, 0006, 0011, 0017]
---

## Context

Specs 0004–0006 map a chosen set of business terms; any other term is
skipped without error. On 2026-11-19 the author noticed that the invoicing
period (`ram:BillingSpecifiedPeriod`, BG-14) is not shown. For German
invoices the date or period of the supply is a mandatory statement
(§ 14 UStG), so a reviewer looks for it first. A review of the gaps found
further terms that invoice reviewers need:

- the delivery date and the delivery address;
- the period of a single line;
- document-level allowances and charges, which make up the totals BT-107 and
  BT-108 that are shown today without their parts;
- the references to project, contract, order and a preceding invoice (for
  credit notes and corrections);
- the electronic addresses of seller and buyer, mandatory in XRechnung;
- the SEPA direct debit details.

Findings about these terms appear today only in the summary under
`Dokument`, because no field exists for them.

## Requirement

The field model of CII and UBL invoices contains the business terms below,
with the rules of 0004–0006 (absent fields and empty sections are left out,
sub-groups appear in their parent's section, labels in German and English as
in 0017, locations as in 0014), for UBL invoices and credit notes alike. Findings about these terms
are attributed to their fields by the rules of 0011, without changes to those
rules.

**New and extended sections**, in this display order (new parts in bold):

| Section (German / English title) | Fields |
|---|---|
| Rechnung / Invoice | BT-1, BT-2, BT-3, BT-5, BT-9, BT-10, **BT-11, BT-12, BT-13, BT-14, BT-15, BT-16, BT-17, BT-18, BT-19**, BT-24, BT-22 (repeatable) |
| **Vorausgegangene Rechnung (BG-3) / Preceding invoice reference (BG-3)**, repeatable | BT-25, BT-26 |
| Verkäufer (BG-4) / Seller (BG-4) | BT-27, BT-29, BT-31, BT-32, **BT-34**, BT-35 … BT-43 as today |
| Käufer (BG-7) / Buyer (BG-7) | BT-44, BT-46, BT-48, **BT-49**, BT-50 … BT-55 as today |
| **Lieferung (BG-13) / Delivery information (BG-13)** | BT-70, BT-71, BT-72, BT-73, BT-74, BT-75, BT-76, BT-78, BT-77, BT-80 |
| Position / Invoice line (0005, 0006) | as today, then **BT-132** after BT-128, and **BT-134, BT-135** at the end |
| Zahlung (BG-16) / Payment instructions (BG-16) | BT-81, BT-83, BT-84, BT-85, BT-86, **BT-89, BT-90, BT-91**, BT-20 |
| **Nachlass (BG-20) / Document level allowances (BG-20)**, repeatable | BT-92, BT-93, BT-94, BT-95, BT-96, BT-97, BT-98 |
| **Zuschlag (BG-21) / Document level charges (BG-21)**, repeatable | BT-99, BT-100, BT-101, BT-102, BT-103, BT-104, BT-105 |
| Umsatzsteuer (BG-23), Summen (BG-22) | as today |

The new delivery section comes after the buyer and before the first line;
allowances and charges come after the payment section and before the VAT
breakdown. The invoicing period (BG-14) and the delivery address (BG-15) are
sub-groups of BG-13 and appear in its section, also in CII, where the period
is part of the settlement and not of the delivery.

**Where the terms overlap with existing ones:**

- In UBL, the creditor identifier (BT-90) is a party identification of the
  seller with the scheme `SEPA`. It is shown only as BT-90, never as the
  seller identifier (BT-29).
- The invoiced object identifier (BT-18) and the tender or lot reference
  (BT-17) are document references; they are not listed as attachments
  (0013).
- The electronic addresses are a separate element from the contact's e-mail
  address (BT-43); both are shown when both are present.

**Labels** (English: the official EN 16931 names; German: short, because the
section names the context, like the existing labels):

| Term | English | German |
|---|---|---|
| BT-11 | Project reference | Projektreferenz |
| BT-12 | Contract reference | Vertragsnummer |
| BT-13 | Purchase order reference | Bestellnummer |
| BT-14 | Sales order reference | Auftragsnummer |
| BT-15 | Receiving advice reference | Wareneingangsmeldung |
| BT-16 | Despatch advice reference | Versandanzeige |
| BT-17 | Tender or lot reference | Vergabe- oder Losnummer |
| BT-18 | Invoiced object identifier | Objektkennung |
| BT-19 | Buyer accounting reference | Buchungsreferenz |
| BT-25 | Preceding Invoice reference | Rechnungsnummer |
| BT-26 | Preceding Invoice issue date | Rechnungsdatum |
| BT-34 | Seller electronic address | Elektronische Adresse |
| BT-49 | Buyer electronic address | Elektronische Adresse |
| BT-70 | Deliver to party name | Empfänger |
| BT-71 | Deliver to location identifier | Kennung des Lieferorts |
| BT-72 | Actual delivery date | Lieferdatum |
| BT-73 | Invoicing period start date | Rechnungszeitraum von |
| BT-74 | Invoicing period end date | Rechnungszeitraum bis |
| BT-75 | Deliver to address line 1 | Straße |
| BT-76 | Deliver to address line 2 | Adresszusatz |
| BT-77 | Deliver to city | Ort |
| BT-78 | Deliver to post code | PLZ |
| BT-80 | Deliver to country code | Land |
| BT-89 | Mandate reference identifier | Mandatsreferenz |
| BT-90 | Bank assigned creditor identifier | Gläubiger-ID |
| BT-91 | Debited account identifier | Belastetes Konto |
| BT-92 | Document level allowance amount | Betrag |
| BT-93 | Document level allowance base amount | Grundbetrag |
| BT-94 | Document level allowance percentage | Prozentsatz |
| BT-95 | Document level allowance VAT category code | Steuerkategorie |
| BT-96 | Document level allowance VAT rate | Steuersatz |
| BT-97 | Document level allowance reason | Grund |
| BT-98 | Document level allowance reason code | Grund (Code) |
| BT-99 … BT-105 | the same as BT-92 … BT-98 with "charge" | the same as BT-92 … BT-98 |
| BT-132 | Referenced purchase order line reference | Bestellposition |
| BT-134 | Invoice line period start date | Zeitraum von |
| BT-135 | Invoice line period end date | Zeitraum bis |

The German labels for the address lines match the existing seller and buyer
labels (e.g. `Straße`, `PLZ`, `Land`); where the existing labels differ from
the table, the existing ones win.

**Display rules** (in addition to 0004, 0006 and 0017):

- Dates: BT-26, BT-72, BT-73, BT-74, BT-134 and BT-135 like BT-2.
- Amounts: BT-92, BT-93, BT-99 and BT-100 like BT-106, with the invoice
  currency.
- Percentages and rates (BT-94, BT-96, BT-101, BT-103) are shown raw, like
  BT-119.
- Codes: BT-95 and BT-102 like BT-118 (`<name> (<code>)`); reason codes
  BT-98 and BT-105 raw.
- Electronic addresses BT-34 and BT-49: the address followed by its scheme
  in brackets, e.g. `rechnung@muster.example (EM)`, `04011000-12345-34
  (0204)`; without a scheme, the address alone.
- BT-91 like BT-84 (IBAN in groups of four).

## Acceptance criteria

The new sample C is described in [fixtures.md](../docs/fixtures.md): a
synthetic invoice in CII and UBL, based on sample A, with: BT-11 `PRJ-7`,
BT-12 `V-2026-03`, BT-13 `PO-4711`, BT-18 `OBJ-1`, a preceding invoice `RE-2026-0017` of
`2026-10-15`, seller electronic address `rechnung@muster.example` (scheme
`EM`), buyer electronic address `04011000-12345-34` (scheme `0204`),
delivery to `Lager Nord`, `Industriestraße 5`, `20457`, `Hamburg`, `DE`, with
delivery date `2026-11-03` and invoicing period `2026-11-01` to
`2026-11-30`, SEPA direct debit (BT-81 `59`, no BT-84 to BT-86) with mandate
`MANDAT-0815`, creditor identifier `DE98ZZZ09999999999` and debited account
`DE02120300000000202051`, one allowance (`50.00 EUR`, reason `Rabatt`,
category `S`, rate `19`), one charge (`10.00 EUR`, reason `Fracht`, category
`S`, rate `19`), and line 1 with order line reference `1` and period
`2026-11-01` to `2026-11-15`. Its totals follow from these: BT-107
`50.00`, BT-108 `10.00`, BT-109 `1010.00`, BT-110 `191.90`, BT-112 and
BT-115 `1201.90`, one VAT breakdown with base `1010.00`. It has no error
under EN 16931, confirmed once with the KoSIT validator.

CII and UBL sample A both carry the electronic addresses
(`rechnung@muster.example` and `einkauf@beispiel.example`, scheme `EM`); the
expectations of earlier specs that list the seller's or buyer's fields of
sample A are extended by BT-34 and BT-49.

- [ ] Given CII sample C in German, then the section `Lieferung (BG-13)`
      comes after `Käufer (BG-7)` and before `Position 1`, and contains in
      this order: BT-70 `Lager Nord`, BT-72 `03.11.2026`, BT-73 `01.11.2026`,
      BT-74 `30.11.2026`, BT-75 `Industriestraße 5`, BT-78 `20457`, BT-77
      `Hamburg`, BT-80 `DE`. In English, the section is `Delivery information
      (BG-13)`, BT-73 reads `Invoicing period start date` with `2026-11-01`.
- [ ] Given CII sample A extended by only an invoicing period (no other
      delivery information), then the section `Lieferung (BG-13)` contains
      exactly BT-73 and BT-74.
- [ ] Given CII sample C in German, then:
  - section `Rechnung` contains BT-11 `PRJ-7`, BT-12 `V-2026-03` and BT-13
    `PO-4711` after BT-10 and before BT-24;
  - section `Vorausgegangene Rechnung (BG-3)` follows `Rechnung` and
    contains BT-25 `RE-2026-0017` and BT-26 `15.10.2026`;
  - section `Verkäufer (BG-4)` contains BT-34 `rechnung@muster.example (EM)`
    between BT-31 and BT-35; section `Käufer (BG-7)` contains BT-49
    `04011000-12345-34 (0204)` directly before BT-50;
  - BT-18 `OBJ-1` is in section `Rechnung`, and the attachments section
    does not list it.
- [ ] Given CII sample C in German, then section `Zahlung (BG-16)` contains
      BT-81 display `SEPA-Lastschrift (59)`, BT-89 `MANDAT-0815`, BT-90
      `DE98ZZZ09999999999` and BT-91 `DE02 1203 0000 0000 2020 51`, and
      no BT-84.
  - Given UBL sample C, section `Verkäufer (BG-4)` contains no BT-29 with
    `DE98ZZZ09999999999`.
- [ ] Given CII sample C in German, then after `Zahlung (BG-16)` and before
      `Umsatzsteuer (BG-23)` come section `Nachlass (BG-20)` with BT-92
      `50,00 EUR`, BT-95 `Normalsatz (S)`, BT-96 `19`, BT-97 `Rabatt`, and
      section `Zuschlag (BG-21)` with BT-99 `10,00 EUR`, BT-102
      `Normalsatz (S)`, BT-103 `19`, BT-104 `Fracht`. In English, the amounts
      read `50.00 EUR` and `10.00 EUR`. Section `Summen (BG-22)` shows BT-107
      `50,00 EUR`, BT-108 `10,00 EUR`, BT-109 `1.010,00 EUR` and BT-112
      `1.201,90 EUR`.
  - Given sample C with a second allowance, there are two `Nachlass (BG-20)`
    sections in document order.
- [ ] Given CII sample C, then section `Position 1` contains BT-132 `1` after
      BT-128, and BT-134 `01.11.2026` and BT-135 `15.11.2026` as its last
      fields.
- [ ] Given UBL sample C, then the field model has the same sections, the
      same fields in the same order with the same displays as CII sample C
      (field locations differ).
- [ ] Given sample C (CII and UBL) with BT-74 `2026-10-31` before BT-73
      `2026-11-01`, then the finding BR-29 is attributed to the field BT-73
      or BT-74 in `Lieferung (BG-13)`, not to the document.
  - Given sample C with an allowance without BT-97 and BT-98, then the
    finding BR-33 is attributed to its `Nachlass (BG-20)` section.
- [ ] Every new field's location selects exactly the element its raw value
      was read from (checked for BT-73, BT-91, BT-92 of the second allowance
      and BT-135 in CII and UBL), and the XML view (0014) shows each new
      field's name for its element.
- [ ] Apart from BT-34 and BT-49 (present in sample A of both syntaxes and in
      sample B), samples A, B and L give the same field model as before; CII
      and UBL still give the same fields for A, B and L (0006); opening
      sample L still meets the time limits of 0007.
- [ ] The German catalogue check (0017) passes with all new labels and
      section titles.

## Out of scope

- Line-level allowances and charges (BG-27, BG-28), item attributes
  (BG-32), and further line references (BT-128 is already shown).
- Payee (BG-10), seller tax representative (BG-11), card payment (BG-18).
- VAT accounting currency (BT-6, BT-111) and VAT point date (BT-7, BT-8).
- BT-79 (delivery country subdivision), BT-165 (address line 3) and
  scheme identifiers of other identifiers.
- Names for allowance and charge reason codes (UNTDID 5189 / 7161).
- Fields of the Extended profile beyond EN 16931.

## Notes

- German labels follow the style of the existing ones and the KoSIT
  XRechnung visualisation (Apache 2.0) as reference for wording.
- BR-29 (period end not before start) and BR-33 (allowance reason required)
  are examples; any rule naming one of the new terms is attributed by 0011.
- The section's element (used for findings without a field, 0011, and the
  XML view, 0014) is the delivery element; where that is absent (in UBL,
  `cac:Delivery` is optional), it is the element of the invoicing period.
  CII always has `ram:ApplicableHeaderTradeDelivery`, possibly empty.
- Review on 2026-11-19 (by Claude, accepted by the author), changes to the
  first draft:
  - Samples A and B (CII and UBL) already contain the electronic addresses
    BT-34 and BT-49, so their models change. (The review first assumed CII
    sample A lacked them; it has them, so no fixture change was needed –
    corrected during implementation.)
  - In UBL the creditor identifier is a seller party identification with
    scheme `SEPA` and must not appear as BT-29 as well.
  - Sample C states its totals and has no payee account, so that the totals
    and the direct debit can be checked; BT-18 added to check it is not an
    attachment.
  - The position of BT-34 in AC3 is stated between neighbours that sample C
    actually has; UBL credit notes are named explicitly.
- The section `Lieferung (BG-13)` holds the invoicing period because EN 16931
  defines BG-14 as part of BG-13; users looking for the "Leistungszeitraum"
  find it there, next to the delivery date.

## Progress

- **Done**:
  - `fields/terms.rs`: 43 new business terms with their official English
    names and four section titles; `po/de.po` has the German labels.
  - `fields/cii.rs`, `fields/ubl.rs`: the new fields and the sections
    `Vorausgegangene Rechnung (BG-3)`, `Lieferung (BG-13)`,
    `Nachlass (BG-20)` and `Zuschlag (BG-21)` for CII and UBL (invoices and
    credit notes).
  - `fields/mod.rs`: a section may have a fallback anchor (UBL: the
    invoicing period when there is no `cac:Delivery`); the condition
    `NotAttribute` keeps the UBL creditor identifier out of BT-29 and BT-46;
    the display format for electronic addresses with their scheme.
  - Sample C (`cii/sample-c.xml`, `ubl/sample-c.xml`), described in
    `docs/fixtures.md`.
- **Verified** (2026-11-19):
  - `cargo test --workspace`: 219 passed; new: 10 tests in
    `crates/core/tests/map_more_terms.rs` (AC1–AC9; the location and XML
    view check of AC9 is there instead of `link_fields_xml.rs`, to share the
    sample C variants) and `sample_c_has_no_errors` in `validate_en16931.rs`.
    The expectation of the seller fields of sample A gained BT-34
    (`map_cii_header.rs`); the UBL/CII comparison for A, B and L, the sample
    L time limits and the catalogue check are green unchanged (AC10, AC11).
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
    warnings` and the architecture check are clean.
  - Smoke run of the release build in a container with a virtual display:
    sample C opens with `No findings` and shows the references and the
    preceding invoice.
  - KoSIT validator 1.6.3 (checksums as released) with the EN 16931
    scenario of `scripts/kosit/scenarios-en16931.xml`, the XRechnung
    configuration 2026-08-31 and the CEN XSLT of `validation-1.3.17`: CII and
    UBL sample C pass the schema and the schematron without any message
    (2026-11-19, downloads approved by the author).

