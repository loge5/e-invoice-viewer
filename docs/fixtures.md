# Sample invoices (test fixtures)

The specs refer to a small, fixed set of **synthetic** sample invoices, so that
every acceptance criterion points at the same, known data. All names,
addresses and identifiers are fictitious; the IBAN is a well-known test IBAN.
Each sample exists as CII and as UBL; samples A and B additionally as a
Factur-X PDF (`RE-2026-0042.pdf`, `RE-2026-0043.pdf`).

When the fixture files are created, their validity (as stated per sample
below) is confirmed once with the official KoSIT validator; if a sample turns
out to be invalid, the sample is fixed, not the expectation.

Files: `crates/core/tests/fixtures/cii/sample-{a,b,c}.xml` and
`crates/core/tests/fixtures/ubl/sample-{a,b,c}.xml`. UBL sample A was created
with spec 0009 and is accepted by the KoSIT validator without any message. UBL
sample B was created with spec 0010; both B samples are accepted by the KoSIT
validator (configuration XRechnung 3.0.2) with only the information
`BR-DE-TMP-32` (no delivery date).

Variants needed by a single spec (e.g. "sample A without BT-24") are described
in that spec and derived from these samples.

## Common data (A and B)

| BT | Value |
|---|---|
| BT-2 issue date | 17.11.2026 (CII `20261117`, format 102 = YYYYMMDD; UBL `2026-11-17`) |
| BT-3 type | `380` (Rechnung) |
| BT-5 currency | `EUR` |
| BT-9 due date | 01.12.2026 (CII `20261201`; UBL `2026-12-01`) |
| Seller (BG-4) | BT-27 `Muster GmbH`, BT-31 `DE123456789`, BT-34 `rechnung@muster.example` (scheme `EM`), BT-35 `Hauptstraße 1`, BT-38 `12345`, BT-37 `Berlin`, BT-40 `DE` |
| Seller contact (BG-6) | BT-41 `Erika Mustermann`, BT-42 `+49 30 1234567`, BT-43 `rechnung@muster.example` |
| Buyer (BG-7) | BT-44 `Beispiel AG`, BT-49 `einkauf@beispiel.example` (scheme `EM`), BT-50 `Musterweg 5`, BT-53 `80331`, BT-52 `München`, BT-55 `DE` |
| Payment (BG-16) | BT-81 `58` (SEPA credit transfer), BT-84 `DE02120300000000202051` |
| Line 1 | BT-126 `1`, BT-153 `Beratung`, BT-129 `8` `HUR`, BT-146 `125.00`, BT-131 `1000.00` |
| Line 2 | BT-126 `2`, BT-153 `Schulungsunterlagen`, BT-129 `2` `C62`, BT-146 `25.00`, BT-131 `50.00` |
| BT-106 sum of lines | `1050.00` |

## Sample A — `RE-2026-0042`, profile EN 16931, one VAT rate

- BT-24 `urn:cen.eu:en16931:2017`
- Both lines: BT-151 `S`, BT-152 `19`
- One VAT breakdown: BT-116 `1050.00`, BT-117 `199.50`, BT-118 `S`, BT-119 `19`
- Totals: BT-109 `1050.00`, BT-110 `199.50`, BT-112 `1249.50`, BT-115 `1249.50`
- **No** buyer reference (BT-10), **no** notes (BT-22, BT-127)
- Valid against EN 16931 (no errors)

## Sample B — `RE-2026-0043`, profile XRechnung 3.0, two VAT categories

- BT-23 `urn:fdc:peppol.eu:2017:poacc:billing:01:1.0` (required by the
  PEPPOL rule `PEPPOL-EN16931-R001` that is part of the XRechnung rules; added
  with spec 0010)
- BT-24 `urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0`
- BT-10 `04011000-12345-34`, BT-22 `Vielen Dank für Ihren Auftrag.`
- Line 1: BT-151 `S`, BT-152 `19`, two notes BT-127 `Lieferung KW 46` and
  `inkl. Anfahrt` (UBL: only `Lieferung KW 46`, because UBL allows at most one
  line note, rule `UBL-SR-34`)
- Line 2: BT-151 `E`, BT-152 `0`
- Two VAT breakdowns, in this order:
  1. BT-116 `1000.00`, BT-117 `190.00`, BT-118 `S`, BT-119 `19`
  2. BT-116 `50.00`, BT-117 `0.00`, BT-118 `E`, BT-119 `0`, BT-120
     `Steuerbefreit nach § 4 Nr. 21 UStG`
- Totals: BT-109 `1050.00`, BT-110 `190.00`, BT-112 `1240.00`, BT-115 `1240.00`
- Valid against EN 16931 and XRechnung 3.0 (no errors)

## Sample C — `RE-2026-0042` with references, delivery and allowances

Created with spec 0021 from sample A; all other data as in sample A.

- References: BT-11 `PRJ-7` (CII also needs the project name, `Projekt`),
  BT-12 `V-2026-03`, BT-13 `PO-4711`, BT-18 `OBJ-1` (type code 130)
- Preceding invoice (BG-3): BT-25 `RE-2026-0017`, BT-26 `2026-10-15`
- Buyer electronic address BT-49 `04011000-12345-34` (scheme `0204`) instead
  of the e-mail address
- Delivery (BG-13): BT-70 `Lager Nord`, BT-72 `2026-11-03`, BT-75
  `Industriestraße 5`, BT-78 `20457`, BT-77 `Hamburg`, BT-80 `DE`
- Invoicing period (BG-14): BT-73 `2026-11-01`, BT-74 `2026-11-30`
- Payment: BT-81 `59` (SEPA direct debit), BT-89 `MANDAT-0815`, BT-90
  `DE98ZZZ09999999999` (UBL: seller party identification with scheme
  `SEPA`), BT-91 `DE02120300000000202051`; no BT-84
- Allowance (BG-20): BT-92 `50.00`, BT-95 `S`, BT-96 `19`, BT-97 `Rabatt`
- Charge (BG-21): BT-99 `10.00`, BT-102 `S`, BT-103 `19`, BT-104 `Fracht`
- Line 1: BT-132 `1`, period BT-134 `2026-11-01` to BT-135 `2026-11-15`
- One VAT breakdown: BT-116 `1010.00`, BT-117 `191.90`, BT-118 `S`, BT-119
  `19`
- Totals: BT-106 `1050.00`, BT-107 `50.00`, BT-108 `10.00`, BT-109
  `1010.00`, BT-110 `191.90`, BT-112 `1201.90`, BT-115 `1201.90`
- Valid against EN 16931: accepted by the KoSIT validator 1.6.3 (EN 16931
  scenario) without any message, CII and UBL

## Sample L — `RE-2026-0500`, 500 lines

- Profile EN 16931, seller/buyer/payment as in the common data
- Lines `1` to `500`: BT-153 `Artikel <n>`, BT-129 `1` `C62`, BT-146 `1.00`,
  BT-131 `1.00`, BT-151 `S`, BT-152 `19`
- Totals: BT-106 `500.00`, BT-110 `95.00`, BT-112 `595.00`, BT-115 `595.00`
- Additionally needed as a variant without any line breaks (single-line XML)
- Not stored as a file: the test helpers generate sample L deterministically
  from sample A (`sample_l_cii` in `crates/core/tests/common/mod.rs`), which
  keeps about 1 MB of repetitive XML out of the repository
