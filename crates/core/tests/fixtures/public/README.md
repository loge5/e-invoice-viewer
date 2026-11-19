# Public sample invoices (spec 0025)

Real-world-like invoices from public collections, used to check that the app
opens them and validates them like the official KoSIT validator. Copied
unchanged, except where noted.

## `xrechnung-testsuite/`

- Source: https://github.com/itplr-kosit/xrechnung-testsuite, release
  `v2026-08-31`, asset `xrechnung-3.0.2-testsuite-2026-08-31.zip`
  (sha256 `1e81ef7563e0fa04c0e0b0631318a7e0843a3c0beb57d83ef95ac0c4a095e3d7`).
- Files: all invoices of `instances/standard/` (66) and
  `instances/extension/` (6), CII and UBL. Left out: `instances/technical-cases/`
  (CVD and CIUS edge cases, partly invalid on purpose; the rule test
  instances of `../kosit/` already cover the rules).
- Licence: Apache License 2.0, see `xrechnung-testsuite/LICENSE` (taken from
  the repository at the same tag; the release asset carries none).

## `cen/`

- Source: https://github.com/ConnectingEurope/eInvoicing-EN16931, tag
  `validation-1.3.17`, directories `cii/examples/` (15 files) and
  `ubl/examples/` (19 files), all files.
- Changes of names only, content unchanged: the two files ending in `.XML`
  were renamed to `.xml` (`BIS3_Invoice_negativ.xml`,
  `BIS3_Invoice_positive.xml`), so that the scripts and tests pick them up,
  and `FT G2G_TD01 con Allegato, Bonifico e Split Payment.xml` to
  `FT_G2G_TD01_con_Allegato_Bonifico_e_Split_Payment.xml`, because the KoSIT
  validator names its report differently for names with spaces and commas.
- Licence: European Union Public Licence 1.2, see `cen/LICENSE.txt`.

## Reference results

`../kosit/expected-en16931.tsv` and `../kosit/expected-xrechnung.tsv` contain
the KoSIT validator's results for these files too (keys `public/…`),
produced by `scripts/kosit-reference.sh`.
