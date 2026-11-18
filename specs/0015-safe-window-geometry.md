---
id: 0015
title: Remember the window size safely
status: implemented
created: 2026-11-16
depends_on: [0007]
---

## Context

Users expect the window to come back in the size they left it. The previous
tool saved its window geometry without checks: after a crash during resizing
it stored a size of 1 × 38 and a position outside all screens, and from then
on the window was invisible on every start. Remembering the size must never be
able to make the app unusable.

## Requirement

The app remembers the **normal size** of a window: its size when it is
neither maximized nor tiled (e.g. GNOME half-screen tiling). It also
remembers whether the window was maximized. The window position is not stored
(the desktop places the window).

- **Saving.** The size is saved when a window is closed, and for every window
  when the app is quit (Ctrl+Q). With several windows, the window closed last
  wins.
- **Applying.** Every new window opens with the remembered size and state:
  the first one after the start as well as further windows (0012).
- **Plausibility.** A remembered size is applied only if it is plausible on
  the current screens:
  - at least the minimum size of 0007 (900 × 600);
  - at most the size of the largest connected monitor.

  Otherwise the default size 1200 × 800 is used, reduced to the largest
  monitor if that is smaller, but never below the minimum size. All sizes are
  logical pixels, i.e. after display scaling.
- **Robustness.** Saving never leaves a half-written settings file: if the app
  is killed while saving, the previous valid value is kept. Unknown entries
  in the stored settings are ignored. If the stored settings are empty or not
  readable, the default applies, no error dialog is shown, and one warning is
  logged.
- **Storage.** Settings are stored only in the app's configuration directory,
  never next to an invoice. They contain no file names of opened invoices.
  The size settings and the recent list of 0012 are independent: if one of
  them cannot be read, the other still works.

## Acceptance criteria

- [x] Given the user resizes the window to 1400 × 900 and closes it, then the
      next start opens the window at 1400 × 900.
  - Given two windows, 1400 × 900 and 1000 × 700, when 1000 × 700 is closed
    last, then the next window opens at 1000 × 700.
  - Given a window at 1400 × 900, when the user presses Ctrl+Q, then the
    next start opens at 1400 × 900.
- [x] Given the window was maximized when closed, then the next start opens it
      maximized, and un-maximizing returns to the last remembered normal size.
- [x] Given a window of 1400 × 900 that the user tiles to the left half of the
      screen and closes there, then the next start opens it untiled at
      1400 × 900.
- [x] Given a single monitor of 3840 × 2160 at 100 % scaling, the stored size
      is replaced by the default as follows:

      | stored size | next start opens at |
      |---|---|
      | 1 × 38 | 1200 × 800 |
      | -500 × 700 | 1200 × 800 |
      | 800 × 500 (below the minimum) | 1200 × 800 |
      | 9000 × 9000 | 1200 × 800 |
      | 1400 × 900 | 1400 × 900 |

  - Given the same monitor at 200 % scaling (1920 × 1080 logical), a stored
    2400 × 1200 is replaced by 1200 × 800.
  - Given a monitor of 1024 × 768 logical, the default becomes 1024 × 768.
  - Given a monitor of 800 × 600 logical, the window opens at the minimum
    size 900 × 600.
  - After closing, the actually shown size is stored.
- [x] Given the stored settings file is empty, truncated or not parseable,
      then the app starts with the default size, shows no error dialog, and
      logs one warning. The recent list of 0012 is still shown.
  - Given stored settings with an additional unknown entry, the remembered
    size is applied.
- [x] Given the app is killed while it saves the settings, then no
      half-written settings file remains, and the next start uses the
      previous valid size.
- [x] Settings are stored only in the app's configuration directory. They
      contain no file names of opened invoices.

## Out of scope

- Remembering the window position, divider position, zoom level or the
  selected view (fields or XML).
- Fullscreen mode.
- Restoring a tiled state.

## Notes

- The plausibility rule, the default calculation and the parsing of stored
  settings are unit-tested. Saving, maximizing and tiling are verified
  manually.
- The largest monitor is used because the desktop decides where a window
  appears; the app cannot know which monitor it will be.
- Review on 2026-11-18: the author decided that the window closed last wins
  and its size applies to every new window, that sizes below the minimum are
  implausible, that the normal (untiled, unmaximized) size is remembered, and
  that size settings and the recent list are independent.

## Progress

- **Done**:
  - `crates/app/src/window_size.rs`: the plausibility rule and default
    calculation (`initial_state`), and the store
    `~/.config/e-invoice-viewer/window` (GLib key file, written atomically,
    unknown entries ignored, a damaged store gives the default and one
    warning).
  - Window: every new window reads the store and opens with the plausible
    size, maximized if it was. It follows its normal size (not while
    maximized, fullscreen or tiled) and saves it with the maximized state
    when it is closed. Ctrl+Q saves every window, the active one last, so
    that it wins.
  - Found while testing: GLib key files cannot take text with NUL
    characters and the binding panics on them. Such a store is now treated
    as damaged, here and for the recent list of 0012 (with a test case).
- **Verified** (2026-11-18):
  - `cargo test --workspace`: 166 passed; new: 6 tests in `window_size.rs`
    and a NUL case in the recent list test.
  - `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --
    -D warnings` are clean.
  - Smoke run of the release build with an isolated configuration
    directory, window size read through AT-SPI: a stored 1400 × 900 opens at
    1400 × 900; a stored 1 × 38 opens at 1200 × 800. Quitting through the
    application action (the same path as Ctrl+Q) stored 1400 × 900 and the
    corrected 1200 × 800.
- **Manually verified** by the author (2026-11-18):
  - AC1–AC3: the size is remembered, also maximized and after tiling.
