# Updating the bundled rules

The app validates offline with rules bundled in the binary
(`crates/core/rules/`, ADR 0002, ADR 0006). It never checks online for new
versions; users learn about rule updates from the release notes
(`CHANGELOG.md`). Check the sources below a few times a year and when
XRechnung announces a new version.

| Rules | Source | In the repository | Version string |
|---|---|---|---|
| CEN EN 16931 business rules | [ConnectingEurope/eInvoicing-EN16931](https://github.com/ConnectingEurope/eInvoicing-EN16931), tag `validation-<version>` | `crates/core/rules/cen-<version>/` | `CEN_VERSION` in `crates/core/src/validate/mod.rs` |
| XRechnung Schematron | [itplr-kosit/xrechnung-schematron](https://github.com/itplr-kosit/xrechnung-schematron), release asset `xrechnung-<x>-schematron-<y>.zip` | `crates/core/rules/xrechnung-<y>/` | `XRECHNUNG_VERSION`, `XRECHNUNG_IDENTIFIER_VERSION` |
| KoSIT validator configuration (scenarios, custom levels) | [itplr-kosit/validator-configuration-xrechnung](https://github.com/itplr-kosit/validator-configuration-xrechnung), release asset | `crates/core/rules/kosit-config-<date>/scenarios.xml` | path in `KOSIT_SCENARIOS` |

Each directory has a `SOURCE.md` with the exact release, commit, checksum and
licence; keep it current. Files are copied **unchanged**.

## Steps

1. Download the new release and compare its checksum with the one published.
2. Replace the files in a new directory named after the version (for example
   `crates/core/rules/cen-1.3.18/`), with `SOURCE.md` and `LICENSE`; delete
   the old directory.
3. Update the `include_str!` paths and version strings in
   `crates/core/src/validate/mod.rs`.
4. Run the tests: `cargo test --workspace`. The rule sets must compile
   completely (`official_rule_sets_compile_completely`,
   `xrechnung_rule_sets_compile_completely`).
5. Regenerate the KoSIT reference results. Download the KoSIT validator
   (`validator-<version>-standalone.jar` from
   [itplr-kosit/validator](https://github.com/itplr-kosit/validator/releases)),
   unpack the configuration release and check out the CEN repository at the
   new tag, then:

   ```bash
   cargo test -p e-invoice-viewer-core --test validate_reference -- --ignored write_variants
   scripts/kosit-reference.sh <validator-standalone.jar> <configuration dir> <cen checkout>
   ```

   This rewrites `crates/core/tests/fixtures/kosit/expected-en16931.tsv` and
   `expected-xrechnung.tsv`. Review the diff: new or removed rule IDs must
   match the release notes of the rules.
6. Run `cargo test --workspace` again; `validate_reference` compares the app
   with the new KoSIT results.
7. Add an entry to `## [Unreleased]` in `CHANGELOG.md`, e.g. "Validation
   uses XRechnung 3.0.3 (Schematron 2.7.0)".

## XML schemas

The XML schemas (CII D16B, UBL 2.1) change rarely. They are taken from the
same KoSIT configuration release (`resources/cii`, `resources/ubl`); see
`SOURCE.md` next to them and spec 0023.
