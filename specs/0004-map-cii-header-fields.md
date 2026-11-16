---
id: 0004
title: Map CII invoice header data to the field model
status: draft
created: 2026-11-16
depends_on: [0003]
---

## Context

The core value of the viewer is a readable, structured view of the invoice
content, where every value is labelled with its EN 16931 business term (BT)
or group (BG) number. Users compare the fields with the PDF and with
validation findings, which refer to BT numbers. CII is the syntax of all
ZUGFeRD / Factur-X PDFs, so it comes first.

## Requirement

Given a CII invoice, the system produces a field model: an ordered list of
sections, each containing fields. Every field carries its BT/BG number, the
German label, the raw value from the XML, a display value for humans, and the
location of its element in the XML. Fields that are absent in the invoice are
not included. This spec covers the document header, the parties, payment and
totals. Line items follow in 0005.

## Contracts (optional)

Sections in this order (fields listed are the minimum set):

| Section (German title) | Fields |
|---|---|
| Rechnung (Kopf) | BT-1, BT-2, BT-3, BT-5, BT-9, BT-10, BT-24, BT-22 (repeatable) |
| Verkäufer (BG-4) | BT-27, BT-29, BT-31, BT-32, BG-5 (BT-35, BT-36, BT-38, BT-37, BT-40), BG-6 (BT-41, BT-42, BT-43) |
| Käufer (BG-7) | BT-44, BT-46, BT-48, BG-8 (BT-50, BT-51, BT-53, BT-52, BT-55) |
| Zahlung (BG-16) | BT-81, BT-83, BT-84, BT-85, BT-86, BT-20 |
| Umsatzsteuer (BG-23, repeatable) | BT-116, BT-117, BT-118, BT-119, BT-120 |
| Summen (BG-22) | BT-106, BT-107, BT-108, BT-109, BT-110, BT-112, BT-113, BT-114, BT-115 |

Each field: `id` (e.g. `BT-1`), `label` (German), `raw` (string as in the
XML), `display` (formatted string), `location` (XPath of the element, with
position predicates for repeated elements).

## Acceptance criteria

- [ ] Given the sample CII invoice with invoice number `RE-2026-0042`, issue
      date `20261117` (format 102), type code `380` and currency `EUR`, when it
      is mapped, then section "Rechnung (Kopf)" contains, in this order:
      `BT-1` raw `RE-2026-0042`; `BT-2` raw `20261117` display `17.11.2026`;
      `BT-3` raw `380` display `Rechnung (380)`; `BT-5` raw `EUR`.
- [ ] Given seller name `Muster GmbH`, VAT ID `DE123456789` and postal address
      `Hauptstraße 1`, `12345`, `Berlin`, `DE`, then section "Verkäufer (BG-4)"
      contains BT-27 `Muster GmbH`, BT-31 `DE123456789`, BT-35
      `Hauptstraße 1`, BT-38 `12345`, BT-37 `Berlin`, BT-40 `DE`.
- [ ] Given payment means code `58` and IBAN `DE02120300000000202051`, then
      section "Zahlung (BG-16)" contains BT-81 display `SEPA-Überweisung (58)`
      and BT-84 display `DE02 1203 0000 0000 2020 51`.
- [ ] Given totals line sum `1000.00`, VAT total `190.00`, grand total
      `1190.00` and amount due `1190.00` in currency `EUR`, then section
      "Summen (BG-22)" contains BT-106 display `1.000,00 EUR`, BT-110 display
      `190,00 EUR`, BT-112 display `1.190,00 EUR`, BT-115 display
      `1.190,00 EUR`.
- [ ] Given two VAT breakdowns (category `S` 19 % on `1000.00`, and category
      `E` 0 % on `50.00` with exemption reason `Kleinunternehmer`), then there
      are two "Umsatzsteuer (BG-23)" sections in document order, the first with
      BT-118 display `Normalsatz (S)` and BT-119 display `19 %`, the second
      with BT-118 display `Steuerbefreit (E)` and BT-120 `Kleinunternehmer`.
- [ ] Given an invoice without a buyer reference (BT-10) and without notes
      (BT-22), then neither BT-10 nor BT-22 appear in the model, and all other
      fields are still present.
- [ ] Every field's `location` selects exactly the element its raw value was
      read from (checked for BT-1, BT-84 and the second BG-23's BT-119 in the
      two-breakdown sample).

## Out of scope

- Line items (0005), UBL (0006).
- Fields of the Extended profile beyond EN 16931.
- Business terms not listed in the contract table; they can be added in later
  specs. They must not cause errors when present.
- Validation of values (0009). Invalid values (e.g. date `2026-13-45`) are
  shown with `display` equal to `raw`.

## Notes

- German labels: the KoSIT XRechnung visualization ships German BT names under
  Apache 2.0 and can serve as the reference for wording.
- Code-list names are required only for BT-3, BT-81 and BT-118 (and BT-151 in
  0005). Other codes are shown as raw values.
