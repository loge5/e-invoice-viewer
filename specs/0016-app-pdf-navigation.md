---
id: 0016
title: App - navigate and zoom multi-page PDFs
status: implemented
created: 2026-11-16
depends_on: [0007]
---

## Context

Many invoices have several pages (cover letter, line item pages, terms), and
small print must be readable. Split off from 0007 to keep that spec small.

## Requirement

In the PDF pane the user can scroll through all pages of the PDF, see which
page is visible, and zoom in and out. A newly opened PDF is shown fitted to
the pane width.

## Acceptance criteria

Time limits refer to the reference machine in
[requirements](../docs/requirements.md).

- [x] Given a 3-page invoice PDF is opened, then all 3 pages are shown one
      below the other with a visible gap, and scrolling reaches the end of
      page 3.
- [x] Given the 3-page PDF, a page indicator shows `Seite 1 von 3` initially,
      and `Seite 2 von 3` once page 2 fills most of the visible area.
- [x] Given a newly opened PDF, then its pages are scaled to fit the pane
      width; when the divider is dragged, the pages are re-scaled to the new
      width while fit-width is active.
- [x] Given the PDF pane has focus, Ctrl++ zooms in and Ctrl+- zooms out in
      steps (100 %, 125 %, 150 %, 200 %, 300 %, 400 % and 75 %, 50 %, 25 %),
      Ctrl+0 returns to fit width; the zoom level is shown (e.g. `150 %`), and
      zooming never goes beyond 25 % or 400 %.
  - The buttons − and + next to the zoom level do the same as Ctrl+- and
    Ctrl++; at 25 % or 400 % the respective button is disabled.
- [x] Given page 2 is visible and the user zooms to 200 %, then page 2 stays
      in view (the visible page does not jump).
- [x] Given a 50-page PDF, opening it shows page 1 within 2 s, and scrolling to
      page 50 shows its content within 1 s after scrolling stops; text at
      400 % is rendered sharply (not an enlarged low-resolution image).

## Out of scope

- Text selection, search, thumbnails, printing, rotation.
- Remembering zoom level between files or sessions.

## Notes

- GUI-bound; verified manually with a synthetic 3-page and 50-page PDF.
- 100 % means 96 logical pixels per inch (an A4 page is about 794 logical
  pixels wide). Fit width shows its effective zoom, e.g. `73 %`; Ctrl++
  and Ctrl+- go to the neighbouring step from there.
- Change on 2026-11-18 at the author's request: buttons − and + next to the
  zoom level, because the keyboard shortcuts are hard to discover.

## Progress

- **Done**:
  - `crates/app/src/pdf_layout.rs` (pure logic): zoom steps, page stack
    with gaps, the visible page, the scroll anchor kept across zoom changes,
    and the device-pixel render width.
  - `crates/app/src/pdf_pane.rs`: all pages one below the other with gaps
    and a shadow, a bar with `Seite X von N` and the zoom level. Pages are
    split into strips of 512 logical pixels, rendered after scrolling stops
    (60 ms) only near the visible area, at device resolution; strips far
    away are released. Fit width re-fits when the pane width changes.
    Ctrl++/Ctrl+= (and keypad +), Ctrl+- and Ctrl+0 work while the PDF pane
    has the focus (a click on the pages gives it the focus).
  - Buttons − and + next to the zoom level (tooltip with the shortcut,
    accessible name), disabled at the smallest and largest step.
  - `pdf_view.rs` renders a strip of a page and wraps it as a paintable of
    logical size, so that textures with device pixels are shown sharply
    without subclassing (the workspace forbids unsafe code).
  - Test PDFs with 3 and 50 pages (`multi_page_pdf` in the test helpers).
- **Verified** (2026-11-18):
  - `cargo test --workspace`: 171 passed; new: 4 tests in `pdf_layout.rs`
    and `pdf_navigation_texts`.
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
    warnings` and the architecture check are clean.
  - Smoke run with `target/manual-0016/50-seiten.pdf` (written by `cargo
    test -p e-invoice-viewer-core --test open_document -- --ignored
    write_manual_samples_0016`): the indicator shows `Seite 1 von 50` about
    0.4 s after starting the app (including start-up and polling), `Seite 25
    von 50` and `Seite 50 von 50` after scrolling to the middle and the end
    through AT-SPI, and the fit-width zoom `73 %`.
  - Zoom buttons through AT-SPI with `3-seiten.pdf`: + goes from `73 %` to
    `75 %` and `100 %`; repeated − ends at `25 %`.
- **Manually verified** by the author (2026-11-18):
  - AC1–AC6: pages, indicator, fit width, zoom by keys and buttons, the
    page kept in view and the 50-page PDF as expected.
