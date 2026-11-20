---
id: 0030
title: User interface and invoice content in every official EU language
status: in-progress
created: 2026-11-20
depends_on: [0017, 0018, 0019, 0022]
---

## Context

The app speaks German and English (0017–0019). E-invoices under EN 16931
are exchanged all over Europe. For 1.0 the app offers every official
language of the EU (author, 2026-11-20; first French, Spanish and Dutch,
extended to all 24 languages the same day).

There is no one in the project who can check the translations as a native
speaker. They are made by Claude and shipped with a note that they are not
checked by native speakers, inviting corrections (author, 2026-11-20).

## Requirement

- **The 24 official EU languages**: Bulgarian (`bg`), Spanish (`es`),
  Czech (`cs`), Danish (`da`), German (`de`), Estonian (`et`), Greek
  (`el`), English (`en`), French (`fr`), Irish (`ga`), Croatian (`hr`),
  Italian (`it`), Latvian (`lv`), Lithuanian (`lt`), Hungarian (`hu`),
  Maltese (`mt`), Dutch (`nl`), Polish (`pl`), Portuguese (`pt`), Romanian
  (`ro`), Slovak (`sk`), Slovenian (`sl`), Finnish (`fi`) and Swedish
  (`sv`). Each new language has everything that exists in German today
  (0017, 0018): the user interface, field labels and section titles, names
  of code lists and units, the app's own validation notes, messages and
  dialogs.
- **Unchanged rules of 0017**: business term numbers, rule IDs, raw values,
  locations and the attribution of findings are the same in every language;
  official rule messages stay exactly as published (CEN rules in English,
  XRechnung `BR-DE-*` in German).
- **Terminology**: field labels follow the established EN 16931 and
  invoicing terms of each language (e.g. French: the Factur-X
  specification; Dutch: the NLCIUS / SI-UBL documentation); otherwise a
  faithful translation of the official English name. Labels stay short
  like the German ones (the section gives the context).
- **Plural forms** follow each language's gettext rule (table in Notes), so
  that counts read correctly (e.g. Polish 1, 2–4 and 5+ differ).
- **Numbers and dates** follow the language (CLDR conventions), for invoice
  values and for the numbers of the app itself (attachment sizes such as
  `1,5 kB`). For the amount 1249.50 EUR and the date 2026-11-17:

  | Languages | Amount | Date |
  |---|---|---|
  | en | `1,249.50 EUR` | `2026-11-17` |
  | ga, mt | `1,249.50 EUR` | `17/11/2026` |
  | de, da | `1.249,50 EUR` | `17.11.2026` |
  | ro, sl | `1.249,50 EUR` | `17.11.2026` |
  | hr | `1.249,50 EUR` | `17.11.2026.` |
  | es, it, el | `1.249,50 EUR` | `17/11/2026` |
  | nl | `1.249,50 EUR` | `17-11-2026` |
  | fr | `1 249,50 EUR` (narrow no-break space U+202F) | `17/11/2026` |
  | pt | `1 249,50 EUR` (no-break space U+00A0, as all below) | `17/11/2026` |
  | bg, cs, et, fi, pl, sk | `1 249,50 EUR` | `17.11.2026` |
  | lv | `1 249,50 EUR` | `17.11.2026.` |
  | hu | `1 249,50 EUR` | `2026.11.17.` |
  | lt, sv | `1 249,50 EUR` | `2026-11-17` |

- **Language choice** (0019): the menu offers *system language*, then the
  24 languages in the EU's protocol order, each named in its own language:
  `Български`, `Español`, `Čeština`, `Dansk`, `Deutsch`, `Eesti`,
  `Ελληνικά`, `English`, `Français`, `Gaeilge`, `Hrvatski`, `Italiano`,
  `Latviešu`, `Lietuvių`, `Magyar`, `Malti`, `Nederlands`, `Polski`,
  `Português`, `Română`, `Slovenčina`, `Slovenščina`, `Suomi`, `Svenska`.
  The menu's title and the entry *system language* are in the current
  language. The system language (0018) selects any of the 24 languages;
  any other language still falls back to English.
- **Desktop and installer**: the desktop entry's generic name and the
  AppStream summary and description (not the release notes) exist in every
  language; the Windows installer offers each of the languages that Inno
  Setup ships a translation for, the others install in English.
- **Completeness** as in 0017: every catalogue is complete and its
  placeholders and plural forms match the English text; a missing or broken
  translation is found by a test, not by a user.
- **Note on quality**: README and CHANGELOG say that the translations other
  than German and English are not checked by native speakers, and that
  corrections are welcome as an issue on GitHub
  (`github.com/loge5/e-invoice-viewer`).

## Acceptance criteria

