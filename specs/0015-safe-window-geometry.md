---
id: 0015
title: Remember the window size safely
status: draft
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

When a window is closed, the app remembers its size and whether it was
maximized. On the next start, a remembered size is applied only if it is
plausible on the current screens; otherwise the default is used. The window
position is not stored (the desktop places the window).

## Acceptance criteria

- [ ] Given the user resizes the window to 1400 × 900 and closes it, then the
      next start opens the window at 1400 × 900.
- [ ] Given the window was maximized when closed, then the next start opens it
      maximized, and un-maximizing returns to the last remembered normal size.
- [ ] Given a stored size of 1 × 38, or a negative size, or a size larger than
      the largest current monitor (e.g. 9000 × 9000 with a 3840 × 2160
      monitor), then the next start uses the default size 1200 × 800 (or the
      monitor size, whichever is smaller) and stores the corrected value on
      close.
- [ ] Given the stored settings file is empty or not parseable, then the app
      starts with the default size, shows no error dialog, and logs one
      warning.
- [ ] Given the app crashes or is killed while being resized, then no
      half-written settings file remains (the previous valid value is kept).
- [ ] Settings are stored only in the app's configuration directory and
      contain no file names of opened invoices other than the recent list from
      0012.

## Out of scope

- Remembering the window position, divider position or zoom level.

## Notes

- The plausibility rule and settings parsing are unit-tested; criteria 1 and 2
  are verified manually.
