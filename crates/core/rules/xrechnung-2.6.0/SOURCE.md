# KoSIT XRechnung Schematron 2.6.0 (XRechnung 3.0.2)

- Source: https://github.com/itplr-kosit/xrechnung-schematron, release `v2.6.0`
  (tag commit `217c6cbf8ac9a99d780cefb4f044b2c2778d4744`)
- Release asset `xrechnung-3.0.2-schematron-2.6.0.zip`
  (sha256 `ca5e07afd04e72cd283d581590ffff3a15f4aac9d2f40c993345d78e67ea22b4`)
- Files, copied unchanged from the asset's `schematron/` directory:
  - `cii/XRechnung-CII-validation.sch`
  - `ubl/XRechnung-UBL-validation.sch`
  - `common.sch` (included by both)
- The release files, unlike the sources at the tag, contain the PEPPOL BIS
  rules (`PEPPOL-EN16931-R*`) that KoSIT merges into the XRechnung rules.
- Licence: Apache License 2.0, see `LICENSE`.

To update: replace the files and this note with a newer release, change the
version strings in `crates/core/src/validate/mod.rs`, regenerate the KoSIT
expected results with `scripts/kosit-reference.sh` and run `cargo test`.
