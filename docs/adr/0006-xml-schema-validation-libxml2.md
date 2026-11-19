# 0006 — XML schema validation with libxml2, schemas embedded

- Status: proposed
- Date: 2026-11-19
- Specs: [0023](../../specs/0023-xml-schema-validation.md)

## Context

The validation (ADR 0002) evaluates the official Schematron rules only. The
KoSIT validator first checks the invoice against the XML schema (CII D16B or
UBL 2.1) and reports schema errors: unknown or misplaced elements, wrong data
types, missing mandatory elements. An invoice that fails the schema can show
"No findings" in the app today. For 1.0 the app should not miss what the
official validator reports (author, 2026-11-19).

The author decided on 2026-11-19: try the pure-Rust crate **uppsala** first,
**libxml2** if uppsala does not hold up.

**Spike (2026-11-19)** with the schemas of the KoSIT configuration
2026-08-31 and samples A, B and C:

| | uppsala 0.4.0 | libxml2 2.15 (crate `libxml` 0.3.24) |
|---|---|---|
| Valid samples (CII, UBL) | valid | valid |
| Unknown element in `rsm` (root schema) | found | found |
| Unknown element in `ram` (imported schema) | **not found** | found |
| `abc` as an amount (`xs:decimal`) | **not found** | found |
| Time per invoice | < 0.1 ms | < 0.2 ms; schema load 8 ms (CII), 26 ms (UBL) |

uppsala checks only the root schema; types from imported schemas are not
enforced, which is where all invoice content is. It is therefore not usable.

## Decision

1. **libxml2** validates the invoice XML against the XML schema, through the
   crate [`libxml`](https://crates.io/crates/libxml) (MIT). libxml2 itself is
   MIT-licensed and already part of the app's runtime: GtkSourceView depends
   on it, so the AppImage and the Windows folder contain it today.
2. **Schemas embedded**: CII D16B (`CrossIndustryInvoice_100pD16B.xsd` and its
   three imports) and UBL 2.1 (Invoice, CreditNote and the common modules
   they import), unchanged, as published in the KoSIT configuration
   2026-08-31, are embedded in the binary like the Schematron rules. libxml2
   reads them through its input callbacks from memory, under an internal
   address that cannot be a file or network address. Nothing is written,
   nothing is fetched; network access of libxml2 is switched off.
3. The schema check lives in `crates/core` next to the Schematron runner.
   `crates/core` stays free of GUI libraries; libxml2 is not one.
4. The unsafe calls (input callbacks, structured errors with the failing
   node) are kept in one small module with its own tests.

## Alternatives

| Option | Why not |
|---|---|
| uppsala | Does not validate content of imported schemas (spike) |
| Schemas as data files next to the binary | Simpler (no input callbacks), but `cargo run`, the AppImage and the Windows folder would each need the files at a known place, and a missing file would silently disable the check |
| Writing the embedded schemas to a temporary folder at run time | Breaks the rule that the app writes nothing but its settings and the user's saves |
| Calling an external validator (Java, xmllint) | No external programs (ADR 0002) |

## Consequences

- **New dependency** `libxml` (MIT), approved as fallback by the author on
  2026-11-19. It generates its bindings at build time with `bindgen`, so
  building needs **libclang** and the libxml2 development files: Arch
  `clang libxml2`, Ubuntu `libclang-dev libxml2-dev`, MSYS2 `clang libxml2`.
  The AppImage container, the Windows job and the README get these.
- `crates/core` contains `unsafe` code for the first time, limited to the
  libxml2 module.
- About 1.4 MB of schemas are embedded (UN/CEFACT and OASIS licences permit
  unchanged redistribution with their notices, which stay in the files; the
  README names them).
- Schema messages are libxml2's English texts, with namespace URIs replaced
  by the usual prefixes (`ram:`, `cbc:`, …) for readability.
