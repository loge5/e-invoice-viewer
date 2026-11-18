---
id: 0017
title: Core - invoice content in German and English
status: in-progress
created: 2026-11-18
depends_on: [0004, 0005, 0006, 0009, 0010]
---

## Context

The app becomes multilingual, starting with German and English (decision of
the author on 2026-11-18, see [ADR 0003](../docs/adr/0003-translations-gettext-embedded.md)).
Much of what a user reads is not produced by the user interface but by the
domain logic: field labels, section titles, the names of codes and units, the
formatted numbers and dates, and the app's own validation notes. These must
be available in both languages before the user interface can switch.

## Requirement

The field model (0004–0006) and the validation result (0009, 0010) can be
produced in **German** or **English**, chosen by the caller for each call:

- **Labels and titles**: every field label and section title exists in both
  languages. The English field labels are the official business term names
  of EN 16931 (e.g. BT-31 `Seller VAT identifier`); section titles of a
  business group use its official name (e.g. `Payment instructions (BG-16)`).
  Business term numbers (`BT-1`, `BG-4`) are the same in both.
- **Code lists and units**: the names of invoice types (BT-3), payment means
  (BT-81), VAT categories (BT-118, BT-151) and units (BT-130) exist in both
  languages. The code itself is still shown in parentheses as before.
- **Numbers and dates** follow the language:
  - German: decimal comma, `.` as thousands separator, dates `DD.MM.YYYY`.
  - English: decimal point, `,` as thousands separator, dates `YYYY-MM-DD`.
- **The app's own validation notes** (profile not applicable, declared
  XRechnung version differs) exist in both languages.
- **Official rule messages** stay exactly as published, in both languages
  (CEN rules in English, XRechnung `BR-DE-*` in German); there are no official
  translations.
- **Language-independent data** is the same in both languages: raw values,
  locations, attributes, business term numbers, rule IDs, severities and the
  attribution of findings.
- **Completeness**: there is no text that exists only in one language. A
  missing translation is detected before a release, not by a user.

## Acceptance criteria

- [ ] Given CII sample A, the section titles in display order are:

      | German | English |
      |---|---|
      | Rechnung | Invoice |
      | Verkäufer (BG-4) | Seller (BG-4) |
      | Käufer (BG-7) | Buyer (BG-7) |
      | Position 1 | Invoice line 1 |
      | Zahlung (BG-16) | Payment instructions (BG-16) |
      | Umsatzsteuer (BG-23) | VAT breakdown (BG-23) |
      | Summen (BG-22) | Document totals (BG-22) |

- [ ] Given CII sample A, these fields read:

      | Field | German label | English label | German display | English display |
      |---|---|---|---|---|
      | BT-1 | Rechnungsnummer | Invoice number | RE-2026-0042 | RE-2026-0042 |
      | BT-2 | Rechnungsdatum | Invoice issue date | 17.11.2026 | 2026-11-17 |
      | BT-3 | Rechnungsart | Invoice type code | Rechnung (380) | Commercial invoice (380) |
      | BT-81 | Zahlungsart | Payment means type code | SEPA-Überweisung (58) | SEPA credit transfer (58) |
      | BT-129 (line 1) | Menge | Invoiced quantity | 8 Stunden (HUR) | 8 hours (HUR) |
      | BT-106 | Summe der Positionen netto | Sum of Invoice line net amount | 1.050,00 EUR | 1,050.00 EUR |
      | BT-112 | Rechnungsbetrag inkl. USt | Invoice total amount with VAT | 1.249,50 EUR | 1,249.50 EUR |

- [ ] Code and unit names in English are exactly those of the table
      "English code and unit names" below, e.g. `381` →
      `Credit note (381)`, `S` → `Standard rate (S)`, `AE` →
      `VAT reverse charge (AE)`, `C62` → `2 pieces (C62)` (German `2 Stück (C62)`). Codes without a
      known name are shown as before (`2.5 XPP` in English, `2,5 XPP` in
      German).
- [ ] Number formats: the amount `1234567.5` with currency `EUR` reads
      `1.234.567,50 EUR` in German and `1,234,567.50 EUR` in English; the
      quantity `2.50` reads `2,5` and `2.5`; the negative amount `-30.00`
      reads `-30,00 EUR` and `-30.00 EUR`. A value that is not a number is
      shown unchanged in both languages.
- [ ] Dates: CII `20261117` (format 102) and UBL `2026-11-17` read
      `17.11.2026` in German and `2026-11-17` in English; an invalid date
      (`20261332`) is shown unchanged in both.
