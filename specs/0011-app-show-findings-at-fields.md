---
id: 0011
title: App - show validation findings at the fields and as a summary
status: draft
created: 2026-11-16
depends_on: [0007, 0009]
---

## Context

A list of rule violations alone forces users to look up where the problem is.
The viewer shows each finding right next to the affected field, and a summary
below the fields for an overview, so users see at a glance whether an invoice
is fine and what exactly is wrong.

## Requirement

After an invoice is opened, it is validated automatically without blocking the
window. Each finding that can be attributed to a field is shown directly at
that field; findings that cannot be attributed appear in the summary only. The
summary below the fields lists all findings, grouped by severity, and allows
jumping to the affected field.

A finding is attributed to the field whose location equals the finding's
location; otherwise to the field of the first business term in its
`business_terms` that exists in the field model; otherwise it is a document
level finding.

## Acceptance criteria

- [ ] Given the sample with the BR-CO-15 total mismatch (0009), when it is
      opened, then the BT-112 field shows an error marker (red icon and red
      text) with `BR-CO-15` and the official message, and the section
      "Summen (BG-22)" header shows the count `1 Fehler`.
- [ ] Given an invoice with one error and one warning, then the summary below
      the last section shows the headline `1 Fehler, 1 Warnung`, followed by
      the errors, then the warnings, each with rule ID, message and the field
      label (e.g. `BT-112 Rechnungsbetrag inkl. USt`).
- [ ] Given the user clicks a finding in the summary, then the field pane
      scrolls so that the affected field is visible and the field is
      highlighted for at least 2 s.
- [ ] Given the sample without BT-24 (BR-01, not attributable to an existing
      field), then the finding appears in the summary under "Dokument" and
      not at any field.
- [ ] Given a valid invoice, then the summary shows `Keine Befunde` and the
      applied rule sets (e.g. `EN 16931 CII`); given a `minimum` profile
      invoice, it shows the not-applicable reason instead.
- [ ] While validation runs, the summary shows `Prüfe…`, and the PDF and field
      panes can be scrolled; for sample L the window never freezes
      for more than 100 ms (manual check with a visible scroll during
      validation).
- [ ] Given validation fails internally (e.g. the rule engine reports an
      error), then the summary shows `Prüfung fehlgeschlagen` with the reason,
      and fields and PDF are still shown.

## Out of scope

- Filtering or exporting findings, a validation report as PDF.
- Explanations or fix suggestions beyond the official message.

## Notes

- GUI-bound criteria are verified manually with the synthetic samples from
  0009 and 0010; the attribution rule is covered by unit tests in the core.
