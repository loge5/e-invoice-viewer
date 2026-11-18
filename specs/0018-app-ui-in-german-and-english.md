---
id: 0018
title: App - user interface in German and English, following the system language
status: draft
created: 2026-11-18
depends_on: [0017]
---

## Context

The app becomes multilingual, starting with German and English (decision of
the author on 2026-11-18, see [ADR 0003](../docs/adr/0003-translations-gettext-embedded.md)).
Spec 0017 makes the invoice content available in both languages. This spec
makes the whole user interface speak the language of the user's desktop, so
that the app can be used by people who do not read German, now that it is
published.

## Requirement

- **Every text of the app's user interface** exists in German and English:
  window and header bar, start view, buttons, menus, tooltips, accessible
  names, the status line and summary, the XML view with its assignment line,
  the PDF bar, the attachments section, notes (toasts) and error messages.
  The invoice content follows the same language (0017).
- **The language follows the system** when the app starts: the first of the
  user's preferred languages that the app supports. If none is supported, the
  app is in **English**.
- **Counts use correct singular and plural forms** in both languages.
- **The desktop entry** (name, generic name, comment shown in the app grid and
  under "Open with") exists in German and English.
- **Completeness**: no user interface text exists in only one language; a
  missing translation is detected before a release.
- The app name `E-Invoice Viewer` is not translated.

## Acceptance criteria

- [ ] Given the system language German (`LANG=de_DE.UTF-8`), the start view
      shows `Rechnung öffnen` and the button `Öffnen…`; given English
      (`LANG=en_US.UTF-8`), it shows `Open an invoice` and `Open…`.
- [ ] Given `LANG=fr_FR.UTF-8`, the app is in English; given
      `LANGUAGE=fr:de` with `LANG=fr_FR.UTF-8`, the app is in German (the
      first supported preferred language).
- [ ] Given CII sample A opened in English, the status line reads
      `No findings`, the view switch reads `Fields` / `XML`, the field
      sections and values are the English ones of 0017, and the PDF bar reads
      `Page 1 of 1`.
- [ ] Given `target/manual-0011/b-version-2-3-without-bt10.xml` opened in
      English, the status line reads `1 error, 1 warning, 1 note` (German:
      `1 Fehler, 1 Warnung, 1 Hinweis`), and a summary with two warnings
      reads `2 warnings` (German: `2 Warnungen`).
- [ ] Error and notes in English, for example: opening a text file named
      `notiz.txt` shows `The file notiz.txt cannot be read as an invoice: …`
      (German: `Die Datei notiz.txt kann nicht als Rechnung gelesen werden:
      …`); dropping two files shows `Only one file per window – the first one
      was opened`.
- [ ] Attachments in English: a 24 600-byte PDF attachment shows
      `24.6 kB` (German: `24,6 kB`); a section without attachments reads
      `No attachments`.
- [ ] The XML view in English: the assignment line reads `No field assigned`
      outside any field and `BT-131 Invoice line net amount · Line 2`
      on the net amount of line 2 of sample A (German: `BT-131 Nettobetrag ·
      Position 2`); the buttons read `Copy XML`, `Save XML as…` and
      `Show in fields`.
- [ ] Accessible names and tooltips are translated: in English the zoom
      buttons are named `Zoom in (Ctrl++)` and `Zoom out (Ctrl+-)`, the recent
      files button `Recently opened`.
- [ ] The installed desktop entry has `Name=E-Invoice Viewer`, an English
      generic name and comment, and German ones (`GenericName[de]=
      Rechnungsbetrachter`, `Comment[de]=Elektronische Rechnungen (ZUGFeRD,
      Factur-X, XRechnung) ansehen und prüfen`); `desktop-file-validate`
      reports no error.
- [ ] An automated check fails if a user interface text has no German
      translation, or if a German translation is outdated or marked fuzzy.

## Out of scope

- Switching the language inside the app (0019).
- Texts shown by GTK, libadwaita or the desktop portal themselves (e.g. the
  file chooser); they follow the system language.
- Languages other than German and English.
- Translating the README and project documentation.

## Notes

- Review on 2026-11-18: the author decided on English as the fallback for
  unsupported system languages.
- The German texts stay exactly as they are today.
- All time limits of earlier specs (opening within 2 s, …) still apply in
  both languages.
