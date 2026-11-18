---
id: 0019
title: App - switch the language in the app
status: in-progress
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
  (0017, 0018) of every open window **immediately**, without a restart. Each
  window keeps its open invoice, its view (fields or XML), its PDF page and
  zoom, and the size of the window and its panes. The fields may scroll back
  to the top.
- **The choice is remembered** for the next start, in the app's configuration
  directory, independently of the window size and recent files (0012, 0015):
  if one of these settings cannot be read, the others still work. *System
  language* means the behaviour of 0018.
- **Keyboard**: the language choice can be reached and used without a mouse.
- **During validation**: if the language is changed while an invoice is being
  validated, the result is shown in the new language. Notes (toasts) that are
  already shown may stay in the old language.
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
      and the amount `1.249,50 EUR` within 1 s, without a restart; a second open
      window changes as well.
  - The window keeps sample A open, stays in the XML view if it was shown,
    and keeps the PDF zoom (e.g. `150 %`) and the page shown.
- [ ] Given the choice `Deutsch` on an English system, when the app is
      started again, then it is in German; when the user chooses *System
      language*, the next start is in English again.
- [ ] Given the stored language `xx` or a damaged settings file, the app
      starts in the system language, logs exactly one warning, shows no
      error dialog, and the recent files list and window size of 0012/0015
      still work.
- [ ] With the keyboard only: F10 opens the main menu, the arrow keys reach
      the language choice, Enter selects `English`, and the window is in
      English.
- [ ] Given a validation still running for `target/manual-0011/l.xml`, when
      the user switches to English, then once the validation is done the
      status line and summary read exactly as when `l.xml` is opened in
      English from the start.
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

- Review on 2026-11-18: the author decided that the switch takes effect
  immediately in all windows.
- Where the choice lives in the window (e.g. a main menu in the header bar)
  is left to the implementation.
- The review on 2026-11-18 added keyboard use and the switch during a running
  validation.
- Like notes (toasts), a message already shown in the start view (e.g. that a
  file could not be opened) stays in the previous language after a switch.

## Progress

- **Done**:
  - `crates/app/src/language_setting.rs`: the choice (system language,
    German, English), its labels (each language named in itself), and the
    store `~/.config/e-invoice-viewer/language` through the shared settings
    store; a damaged or unknown entry gives the system language and one
    warning.
  - Main menu button in the header bar (opens with F10) with the section
    "Language" and the stateful action `app.language` (`system`, `de`,
    `en`). Changing it stores the choice, sets the language of the user
    interface and shows every window again in the new language.
  - Window, XML view and PDF pane set their texts in one `apply_texts`,
    used at start and on a switch. The open document is kept and shown
    again from its bytes in the new language and validated again; the PDF
    is not reloaded (page and zoom stay) and the XML view stays chosen. The
    recent files menu is rebuilt.
- **Verified** (2026-11-18):
  - `cargo test --workspace`: 200 passed; new: 5 tests in
    `language_setting.rs` (AC1, AC3–AC5).
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
    warnings` and the architecture check are clean.
  - Smoke run of the release build on an English system, read through
    AT-SPI, switching through the application action (as the menu does):
    with the Factur-X PDF of sample A and `b.xml` in two windows, one second
    after switching to German both show `Keine Befunde` /
    `Keine Fehler, 1 Hinweis`, `Verkäufer (BG-4)`, `1.249,50 EUR`,
    `Seite 1 von 1` and `Vergrößern (Strg++)`; the store reads
    `choice=de`; the next start is German; choosing the system language
    switches back to English, also for the next start; a stored `xx` gives
    English and one warning.
- **Next** — manual checks by the author, then the author decides on
  `implemented`:
  - Open the main menu with the mouse and with F10 / arrow keys / Enter;
    the current choice is marked.
  - In the XML view with the PDF at 150 % on page 2 of
    `target/manual-0016/3-seiten.pdf`… (no invoice: use the Factur-X PDF of
    sample A at 150 %): switching keeps the XML view, zoom and page.
  - Switch while `target/manual-0011/l.xml` is still being validated: the
    result appears in the new language.

