# XML schemas CII D16B and UBL 2.1 (KoSIT configuration 2026-08-31)

- Source: https://github.com/itplr-kosit/validator-configuration-xrechnung,
  release `v2026-08-31`, asset
  `xrechnung-3.0.2-validator-configuration-2026-08-31.zip`
  (sha256 `2530cd107c414511c5d0462ec10f886910395abfca820db82e83d70bf01221a8`).
- Files, copied unchanged:
  - `cii/`: all four files of `resources/cii/16b/xsd/` — UN/CEFACT Cross
    Industry Invoice, schema version `100.D16B`.
  - `ubl/maindoc/`: `UBL-Invoice-2.1.xsd`, `UBL-CreditNote-2.1.xsd`, and
    `ubl/common/`: the 13 modules of `resources/ubl/2.1/xsd/common/` they
    import (directly or indirectly) — OASIS UBL 2.1.
- Licences: the CII schemas carry the UN/CEFACT copyright notice, which
  permits copying and distribution of the unmodified files; the UBL schemas
  carry the OASIS copyright notice, which permits copying and distribution
  with the notice. The notices stay in the files.
- Used by spec 0023 (ADR 0006); embedded into the program.

To update: see `docs/rule-updates.md`.
