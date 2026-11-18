# 0003 — Translations: gettext catalogues embedded in the binary, explicit language

- Status: proposed
- Date: 2026-11-18
- Specs: [0017](../../specs/0017-core-content-per-language.md),
  [0018](../../specs/0018-app-ui-in-german-and-english.md),
  [0019](../../specs/0019-app-language-switch.md)

## Context

The author decided on 2026-11-18 that the app becomes multilingual, starting
with German and English:

- the language follows the system, and can be switched in the app; the choice
  is remembered;
- everything the app itself produces is translated: user interface, field
  labels, section titles, code lists, units, its own messages, and the number
  and date formats. The official rule messages stay as published (CEN in
  English, XRechnung `BR-DE-*` in German);
- translations are maintained as gettext `.po` files, so that translators can
  use the usual tools (Poedit, Weblate) and further languages need no code
  change.

Constraints from the project: one self-contained binary installed by
`scripts/install.sh`, `cargo run` must work without installing anything,
`crates/core` stays free of GUI dependencies, and the language must be
switchable while the app runs.

## Decision (proposed)

1. **Source strings are English.** All texts in the code are written in
   English and marked for translation; `po/de.po` holds the German
   translation. English needs no catalogue and is the fallback for anything
   untranslated.
2. **Catalogues are embedded.** At build time each `po/<lang>.po` is compiled
   to a binary catalogue (`msgfmt`, GNU gettext) and embedded into the
   executable. At run time the catalogues are read with the pure-Rust
   [`gettext`](https://crates.io/crates/gettext) crate (MIT). Nothing is
   installed besides the binary, and `cargo run` sees the same translations.
3. **The language is explicit, not global.** A small i18n module in
   `crates/core` knows the available languages and their catalogues. Code that
   produces text receives the language it should use; there is no process-wide
   locale switch (`setlocale`). This makes switching languages at run time a
   matter of producing the texts again, and lets tests check both languages
   side by side.
4. **Number and date formats are part of the language**, implemented in the
   core display rules (German `1.050,00` and `17.11.2026`, English `1,050.00`
   and `2026-11-17`), not taken from the system's `LC_NUMERIC` / `LC_TIME`.
5. **Language resolution**: the language chosen in the app; otherwise the
   first supported language of the system's preferred languages (as GLib
   reports them from `LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, `LANG`); otherwise
   English.
6. **Tooling**: a script extracts the marked strings into `po/messages.pot`
   (`xgettext`); a test fails if `po/de.po` misses a string or has a fuzzy or
   stale entry, so that a release never ships untranslated German.

## Alternatives

| Option | Why not |
|---|---|
| `gettext-rs` (bindings to GNU libintl) with installed `.mo` files | The GNOME standard, but it depends on the process locale and on catalogues installed under a `locale/` directory: `cargo run` and `~/.local/bin` need extra path handling, and switching the language at run time means changing environment variables and working around libintl's caching |
| Fluent (`fluent-rs`, `.ftl` files) | Good plural and grammar support, but not the format the author chose and less common in Linux desktop translation tooling |
| Translation tables in Rust code | No dependency and compiler-checked, but declined by the author: further languages need code changes and translators get no tools |

## Consequences

- **New dependency** `gettext` (pure Rust, MIT, no native library) — needs the
  author's approval (AGENTS.md "ask first").
- **New build dependency**: GNU gettext (`msgfmt`, `xgettext`) must be
  installed to build; it is part of every common distribution (`gettext`
  package). The README lists it.
- `crates/core` gets the i18n module; it remains GUI-free and licence-neutral.
- Texts that GTK, libadwaita or the desktop portal show themselves (e.g. the
  file chooser) follow the **system** language, not the language chosen in
  the app. This is accepted.
- The desktop entry (`Name`, `Comment`) carries German and English variants
  written by the install script.
