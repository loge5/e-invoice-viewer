---
id: 0012
title: App - drag and drop, "Open with" and recent files
status: reviewed
created: 2026-11-16
depends_on: [0007]
---

## Context

Invoices usually arrive as e-mail attachments or downloads. Users want to open
them the way they open any file in GNOME: drag them onto the window, choose
the app in the file manager's "Open with" menu, or pick a recently opened
file, without the app reopening something on its own.

## Requirement

The app accepts files by drag and drop and offers a list of recently opened
files. It can be installed for the current user, without administrator
rights. Once installed, it appears as an application for PDF and XML files in
the file manager's "Öffnen mit" menu, but it does not become the default
application. It can be uninstalled the same way.

**Windows.** Each file opened from outside (the file manager or a further
command line call) opens in its own window. An empty window showing the start
view is reused for such a file instead. Files opened inside a window (drop,
file dialog, recent files) replace what that window shows.

**Only local files.** The app reads only files the desktop presents as files.
This includes files on network shares the user has mounted in the file
manager (e.g. smb, sftp). Web addresses (`http://`, `https://`, `ftp://`) and
other addresses that would make the app fetch content from the network are
never loaded, whichever way they reach the app.

**Recent files.** The menu "Zuletzt geöffnet" in the header bar is available
in the start view and while an invoice is shown.
- Every successful opening counts: file dialog, drop, command line, "Öffnen
  mit" and the recent list itself. An opening that fails (unreadable, not an
  invoice) does not count.
- Entries are ordered most recent first. There are at most 10 entries and no
  duplicates; two entries are the same file if their full paths are equal.
- Each entry shows the file name and its folder, so two files with the same
  name in different folders can be told apart.
- Selecting an entry opens the file in the current window.
- All windows show the same list.
- The list belongs to the user, not to an invoice. It is kept in the app's
  configuration directory and survives restarts.
- Starting the app never opens a recent file by itself.
- The app does not add the invoices it opens to the desktop's shared list of
  recently used files (e.g. "Zuletzt verwendet" in the file manager).

## Acceptance criteria

Sample invoices are defined in [fixtures.md](../docs/fixtures.md).

- [ ] Given the start view or an opened invoice, when the user drops
      `RE-2026-0042.pdf` onto the window, then that file is opened in this
      window, replacing a previously shown invoice.
  - Dropping `RE-2026-0042.pdf` and `RE-2026-0043.pdf` at once opens
    `RE-2026-0042.pdf` and shows the note `Nur eine Datei pro Fenster – die
    erste wurde geöffnet`.
  - Dropping a file that is not an invoice (e.g. `foto.png`) or a folder
    shows the error message of 0007 for unreadable invoices.
- [ ] Given the user runs the repository's install step as a normal user,
      then:
  - the app is listed under "Öffnen mit" for `RE-2026-0042.pdf` and
    `RE-2026-0042.xml` in Nautilus (manual check);
  - choosing it there opens the file;
  - `xdg-mime query default` returns the same application for
    `application/pdf`, `application/xml` and `text/xml` as before the
    installation;
  - after the uninstall step the app is no longer listed, and its desktop
    entry and binary are gone.
- [ ] Given a window shows sample A, when the user opens `RE-2026-0043.pdf`
      via the file manager or runs `e-invoice-viewer RE-2026-0043.pdf`, then
      a second window opens with it, and the first window still shows sample
      A.
  - Selecting both sample files in Nautilus and choosing "Öffnen mit" opens
    two windows, one per file.
  - Given the only window shows the start view, opening a file from outside
    opens it in that window; no second window appears.
- [ ] Given `e-invoice-viewer https://example.com/RE-2026-0042.pdf`, or a
      dropped link `https://example.com/RE-2026-0042.pdf` from a browser,
      then no network connection is made, and the message `Die Adresse
      https://example.com/RE-2026-0042.pdf ist keine lokale Datei` is shown
      (start view, or a note while an invoice stays shown).
- [ ] Given the user opened `a.pdf`, then `b.xml`, then `a.pdf` again, then
      "Zuletzt geöffnet" lists `a.pdf`, then `b.xml`, each with its folder.
  - Given 10 entries, opening an eleventh file adds it on top and removes
    the oldest entry.
  - Given `/tmp/x/rechnung.pdf` and `/tmp/y/rechnung.pdf` were opened, then
    both are listed, and their folders tell them apart.
  - Given opening `kaputt.xml` failed, then it is not listed.
- [ ] Given two windows are open, when a file is opened in one of them, then
      the list in the other window shows it on top as well.
- [ ] Given the user selects a recent entry while a window shows sample B,
      then the selected file replaces sample B in that window.
- [ ] Given a recent entry `alt.pdf` whose file no longer exists, when the
      user selects it, then the window keeps showing what it showed, the note
      `Die Datei alt.pdf wurde nicht gefunden` is shown, and the entry is
      removed from the list.
- [ ] Given the stored list is empty, truncated or not readable, then the app
      starts normally with an empty list. It shows no error dialog and logs
      one warning.
  - If the app is killed while the list is being saved, the previous list
    is kept; no half-written list remains.
- [ ] Given recent entries exist, when the app is started without arguments,
      then the start view is shown and no file is opened.
  - The list is stored only in the app's configuration directory. Nothing is
    written next to the invoice files.
  - Opening sample A by drop, command line or the recent list leaves the
    desktop's shared list of recently used files unchanged.

## Out of scope

- Becoming the default application for PDF or XML files.
- An AUR package or Flatpak; an application icon of its own.
- Tabs or multiple invoices per window.
- Integration with mail clients.
- Pinning, renaming or manually removing recent entries; a "clear list"
  action.
- What the system file dialog records about the user's choice. It follows
  the desktop's settings, not the app's.

## Notes

- Mostly GUI/desktop-bound: verified manually with the samples. The rules of
  the recent list are covered by unit tests: ordering, duplicates by full
  path, the size limit, and unreadable stored lists.
- Today the app already loads `https://…` arguments through the desktop's
  virtual file system. The local-files criterion fixes this for the command
  line too.
- 0015 stores the window size in the same configuration directory.
- Review on 2026-11-17: the author decided installation for the current user
  via an install step in the repository, mounted network shares allowed, web
  addresses refused, reuse of an empty start window, list rules as above, no
  entries in the desktop's shared recent list, and that a missing recent file
  keeps the current invoice shown.