- [x] Given CII sample A, then:

      | Language | Status line | Seller section |
      |---|---|---|
      | French | `Aucun constat` | `Vendeur (BG-4)` |
      | Spanish | `Sin observaciones` | `Vendedor (BG-4)` |
      | Dutch | `Geen bevindingen` | `Verkoper (BG-4)` |

      and in every other new language both differ from English.
- [x] Given CII sample A in each of the 24 languages, then the grand total
      (BT-112) and the issue date (BT-2) read as in the table of the
      requirement.
- [x] An attachment of 1 500 bytes is listed as `1.5 kB` in English and
      `1,5 kB` in every language with a decimal comma.
- [x] Given CII sample D, then in each new language every field has a label
      that differs from the English one or is on a short list of terms
      identical on purpose (e.g. `IBAN`, `BIC`, `XML`), and every section
      title as well.
- [x] For the counts 0, 1, 2, 3, 5, 11, 21, 22 and 101, each language picks
      the plural form its gettext rule (Notes) gives, and its catalogue has
      exactly that many forms.
- [x] Given findings of CEN and XRechnung rules, then their messages are the
      published ones in every language; the app's own notes (profile not
      applicable, declared XRechnung version) are translated.
- [x] The language menu shows the 25 entries in the order of the
      requirement, with the title and *system language* in the current
      language (French: `Langue`, `Langue du système`); choosing `Français`
      switches every open window at once (as 0019); the choice is
      remembered.
- [x] Given the system languages `fr_FR.UTF-8`, `es_ES`, `nl_BE`, `pl_PL`,
      `ga_IE`, `el_GR` or `pt_PT`, then the app starts in that language;
      `ru_RU` and `tr_TR` give English.
- [x] The catalogue check passes for all 23 catalogues: no missing or fuzzy
      entry, no entry that is not in the sources, placeholders and plural
      forms consistent (`msgfmt --check`).
- [x] At the minimum window size of 0015 (900 × 600) with CII sample A open,
      no text is cut off in the header bar, the status line, the field list
      or the menus, in the languages with the longest texts (determined by a
      script over the catalogues: the three languages with the longest
      header bar and status line texts) and in Greek and Bulgarian (other
      scripts); checked by screenshots on a private display, attached to the
      progress notes.
- [ ] The desktop entry and the AppStream file validate with
      `desktop-file-validate` and `appstreamcli validate` and contain all
      new languages; the installer script lists the Inno Setup languages
      available for them.
- [x] README and CHANGELOG contain the note on quality.

## Out of scope

- Translations of the official rule messages (none are published).
- Languages that are not official EU languages.
- Translating the README, the project documentation and the AppStream
  release notes.
- Names for code lists that are shown raw today (0024).
- Regional variants (e.g. Brazilian Portuguese, Austrian German): they use
  the texts of their language.

## Notes

- The catalogues are `po/<code>.po`, built and embedded like `po/de.po`.
- gettext plural rules (`nplurals`; formula):
  - 2; `n != 1`: bg, da, de, el, es, et, fi, hu, it, nl, pt, sv;
  - 2; `n > 1`: fr;
  - 3; `n==1 ? 0 : (n==0 || (n%100>0 && n%100<20)) ? 1 : 2`: ro;
  - 3; `(n%10==1 && n%100!=11) ? 0 : n != 0 ? 1 : 2`: lv;
  - 3; `n==1 ? 0 : (n>=2 && n<=4) ? 1 : 2`: cs, sk;
  - 3; `n==1 ? 0 : (n%10>=2 && n%10<=4 && (n%100<10 || n%100>=20)) ? 1 : 2`: pl;
  - 3; `(n%10==1 && n%100!=11) ? 0 : (n%10>=2 && n%10<=4 && (n%100<10 || n%100>=20)) ? 1 : 2`: hr;
  - 3; `(n%10==1 && n%100!=11) ? 0 : (n%10>=2 && (n%100<10 || n%100>=20)) ? 1 : 2`: lt;
  - 4; `n%100==1 ? 0 : n%100==2 ? 1 : (n%100==3 || n%100==4) ? 2 : 3`: sl;
  - 4; `n==1 ? 0 : (n==0 || (n%100>1 && n%100<11)) ? 1 : (n%100>10 && n%100<20) ? 2 : 3`: mt;
  - 5; `n==1 ? 0 : n==2 ? 1 : (n>2 && n<7) ? 2 : (n>6 && n<11) ? 3 : 4`: ga.
- Number formats follow CLDR; group separators that are spaces are
  no-break spaces (U+00A0), French uses U+202F as CLDR does. Spanish uses
  `.` as commonly on invoices in Spain (the RAE recommends a space).
- Dutch in Belgium (`nl_BE`) and other regional variants use the texts and
  formats of their language.
- Inno Setup 6 ships Bulgarian, Czech, Danish, Dutch, Finnish, French,
  German, Hungarian, Italian, Polish, Portuguese, Slovak, Slovenian, Spanish
  and Swedish; Greek, Estonian, Irish, Croatian, Latvian, Lithuanian,
  Maltese and Romanian exist only as unofficial translations and are not
  bundled.
