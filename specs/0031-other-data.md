---
id: 0031
title: Other data - elements without a business term
status: draft
created: 2026-11-20
depends_on: [0004, 0005, 0006, 0024]
---

## Context

The field model (0004 to 0006, 0024) shows every EN 16931 business term. Elements
the schema allows beyond them — the Extended profile of ZUGFeRD/Factur-X,
the XRechnung extension, or any other content — are not shown, so part of
an invoice can be invisible. Split off from 0027 at its review
(2026-11-20).

## Requirement

- Elements of the invoice that are not mapped to a business term are not
  hidden: a section `Weitere Angaben` / `Other data` per place (document,
  each party, each line) lists them with their element names and values, in
  document order.
- Applies to every CII and UBL invoice.

## Acceptance criteria

To be completed at the review; drafts:

- [ ] Given an Extended sample, then every element not mapped to a business
      term appears under `Other data` with its value, at the place it
      belongs to.
- [ ] Given an invoice with only business terms, then no `Other data`
      section is shown.

## Notes

- To clarify at the review: the label of an element (its name, or names
  from the FeRD technical appendix), how attributes are shown, and the
  places for UBL.