- [ ] Validation notes: for a Factur-X invoice with profile Minimum the reason
      reads `Profil Minimum: keine vollständige Rechnung nach EN 16931` in
      German and `Profile Minimum: not a complete invoice according to
      EN 16931` in English; for an XRechnung invoice declaring version 2.3
      the note reads `Geprüft nach XRechnung 3.0.2, Rechnung erklärt Version
      2.3` and `Checked against XRechnung 3.0.2, invoice declares version 2.3`.
- [ ] Given CII sample B without BT-10, the finding `BR-DE-15` has the same
      message, location, severity and attribution in German and English, and
      a CEN finding (e.g. `BR-CO-15` on a sample with wrong totals) keeps its
      English message in German.
- [ ] For all sample invoices (CII and UBL, A, B, L), the German and English
      field models differ only in labels, titles and display values; raw
      values, locations, attributes and IDs are equal.
- [ ] Every text that can appear in the field model or the validation notes
      has a German and an English version; an automated check fails if one is
      missing.

## Out of scope

- The user interface texts (0018) and choosing the language (0018, 0019).
- Languages other than German and English (the mechanism must allow adding
  them without code changes, ADR 0003).
- Translating the official rule messages.
- Singular and plural forms of unit names (`1 hours`); units are shown in one
  form as today.

## Notes

- Review on 2026-11-18: the author decided on the official EN 16931 names for
  the English labels, accepting that they are longer than the German ones;
  consequently the line section is `Invoice line 1` (BG-25 "Invoice line"),
  and the code and unit names are given as a table.
- The German texts stay exactly as they are today; the existing tests for
  German keep passing unchanged.
- **English code and unit names** (normative for the acceptance criteria).
  They are the official names of the code lists in sentence case; names
  marked * are shortened or adapted, as the German ones are:

  | List | Code | English name |
  |---|---|---|
  | Invoice type (UNTDID 1001) | 326 | Partial invoice |
  | | 380 | Commercial invoice |
  | | 381 | Credit note |
  | | 384 | Corrected invoice |
  | | 389 | Self-billed invoice |
  | | 875 | Partial construction invoice |
  | | 876 | Partial final construction invoice |
  | | 877 | Final construction invoice |
  | Payment means (UNTDID 4461) | 1 | Instrument not defined |
  | | 10 | In cash |
  | | 20 | Cheque |
  | | 30 | Credit transfer |
  | | 42 | Payment to bank account |
  | | 48 | Bank card |
  | | 49 | Direct debit |
  | | 54 | Credit card |
  | | 57 | Standing agreement |
  | | 58 | SEPA credit transfer |
  | | 59 | SEPA direct debit |
  | | 97 | Clearing between partners |
  | VAT category (UNTDID 5305) | S | Standard rate |
  | | Z | Zero rated goods |
  | | E | Exempt from tax |
  | | AE | VAT reverse charge |
  | | K | Intra-community supply * |
  | | G | Export outside the EU * |
  | | O | Outside the scope of VAT * |
  | | L | Canary Islands IGIC * |
  | | M | Ceuta and Melilla IPSI * |
  | Unit (UN/ECE Rec. 20) | C62, H87 | pieces * |
  | | HUR | hours * |
  | | DAY | days * |
  | | MON | months * |
  | | KGM | kilograms * |
  | | MTR | metres * |
  | | LTR | litres * |
  | | LS | lump sum |

  Units are in the plural, like the German ones (`Stunden`).

## Progress

- **Done**:
  - `crates/core/src/i18n.rs`: `Language` (German, English), `tr`, `tr_n`
    and the marker `n_`; the German catalogue is compiled from `po/de.po` by
    `crates/core/build.rs` (`msgfmt --check`) and embedded (ADR 0003).
  - Source texts are English: the official EN 16931 names in
    `fields/terms.rs`, section titles, code and unit names (`display.rs`),
    the validation notes. The former German texts are in `po/de.po`
    (109 entries), so German output is unchanged.
  - `map_invoice`, `document::open` and `Validator::validate` take the
    language. Number and date formats follow it. The app passes German until
    0018.
  - `scripts/update-po.sh` extracts the marked texts into
    `po/messages.pot` and merges them into `po/de.po`.
- **Verified** (2026-11-18):
  - `cargo test --workspace`: 189 passed; new: 6 tests in
    `tests/content_per_language.rs` (AC1, AC2, AC6–AC9) and 3 unit tests in
    `fields/display.rs` (AC3–AC5). All existing tests pass unchanged apart
    from the added language argument.
  - The completeness test fails when an entry is removed from `po/de.po`
    (checked by removing `Seller city`).
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
    warnings` and the architecture check are clean.
  - Smoke run of the release build: the app still shows `Rechnungsnummer`,
    `Rechnung (380)`, `8 Stunden (HUR)` and `1.249,50 EUR`.
- **Next** — all criteria are covered by automated tests; the author decides
  on `implemented`.

