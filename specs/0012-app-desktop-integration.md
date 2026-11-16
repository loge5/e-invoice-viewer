---
id: 0012
title: App - drag and drop, "Open with" and recent files
status: draft
created: 2026-11-16
depends_on: [0007]
---

## Context

Invoices usually arrive as e-mail attachments or downloads. Users want to open
them the way they open any file in GNOME: drag them onto the window, choose
the app in the file manager's "Open with" menu, or pick a recently opened
file, without the app reopening something on its own.

## Requirement

The app accepts files by drag and drop, registers itself as an application for
PDF and XML files so it appears in "Open with" (without becoming the default
application), and offers a list of recently opened files. Each opened file
from outside (file manager, second command line call) opens in its own window.

## Acceptance criteria

- [ ] Given the start view or an opened invoice, when the user drops
      `RE-2026-0042.pdf` onto the window, then that file is opened in this
      window (replacing a previously shown invoice). Dropping two files at
      once opens the first one and shows the note `Nur eine Datei pro Fenster
      – die erste wurde geöffnet`.
- [ ] Given the app is installed, then the desktop entry declares the MIME
      types `application/pdf`, `application/xml` and `text/xml`, so the app
      is listed under "Öffnen mit" for these files in the file manager (manual
      check in Nautilus), and installing it does not change the default
      application for PDF files.
- [ ] Given a window is already open, when the user opens another invoice via
      the file manager or runs `e-invoice-viewer other.pdf`, then a second
      window opens with `other.pdf`, and the first window stays unchanged.
- [ ] Given the user opened `a.pdf`, then `b.xml`, then `a.pdf` again, then the
      "Zuletzt geöffnet" menu lists `a.pdf`, `b.xml` in this order (most
      recent first, no duplicates, at most 10 entries).
- [ ] Given a recent entry whose file no longer exists, when the user selects
      it, then the message `Die Datei <name> wurde nicht gefunden` is shown and
      the entry is removed from the list.
- [ ] The recent files list is stored in the user's configuration directory of
      the app; nothing is written next to the invoice files, and starting the
      app does not open any of the recent files automatically.

## Out of scope

- Becoming the default application for PDF files.
- Tabs or multiple invoices per window.
- Integration with mail clients.

## Notes

- Mostly GUI/desktop-bound: verified manually; ordering and size limit of the
  recent list are covered by unit tests.
