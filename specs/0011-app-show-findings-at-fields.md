---
id: 0011
title: App - show validation findings at the fields and as a summary
status: reviewed
created: 2026-11-16
depends_on: [0007, 0009, 0010]
---

## Context

A list of rule violations alone forces users to look up where the problem is.
The viewer shows each finding right next to the affected field and lists all
findings in a summary, so users see at a glance whether an invoice is fine
and what exactly is wrong.

## Requirement

After an invoice is opened, it is validated automatically (0009, 0010)
without blocking the window. The result appears in three places:

- **Status line** at the top of the field pane: the headline of the result
  (e.g. `2 Fehler`, `Keine Befunde`, `Prüfe…`). Clicking it jumps to the
  summary.
- **At the fields:** each finding attributed to a field is shown at that
  field. A finding attributed to a section is shown at the section header.
  Every section header shows the number of findings in it per severity
  (e.g. `2 Fehler`).
- **Summary** below the last section: the headline, the applied rule sets,
  the XRechnung version note if any, and all findings, grouped by severity
  (errors, then warnings, then notes). Each entry shows the rule ID, the
  official message and where it belongs: the field (`BT-112 Rechnungsbetrag
  inkl. USt`), the section (`Position 2`) or `Dokument`. Clicking an entry
  that belongs to a field or section jumps there; `Dokument` entries are not
  clickable.

Severities are shown in German as `Fehler` (error), `Warnung`/`Warnungen`
(warning) and `Hinweis`/`Hinweise` (information). Each severity has its own
icon. Severity is never conveyed by colour alone, and screen readers announce
it with the finding.

The headline counts the findings per severity, in the order Fehler,
Warnungen, Hinweise. Severities without findings are left out, e.g.
`1 Fehler, 1 Warnung, 1 Hinweis`. With only notes it reads `Keine Fehler,
1 Hinweis`, and without findings `Keine Befunde`. If the rules do not apply
to the profile, it shows the not-applicable reason (0009).

A finding is attributed to the first of these that exists:

1. the field whose location equals the finding's location;
2. among the fields located within the finding's location, a field of the
   first business term in the finding's `business_terms` that has one;
3. the section whose element contains the finding's location (e.g. the
   invoice line `Position 2`);
4. otherwise the document.

When the user opens another invoice while validation is still running, the
result of the earlier invoice is discarded and never shown. For invoices
without a field model (UBL until 0006), validation still runs, and all
findings belong to `Dokument`.

## Acceptance criteria

- [ ] Given CII sample A with BT-112 changed to `1300.00` (variant of 0009),
      when it is opened, then the BT-112 field shows an error marker with
      `BR-CO-15` and its official message, the BT-115 field shows `BR-CO-16`,
      and the header of section "Summen (BG-22)" shows `2 Fehler`.
- [ ] Given CII sample A with BT-131 of line 2 removed, then `BR-24` is shown
      at the header of section "Position 2" and not at line 1's BT-131.
- [ ] Given CII sample B declaring XRechnung version 2.3 and without BT-10
      (variants of 0010), then the status line and the summary show the
      headline `1 Fehler, 1 Warnung, 1 Hinweis`.
  - The summary lists, in this order: `BR-DE-15` with `Dokument`,
    `BR-DE-21` with `BT-24 Spezifikationskennung`, and `BR-DE-TMP-32` with
    `Dokument`.
  - The applied rule sets `EN 16931 CII 1.3.17` and `XRechnung 3.0.2` are
    shown.
  - The note `Geprüft nach XRechnung 3.0.2, Rechnung erklärt Version 2.3` is
    shown.
- [ ] Given the user clicks a summary entry that belongs to a field, then
      the field pane scrolls so that the field is visible, and the field is
      highlighted for at least 2 s.
  - If the XML view is shown, the field view is shown first.
  - Clicking the status line scrolls to the summary.
- [ ] Given CII sample A without BT-24 (variant of 0009), then `BR-01`
      appears in the summary with `Dokument` and at no field.
- [ ] Given CII sample A, then the headline is `Keine Befunde` and the
      applied rule set `EN 16931 CII 1.3.17` is shown.
- [ ] Given sample A with BT-24 `urn:factur-x.eu:1p0:minimum`, then the
      not-applicable reason is shown instead of findings.
- [ ] Given CII sample B, then the headline is `Keine Fehler, 1 Hinweis`.
- [ ] While validation runs, the status line and the summary show `Prüfe…`,
      and the PDF and field panes can be scrolled. For sample L the window
      never freezes for more than 100 ms (manual check: scroll visibly while
      validation runs).
- [ ] Given the user opens sample A while sample L is still being validated,
      then only the result of sample A is shown.
- [ ] Given validation ends without a result (e.g. the background task
      fails), then the status line and the summary show `Prüfung
      fehlgeschlagen` with the reason, and fields and PDF are still shown.
- [ ] Given UBL sample A with BT-1 removed (variant of 0009), then `BR-02`
      appears in the summary with `Dokument`, although no field model is
      shown yet.

## Out of scope

- Filtering or exporting findings, a validation report as PDF.
- Explanations or fix suggestions beyond the official message.
- Jumping from a finding to the XML view (0014).

## Notes

- GUI-bound criteria are verified manually with the synthetic samples and
  variants from 0009 and 0010.
- The attribution rule (including the line-2 case) is covered by unit tests
  in the core.
- The headline and status texts for all states (running, failed, not
  applicable, counts) are covered by unit tests of the app's text functions.
- Expected findings of the variants were checked with the validator on
  2026-11-17:
  - BT-112 = 1300.00 gives `BR-CO-15` (location: root) and `BR-CO-16`
    (location: monetary summation).
  - Line 2 without BT-131 gives `BR-24` at line 2, plus `BR-CO-10` and
    `BR-S-08`.
- Validation of the first invoice per syntax includes compiling the rules,
  which takes up to about 0.6 s on the reference machine (ADR 0002).
