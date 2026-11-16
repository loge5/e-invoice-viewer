# Sample invoices (test fixtures)

The specs refer to a small, fixed set of **synthetic** sample invoices, so that
every acceptance criterion points at the same, known data. All names,
addresses and identifiers are fictitious; the IBAN is a well-known test IBAN.
Each sample exists as CII and as UBL; samples A and B additionally as a
Factur-X PDF (`RE-2026-0042.pdf`, `RE-2026-0043.pdf`).

When the fixture files are created, their validity (as stated per sample
below) is confirmed once with the official KoSIT validator; if a sample turns
out to be invalid, the sample is fixed, not the expectation.

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

- BT-24 `urn:cen.eu:en16931:2017#compliant#urn:xeinkauf.de:kosit:xrechnung_3.0`
- BT-10 `04011000-12345-34`, BT-22 `Vielen Dank für Ihren Auftrag.`
- Line 1: BT-151 `S`, BT-152 `19`, two notes BT-127 `Lieferung KW 46` and
  `inkl. Anfahrt`
- Line 2: BT-151 `E`, BT-152 `0`
- Two VAT breakdowns, in this order:
  1. BT-116 `1000.00`, BT-117 `190.00`, BT-118 `S`, BT-119 `19`
  2. BT-116 `50.00`, BT-117 `0.00`, BT-118 `E`, BT-119 `0`, BT-120
     `Steuerbefreit nach § 4 Nr. 21 UStG`
- Totals: BT-109 `1050.00`, BT-110 `190.00`, BT-112 `1240.00`, BT-115 `1240.00`
- Valid against EN 16931 and XRechnung 3.0 (no errors)

## Sample L — `RE-2026-0500`, 500 lines

- Profile EN 16931, seller/buyer/payment as in the common data
- Lines `1` to `500`: BT-153 `Artikel <n>`, BT-129 `1` `C62`, BT-146 `1.00`,
  BT-131 `1.00`, BT-151 `S`, BT-152 `19`
- Totals: BT-106 `500.00`, BT-110 `95.00`, BT-112 `595.00`, BT-115 `595.00`
- Additionally stored as a variant without any line breaks (single-line XML)
