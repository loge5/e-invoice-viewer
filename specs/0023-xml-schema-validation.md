---
id: 0023
title: Validate - XML schema check (CII D16B, UBL 2.1)
status: draft
created: 2026-11-19
depends_on: [0009, 0010, 0011, 0014]
---

## Context

The app checks invoices against the EN 16931 and XRechnung business rules
(0009, 0010), but not against the XML schema. The KoSIT validator checks the
schema first. An invoice with an unknown element, an element in the wrong
place or a text where an amount belongs can therefore show "No findings" in
the app, although the official validator rejects it. For version 1.0 the app
must report these errors (author, 2026-11-19; see
[ADR 0006](../docs/adr/0006-xml-schema-validation-libxml2.md)).

## Requirement

- **Every invoice XML is checked against its XML schema** before the business
  rules: CII invoices (also from a PDF) against UN/CEFACT CII D16B, UBL
  invoices against UBL 2.1 Invoice, UBL credit notes against UBL 2.1
  CreditNote. This applies to every profile, including Minimum and Basic WL,
  for which the business rules are not applied (0009).
- **Schema errors are findings** with the rule `XSD`, severity error, a
  message that names the element and what is wrong (libxml2's English text,
  with namespace URIs replaced by the usual prefixes such as `ram:`, `cbc:`),
  and the location of the element concerned. They are attributed to fields,
  sections or the document like every finding (0011), appear in the status
  line counts and the summary, and "Show in XML" (0014) jumps to the element.
- **The business rules still run** when the schema check fails, so that the
  user sees everything at once. (The KoSIT validator stops after schema
  errors; the app shows more, not less.)
- **The summary names the schema**: `Checked against XML schema CII D16B and
  EN 16931 CII 1.3.17` (German `Geprüft nach XML-Schema CII D16B und EN 16931
  CII 1.3.17`); with XRechnung `… CII D16B, EN 16931 CII 1.3.17 and
  XRechnung 3.0.2`; UBL `XML schema UBL 2.1`. For Minimum and Basic WL the
  summary names the schema, and the note of 0009 says why the business rules
  were not applied.
- **Strictly local**: the schemas are part of the program; the check opens no
  file and no network connection and writes nothing.
- **Speed**: opening sample L (500 lines) still meets the limits of 0007 and
  0009.

## Acceptance criteria

Samples are defined in [fixtures.md](../docs/fixtures.md).

- [ ] Given samples A, B and C (CII and UBL), then the validation has no
      finding `XSD`, and the summary reads `Checked against XML schema CII
      D16B and EN 16931 CII 1.3.17` for CII sample A (UBL: `XML schema UBL
      2.1`).
- [ ] Given CII sample A with an element `<ram:Foo>1</ram:Foo>` before
      `ram:TypeCode` in `rsm:ExchangedDocument`, then there is exactly one
      finding `XSD`, severity error, whose message contains `ram:Foo` and
      `ram:TypeCode`, and whose location is that of `ram:Foo`; the status
      line reads `1 error` and "Show in XML" selects the `ram:Foo` element.
- [ ] Given CII sample A with BT-106 (`ram:LineTotalAmount` in the totals)
      `abc`, then there is a finding `XSD` at that element, attributed to the
      field BT-106 in `Summen (BG-22)`; the business rule findings about the
      totals appear as well.
- [ ] Given UBL sample A without `cbc:ID` (BT-1) — the variant
      `a-ubl-without-bt1.xml` — then there is a finding `XSD` naming
      `cbc:IssueDate` as unexpected (or `cbc:ID` as missing).
- [ ] For every invoice of the KoSIT reference (370 invoices,
      `expected-en16931.tsv`), the app reports a finding `XSD` exactly when
      KoSIT marks the invoice `!xsd`.
- [ ] Given the Factur-X sample with profile Minimum (`a-minimum.xml`), the
      schema is checked, and the summary names the schema and the note of
      0009.
- [ ] The finding messages contain no namespace URI in braces
      (`{urn:…}`), only prefixed names.
- [ ] Opening sample L takes no longer than the limit of 0007, and its
      validation no longer than the limit of 0009 (checked with the
      existing timing tests).
- [ ] German and English texts for the summary exist; the catalogue check
      (0017) passes.
- [ ] The AppImage and the Windows build contain the check and the embedded
      schemas: opening `cii-bad.xml` (sample A with `ram:Foo`) in both shows
      `1 error` (smoke check in a container for the AppImage; manual check in
      the VM for Windows).

## Out of scope

- The profile-specific XML schemas of Factur-X/ZUGFeRD (0027).
- Translating the libxml2 messages.
- XSD 1.1, DTDs.

## Notes

- Schemas and versions: CII D16B (`100.D16B`, UN/CEFACT 2016), UBL 2.1
  (OASIS 2013), as shipped in the KoSIT configuration 2026-08-31.
- libxml2 is initialised once per process; the validation already runs on a
  worker thread (0011), and every validation uses its own validation
  context.
- The location of a schema finding uses the same notation as field and rule
  locations (0011, 0014), computed from the element libxml2 reports, so that
  attribution and "Show in XML" work without changes.

## Test list (proposed)

- T1 `samples_are_schema_valid` (core, `validate_schema.rs`): A, B, C in CII
  and UBL have no `XSD` finding; summary strings for CII and UBL (AC1).
- T2 `unknown_element_is_reported_at_its_location`: sample A with `ram:Foo`;
  one finding, message with prefixes, location selects `ram:Foo`,
  attribution and the XML range (AC2, AC7).
- T3 `wrong_datatype_is_reported_at_the_field`: BT-106 `abc`; finding
  attributed to BT-106; business rule findings still present (AC3).
- T4 `missing_mandatory_element_in_ubl`: variant `a-ubl-without-bt1.xml`
  (AC4).
- T5 `schema_verdict_equals_kosit`: all 370 invoices of
  `expected-en16931.tsv` (AC5).
- T6 `minimum_profile_is_schema_checked` (AC6).
- T7 existing timing tests for sample L unchanged (AC8); catalogue check
  (AC9).
- T8 `schemas_are_read_from_memory_only`: unit test of the libxml2 module:
  an import of an unknown address fails instead of reading a file; network
  is off.
- P1 smoke check of the AppImage in a container with `cii-bad.xml`; the
  Windows check is manual in the VM (AC10).
