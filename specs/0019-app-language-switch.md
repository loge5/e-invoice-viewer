---
id: 0019
title: App - switch the language in the app
status: draft
created: 2026-11-18
depends_on: [0018]
---

## Context

The app follows the system language (0018). The author decided on 2026-11-18
that the language can also be switched in the app and that the choice is
remembered (see [ADR 0003](../docs/adr/0003-translations-gettext-embedded.md)):
for example to check an invoice for an English-speaking colleague, or to use
the app in German on an English desktop.

## Requirement

- **A language choice in the app** offers: *system language*, `Deutsch` and
  `English`. Each language is named in its own language, so that a user who
  cannot read the current language still finds theirs. The current choice is
  marked.
- **Choosing a language** changes the user interface and the invoice content
  (0017, 0018) of every open window. [NEEDS CLARIFICATION: immediately,
  keeping each window's open invoice, scroll position, view (fields or XML)
  and zoom — or only for windows opened afterwards / after a restart?]
- **The choice is remembered** for the next start, in the app's configuration
  directory, independently of the window size and recent files (0012, 0015):
  if one of these settings cannot be read, the others still work. *System
  language* means the behaviour of 0018.
- **Robustness**: an unknown or damaged stored language falls back to the
  system language, with one warning in the log and no error dialog.
- Choosing a language never opens a network connection and never writes
  anything except the setting.

## Acceptance criteria

- [ ] Given the system language English and no stored choice, the language
      choice shows *System language* as selected and offers `Deutsch` and
      `English`.
- [ ] Given CII sample A open in English, when the user chooses `Deutsch`,
      then the window shows `Keine Befunde`, the section `Verkäufer (BG-4)`
      and the amount `1.249,50 EUR` [NEEDS CLARIFICATION: timing, see
      Requirement]; a second open window changes as well.
- [ ] Given the choice `Deutsch` on an English system, when the app is
      started again, then it is in German; when the user chooses *System
      language*, the next start is in English again.
- [ ] Given the stored language `xx` or a damaged settings file, the app
      starts in the system language, logs exactly one warning, shows no
      error dialog, and the recent files list and window size of 0012/0015
      still work.
- [ ] The setting is stored only in the app's configuration directory and
      contains no file names; it is written atomically like the settings of
      0015.
- [ ] Texts of GTK and the desktop portal (e.g. the file chooser) stay in the
      system language; this is expected and not a defect.

## Out of scope

- Languages other than German and English (the choice lists every language
  the app has translations for).
- Following a change of the system language while the app runs.
- Choosing number or date formats separately from the language.

## Notes

- Where the choice lives in the window (e.g. a main menu in the header bar)
  is left to the implementation; it must be reachable with the keyboard.
