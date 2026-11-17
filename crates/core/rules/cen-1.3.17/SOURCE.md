# CEN EN 16931 validation artefacts 1.3.17

- Source: https://github.com/ConnectingEurope/eInvoicing-EN16931
- Tag: `validation-1.3.17` (commit `d0dea7300c2ad60d50af4a4f31e122fdbf7c0ed3`)
- Files, copied unchanged:
  - `cii/schematron/preprocessed/EN16931-CII-validation-preprocessed.sch`
  - `ubl/schematron/preprocessed/EN16931-UBL-validation-preprocessed.sch`
- Licence: European Union Public Licence 1.2, see `LICENSE.txt`.
  Distributed as part of a GPL-3.0-or-later program under the compatibility
  clause of EUPL-1.2 (Art. 5 and appendix), see ADR 0002.

To update: replace both files and this note with a newer tag, change the
version string in `crates/core/src/validate.rs`, regenerate the KoSIT expected
results with `scripts/kosit-reference.sh` and run `cargo test`.
