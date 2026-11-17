# KoSIT validator configuration for XRechnung 3.0.2, release 2026-08-31

- Source: https://github.com/itplr-kosit/validator-configuration-xrechnung,
  release asset `xrechnung-3.0.2-validator-configuration-2026-08-31.zip`
  (sha256 `2530cd107c414511c5d0462ec10f886910395abfca820db82e83d70bf01221a8`)
- File, copied unchanged: `scenarios.xml`. Only its scenario match
  expressions and `customLevel` severity changes are used (spec 0010).
- Licence: Apache License 2.0, see `LICENSE` (taken from the repository; the
  release asset carries none).

To update: replace `scenarios.xml` and this note with a newer release, update
the version strings in `crates/core/src/validate/mod.rs`, regenerate the KoSIT
expected results with `scripts/kosit-reference.sh` and run `cargo test`.
