---
id: 0018
title: App - user interface in German and English, following the system language
status: reviewed
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
      first supported preferred language); given `LANG=C` or `LANG=POSIX`,
      the app is in English.
- [ ] Given the Factur-X PDF of sample A
      (`target/manual-0013/a-mit-lieferschein.pdf`) opened in English, the
      status line reads `No findings`, the view switch reads `Fields` / `XML`,
      the field sections and values are the English ones of 0017, the PDF bar
      reads `Page 1 of 1` (German: `Seite 1 von 1`), and the summary reads
      `Checked against EN 16931 CII 1.3.17` (German: `Geprüft nach EN 16931
      CII 1.3.17`); for CII sample B it reads `Checked against EN 16931 CII
      1.3.17 and XRechnung 3.0.2`.
- [ ] Given `target/manual-0011/b-version-2-3-without-bt10.xml` opened in
      English, the status line reads `1 error, 1 warning, 1 note` (German:
      `1 Fehler, 1 Warnung, 1 Hinweis`).
- [ ] Counts read, in English and German:

      | Findings | English | German |
      |---|---|---|
      | none | `No findings` | `Keine Befunde` |
      | 2 warnings | `No errors, 2 warnings` | `Keine Fehler, 2 Warnungen` |
      | 1 note | `No errors, 1 note` | `Keine Fehler, 1 Hinweis` |
      | 3 errors, 2 notes | `3 errors, 2 notes` | `3 Fehler, 2 Hinweise` |
- [ ] Error and notes in English, for example: opening a text file named
      `notiz.txt` with the content `Hallo` shows `The file notiz.txt cannot
      be read as an invoice: XML is not well-formed (line 1)` (German: `Die
      Datei notiz.txt kann nicht als Rechnung gelesen werden: XML ist nicht
      wohlgeformt (Zeile 1)`); dropping two files shows `Only one file per window – the first one
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
- [ ] The installed desktop entry has `Name=E-Invoice Viewer`,
      `GenericName=Invoice viewer`, `Comment=View and check electronic
      invoices (ZUGFeRD, Factur-X, XRechnung)`, and German ones (`GenericName[de]=
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
  unsupported system languages, and accepted the recommendations of the
  review (PDF sample for the PDF bar, concrete desktop entry, error and count
  texts, the rule set line, `LANG=C`).
- The German texts stay exactly as they are today.
- All time limits of earlier specs (opening within 2 s, …) still apply in
  both languages.
