---
id: 0025
title: Robustness - public sample invoices and fuzzing
status: reviewed
created: 2026-11-19
depends_on: [0009, 0010, 0013, 0023, 0024]
---

## Context

The tests use synthetic samples (A, B, C, L) and the small rule test
instances of KoSIT. Real invoices are larger and use more of the standards.
They also arrive as e-mail attachments from strangers, so the app must never
crash or hang on a broken or hostile file. For 1.0 (author, 2026-11-19) the
app is checked against the public sample collections and the input parsers
are fuzzed.

## Requirement

- **Public sample collections** are part of the regression tests, where their
  licence allows it, copied unchanged with their licence and source:
  - the KoSIT XRechnung test suite (`itplr-kosit/xrechnung-testsuite`,
    Apache 2.0): its business cases ("standard" and "extension"), CII and UBL;
  - the CEN examples of the EN 16931 validation artefacts
    (`ConnectingEurope/eInvoicing-EN16931`, tag `validation-1.3.17`,
    `cii/examples`, `ubl/examples`, EUPL 1.2).
  - Factur-X/ZUGFeRD PDF samples follow with 0027, once their licence is
    known.
- **For every sample**: it opens without error; the field model has no empty
  section and contains BT-1, BT-2 and BT-112 (or BT-5 and BT-115 for credit
  notes without a total) where the sample has them; every finding has a
  location inside the document; and the validation result equals the KoSIT
  validator's (schema verdict and set of fired EN 16931 rules; XRechnung
  rules for samples of an XRechnung scenario), as recorded once by
  `scripts/kosit-reference.sh`.
- **Fuzzing** with `cargo fuzz` (a development tool, not part of the app):
  targets for opening a document from bytes (PDF and XML paths, including
  attachment extraction and the field model), for the validation, and for
  the XML view layout. The fuzz targets live outside the workspace build, so
  that normal builds need no nightly compiler.
- **No crash, no hang**: for any input, opening returns a document or an
  error message within 5 s and does not panic or abort. Inputs found by the
  fuzzer that broke this are fixed and kept as regression tests.
- **Limits**: inputs larger than the existing limits (0013, 0002) are refused
  with the existing messages; deeply nested XML is refused instead of
  overflowing the stack.

## Acceptance criteria

- [x] The samples of the two collections are in
      `crates/core/tests/fixtures/public/` with `LICENSE` and `README.md`
      (source, tag or commit, what was left out and why).
- [x] A test opens every sample and checks the points of the requirement;
      it lists every deviation from the KoSIT result by file and rule, and
      passes only without deviations or with deviations recorded in a
      reviewed exception list with a reason each.
- [x] Given an XML file with 100 000 nested elements, opening it shows an
      error message (not well-formed or too deep) within 5 s, without a
      crash.
- [x] `cargo fuzz` targets exist for opening, validation and the XML layout;
      each ran for at least one hour on the reference machine without a
      crash or an input taking longer than 5 s; the corpus of interesting
      inputs is kept in the repository (minimised).
- [x] Every crash or hang found while fuzzing has a regression test with the
      minimised input and is fixed.
- [x] The README of the fuzz directory explains how to run the targets.

## Out of scope

- Real customer invoices (never committed, AGENTS.md).
- Fuzzing the GTK user interface.
- PEPPOL BIS samples (licence of the examples not checked; may follow).

## Notes

- Fuzzing needs a nightly Rust compiler and `cargo-fuzz`; both are installed
  for the developer only.
- The KoSIT reference results for the new samples are produced with the
  KoSIT validator 1.6.3 as for 0009/0010; the validator is downloaded for that
  (approved on 2026-11-19).

- Review on 2026-11-19: reviewed by Claude, accepted by the author together
  with the test list.

## Test list (approved 2026-11-19)

- T1 `public_samples_open_and_match_kosit` (core, `public_samples.rs`): every
  sample opens, no empty section, key fields present, finding locations
  inside the document, validation equals the recorded KoSIT result, with the
  exception list (AC1, AC2).
- T2 `deep_nesting_is_refused`: 100 000 nested elements, error within 5 s
  (AC3).
- T3 one regression test per fuzzing find (AC5).
- P1 fuzz targets `open`, `validate`, `xml_layout`, one hour each on the
  reference machine, results in the spec (AC4).

## Progress

- **Done**:
  - Public samples in `crates/core/tests/fixtures/public/`: XRechnung test
    suite v2026-08-31 (standard 66, extension 6) and the CEN examples of
    `validation-1.3.17` (CII 15, UBL 19), with licences and a README
    (source, what was left out or renamed and why). KoSIT references
    regenerated with `scripts/kosit-reference.sh` (480 EN 16931, 431
    XRechnung entries).
  - `detect` refuses XML nested deeper than 256 levels (the libxml2
    default) with its own message, before any parser recurses.
  - Fuzz crate `fuzz/` (own workspace, nightly only) with the targets
    `open`, `validate` and `xml_layout`; README with commands and memory
    limits; minimised corpus committed.
  - The schema check no longer uses XML catalogs and compiles one schema at
    a time: with libxml2 2.9 (Ubuntu 24.04, also in the AppImage) parallel
    compiles hung in the catalog lock (found by CI, not by the fuzzer; test
    `concurrent_validation.rs`).
- **Verified** (2026-11-19):
  - AC1/AC2: `public_samples.rs` (every sample opens, no empty section, key
    fields, finding locations select an element) and `validate_reference.rs`
    (schema verdict, EN 16931 and XRechnung rules equal KoSIT for all
    samples); no exception list was needed.
  - AC3: `robustness.rs` — 100 000 nested elements, as XML and inside an
    invoice, refused within 5 s with "nested more than 256 levels".
  - AC4: one hour per target on the reference machine, one after another,
    memory-limited:
    - `open`: 5 279 583 runs, no finding, at most 587 MB;
    - `xml_layout`: 23 713 500 runs, no finding, at most 499 MB;
    - `validate`: two timeouts under the address sanitizer (below); then,
      without the sanitizer, 293 640 runs, no finding, at most 102 MB.
    - Corpus minimised with `cargo fuzz cmin`: open 4 006, xml_layout 2 644,
      validate 1 194 files (5.5 MB packed in git).
  - AC5: both `validate` timeouts are regression tests in `robustness.rs`
    (`fuzz_validate_timeout_5c0933ae`, `fuzz_validate_timeout_a782dc5b`).
    Neither was a hang: natively they take 0.35 s (first validation with
    schema errors, warm-up) and 0.2 s; the address sanitizer makes a
    validation about 20 times slower. Fixes: the fuzz target warms up the
    error paths in `init`, and `validate` is fuzzed without the sanitizer
    (reasons in `fuzz/README.md`; the sanitizer only instruments Rust code,
    which is memory-safe outside `validate/xsd.rs`).
  - AC6: `fuzz/README.md`.

