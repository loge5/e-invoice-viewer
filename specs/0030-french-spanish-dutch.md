---
id: 0030
title: User interface and invoice content in French, Spanish and Dutch
status: draft
created: 2026-11-20
depends_on: [0017, 0018, 0019, 0022]
---

## Context

The app speaks German and English (0017–0019). E-invoices under EN 16931
are exchanged all over Europe, and French, Spanish and Dutch reach the
largest groups of users after German and English. For 1.0 the app offers
these three languages as well (author, 2026-11-20).

There is no one in the project who can check the translations as a native
speaker. They are made by Claude and shipped with a note that they are not
checked by native speakers, inviting corrections (author, 2026-11-20).

## Requirement

- **Three more languages**: French (`fr`), Spanish (`es`) and Dutch (`nl`),
  with everything that exists in German today (0017, 0018): the user
  interface, field labels and section titles, names of code lists and
  units, the app's own validation notes, messages and dialogs.
- **Unchanged rules of 0017**: business term numbers, rule IDs, raw values,
  locations and the attribution of findings are the same in every language;
  official rule messages stay exactly as published (CEN rules in English,
  XRechnung `BR-DE-*` in German).
- **Terminology**: field labels follow the established EN 16931 terms of
  each language where a public source exists (French: the Factur-X
  specification; Dutch: the NLCIUS / SI-UBL documentation); otherwise, and
  for Spanish, a faithful translation of the official English name. Labels
  stay short like the German ones (the section gives the context).
- **Numbers and dates** follow the language:

  | Language | Amount | Date |
  |---|---|---|
  | French | `1 249,50 EUR` (narrow no-break space U+202F as thousands separator) | `DD/MM/YYYY` |
  | Spanish | `1.249,50 EUR` | `DD/MM/YYYY` |
  | Dutch | `1.249,50 EUR` | `DD-MM-YYYY` |

- **Language choice** (0019): the menu offers *system language*, then
  `Deutsch`, `English`, `Español`, `Français`, `Nederlands`, each named in
  its own language. The system language (0018) now also selects French,
  Spanish or Dutch; any other language still falls back to English.
- **Desktop and installer**: the desktop entry's generic name and the
  AppStream summary and description exist in the three languages; the
  Windows installer offers them as installer languages.
- **Completeness** as in 0017: every catalogue is complete and its
  placeholders and plural forms match the English text; a missing or broken
  translation is found by a test, not by a user.
- **Note on quality**: README and CHANGELOG say that the French, Spanish and
  Dutch translations are not checked by native speakers and how to report
  a correction.

## Acceptance criteria

- [ ] Given CII sample A in French, then the status line, the section titles
      (e.g. `Vendeur (BG-4)`) and the amount `1 249,50 EUR` with U+202F are
      French; the same in Spanish (`1.249,50 EUR`) and Dutch, with dates
      `17/11/2026` (French, Spanish) and `17-11-2026` (Dutch).
- [ ] Given CII sample D, then in each of the three languages every field
      has a label in that language that differs from the English one or is
      listed as identical on purpose (e.g. `IBAN`), and every section title
      as well.
- [ ] Given findings of CEN and XRechnung rules, then their messages are the
      published ones in every language; the app's own notes (profile not
      applicable, declared XRechnung version) are translated.
- [ ] The language menu shows the six entries in the order of the
      requirement; choosing `Français` switches every open window at once
      (as 0019); the choice is remembered.
- [ ] Given the system languages `fr_FR.UTF-8`, `es_ES`, `nl_BE` or `nl`,
      then the app starts in that language; `it_IT` still gives English.
- [ ] The catalogue check passes for `de`, `fr`, `es` and `nl`: no missing
      or fuzzy entry, no entry that is not in the sources, placeholders and
      plural forms consistent (`msgfmt --check`).
- [ ] At the narrowest window size of 0015 no text of the three languages is
      cut off in the header bar, the status line or the menus (checked once
      in the GUI test or by screenshot).
- [ ] The desktop entry and the AppStream file validate with
      `desktop-file-validate` and `appstreamcli validate` and contain the
      three languages; the installer script lists `French`, `Spanish` and
      `Dutch`.
- [ ] README and CHANGELOG contain the note on quality.

## Out of scope

- Translations of the official rule messages (none are published).
- Further languages (Italian, Polish …); the structure allows adding them
  the same way.
- Translating the README and the project documentation.
- Names for code lists that are shown raw today (0024).

## Notes

- The catalogues are `po/fr.po`, `po/es.po` and `po/nl.po`, built and
  embedded like `po/de.po`.
- Plural forms: French `n > 1` (0 is singular), Spanish and Dutch `n != 1`.
- Dutch in Belgium (`nl_BE`) uses the same texts and formats.
