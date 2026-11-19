---
id: 0025
title: Robustness - public sample invoices and fuzzing
status: draft
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

- [ ] The samples of the two collections are in
      `crates/core/tests/fixtures/public/` with `LICENSE` and `README.md`
      (source, tag or commit, what was left out and why).
- [ ] A test opens every sample and checks the points of the requirement;
      it lists every deviation from the KoSIT result by file and rule, and
      passes only without deviations or with deviations recorded in a
      reviewed exception list with a reason each.
- [ ] Given an XML file with 100 000 nested elements, opening it shows an
      error message (not well-formed or too deep) within 5 s, without a
      crash.
- [ ] `cargo fuzz` targets exist for opening, validation and the XML layout;
      each ran for at least one hour on the reference machine without a
      crash or an input taking longer than 5 s; the corpus of interesting
      inputs is kept in the repository (minimised).
- [ ] Every crash or hang found while fuzzing has a regression test with the
      minimised input and is fixed.
- [ ] The README of the fuzz directory explains how to run the targets.

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

## Test list (proposed)

- T1 `public_samples_open_and_match_kosit` (core, `public_samples.rs`): every
  sample opens, no empty section, key fields present, finding locations
  inside the document, validation equals the recorded KoSIT result, with the
  exception list (AC1, AC2).
- T2 `deep_nesting_is_refused`: 100 000 nested elements, error within 5 s
  (AC3).
- T3 one regression test per fuzzing find (AC5).
- P1 fuzz targets `open`, `validate`, `xml_layout`, one hour each on the
  reference machine, results in the spec (AC4).