- Review on 2026-11-20 (Claude): added the app's own numbers (attachment
  sizes, formatted per language today only for German), the language menu's
  own texts (written outside the catalogue today), concrete expected texts
  for sample A, and the issue tracker for corrections; the screenshot check
  names the size and the sample. Accepted by the author together with the
  test list.
- Extended on 2026-11-20 at the author's request from French, Spanish and
  Dutch to all 24 official EU languages: plural rules, the format table,
  the menu order (EU protocol order), installer languages limited to those
  Inno Setup ships, screenshot check limited to the longest texts and the
  other scripts. The test list was extended accordingly (T1, T5, T9).

## Test list (approved 2026-11-20, extended the same day)

- T1 `sample_a_in_every_language` (core): field model of CII and UBL
  sample A per language — seller section title, BT-112, BT-2 against the
  tables (AC1, AC2).
- T2 `sample_d_labels_are_translated` (core): every label and title of
  sample D differs from English or is on a short list of identical terms,
  per new language (AC4).
- T3 `official_messages_unchanged_own_notes_translated` (core): a CEN
  finding keeps its English message, a `BR-DE` finding its German one; the
  profile note of sample A Minimum is translated (AC6).
- T4 status line and attachment size per language (app, `texts.rs`) (AC1,
  AC3).
- T5 `language_menu_entries` and `system_language_detection` (app,
  `language_setting.rs`, core `i18n`): order, own names, title and *system
  language* per language; the system languages of AC8 (AC7, AC8).
- T6 catalogue check for all catalogues (extends
  `german_catalogue_is_complete`, renamed `every_catalogue_is_complete`)
  plus `msgfmt --check` in the build (AC9).
- T7 packaging test: desktop entry and metainfo contain every new language
  and validate; installer script lists the Inno Setup languages (AC11).
- T8 README and CHANGELOG mention the note on quality (packaging test)
  (AC12).
- T9 `plural_forms_follow_the_language_rules` (core): resolved form index
  for the counts of AC5 per language, and the number of forms in each
  catalogue (AC5).
- P1 screenshots at 900 × 600 (AC10).

## Progress

- **Done**:
  - `Language` has the 24 languages in the EU protocol order, each with one
    line of conventions (code, own name, decimal and thousands separator,
    date form); dates and numbers of the field model and the attachment
    sizes of the app use them, so no other code matches on a language.
  - `build.rs` compiles every `po/*.po` and generates the list of embedded
    catalogues; 22 new catalogues with 281 entries each.
  - Translations: French, Spanish and Dutch by Claude; the other 19 by four
    Claude subagents from one brief (style, terminology, placeholders,
    plural forms), checked by a generator script (placeholders, number of
    plural forms, no empty entry) and `msgfmt --check`. The severity
    headings (entries `Error`/`Errors` …) are headings without a number:
    their forms are the nominative singular and plural in every language.
  - Language menu from `Language::ALL` (system language, then 24 entries
    named in themselves); desktop entry and AppStream file in all languages;
    the installer lists the 15 languages Inno Setup ships.
  - README, CHANGELOG and `docs/requirements.md` with the note on quality.
- **Verified** (2026-11-20):
  - `cargo test --workspace`, fmt and clippy clean. New: `eu_languages.rs`
    (AC1, AC2, AC4, AC6), `plural_forms.rs` (AC5: every catalogue's rule,
    evaluated by the gettext library the app uses, picks the expected form
    for 0, 1, 2, 3, 5, 11, 21, 22, 101), unit tests in `i18n` (AC8, menu
    names, a catalogue per language), `status_and_size_in_every_language`
    and `language_choice_labels` in the app (AC1, AC3, AC7), the extended
    catalogue check (AC9), packaging tests (AC11 for the files, AC12).
  - AC10: a script over the catalogues found Irish, French and Bulgarian
    with the longest header bar and status line texts; screenshots at
    900 × 600 with sample A in these and in Greek on a private Broadway
    display (`target/manual-0030/`): nothing cut off; the language menu is
    taller than the window and scrolls to its last entry. Field labels
    longer than the label column (18 to 28 characters, e.g. French
    `Identifiant de spécification`) widen it and move the value to the
    right, as long German labels do.
- **Open**: AC11 for the Windows build — whether Inno Setup on the runner
  has all 15 language files is shown only by the release workflow (started
  by the author on `ci-trial`).
- **Uncertain terms** reported by the translators (candidates for
  corrections): national names of the construction invoice types
  (BT-3), the VAT point date (BT-7, BT-8), self-billed invoice, clearing between partners, standing
  agreement, registration and tax numbers (BT-30, BT-31, BT-32 per
  country), remittance information (BT-83), and the word for "findings" in
  Lithuanian, Polish and Irish.

