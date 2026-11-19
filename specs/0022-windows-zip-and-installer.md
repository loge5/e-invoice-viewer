---
id: 0022
title: Packaging - Windows as portable ZIP and installer
status: implemented
created: 2026-11-19
depends_on: [0012, 0013, 0019, 0020]
---

## Context

E-invoices reach many people who work on Windows, also at the author's
company. The author decided on 2026-11-19 to offer the app for Windows as a
portable ZIP and as an installer, built only on GitHub Actions and tested in
a Windows VM (see [ADR 0005](../docs/adr/0005-windows-build-msys2-github-actions.md)).

## Requirement

- **Two files per release** for 64-bit Windows 10 (version 1809 or later)
  and Windows 11:
  - `E-Invoice_Viewer-<version>-windows-x86_64.zip`: unpack anywhere and
    start `bin\e-invoice-viewer.exe`; nothing is installed.
  - `E-Invoice_Viewer-<version>-windows-x86_64-setup.exe`: installs for the
    current user without administrator rights.
  - `<version>` is the version in `Cargo.toml`.
- **Same app**: PDF and XML, fields, XML view with highlighting, PDF pages,
  validation, attachments, recent files, window size, both languages and the
  language switch, as on Linux. It opens no network connection and writes
  nothing except its settings in `%LOCALAPPDATA%\e-invoice-viewer` and the
  files the user saves.
- **A normal Windows program**: no console window opens; the executable,
  the window and the taskbar show the app's icon, and the symbolic icons
  (zoom buttons, menu, status) are shown. Without a choice in the app (0019),
  the language follows Windows as GLib reports it, which on Windows is the
  user's locale (Settings → Time & language → Region), not necessarily the
  display language. Windows' dark mode is followed if the bundled libadwaita
  supports it on Windows; otherwise the app is light.
- **Installer**:
  - installs into `%LOCALAPPDATA%\Programs\E-Invoice Viewer`;
  - adds a Start menu entry `E-Invoice Viewer`;
  - registers the app for `.pdf` and `.xml` under "Open with", without
    becoming the default application for either;
  - offers to start the app at the end;
  - installing a newer version over an older one replaces it;
  - the uninstaller (Settings → Apps) removes program files, Start menu entry
    and file registration, and keeps the user's settings (like 0012).
- **Opening files**: double-click via "Open with", drag and drop onto the
  window, the file dialog and the recent files work with Windows paths,
  including files on mapped network drives and UNC paths
  (`\\server\share\…`), which are local files in the sense of 0012. Web
  addresses are still refused.
- **File names for saving** (0013, on every system, not only Windows): a
  proposed name never contains characters Windows does not allow in file
  names (`< > : " / \ | ? *`), which are replaced by `_`; it does not end
  with a dot or space; and Windows device names (`CON`, `PRN`, `AUX`, `NUL`,
  `COM1`–`COM9`, `LPT1`–`LPT9`, also with an extension) get a leading `_`.
  If nothing is left, the name is `Anhang-<position>` as in 0013.
- **Release workflow**: the AppImage workflow (0020) becomes one release
  workflow. For a tag `v<version>` it builds the AppImage and both Windows
  files and creates one draft release with all of them and the package lists
  of both platforms; started by hand, it keeps them as artefacts. The tests
  of the workspace run on Windows before packaging; a failing test stops the
  Windows build. A release is all or nothing: if one platform fails, no
  release is created.
- **Bundled libraries are named**: the ZIP and the installation contain the
  licence texts and a list of the bundled MSYS2 packages with version,
  licence and source address; the list is attached to the release.
- **README**: the "Download" section explains the Windows files, including
  the SmartScreen warning for an unsigned program and how to continue.

## Acceptance criteria

- [x] Given the workflow started by hand, then it produces
      `E-Invoice_Viewer-0.1.0-windows-x86_64.zip` and
      `E-Invoice_Viewer-0.1.0-windows-x86_64-setup.exe` as artefacts, together
      with the AppImage, and the log shows `cargo test --workspace` passing on
      Windows.
- [x] Given the ZIP unpacked in the VM into a folder with a space in its name
      (e.g. `C:\Users\<user>\Desktop\EIV test`), when
      `bin\e-invoice-viewer.exe` is started by double-click, then the start
      view appears without a console window, and the window and taskbar show
      the app icon (manual check).
  - Opening `a-mit-lieferschein.pdf` (sample A as Factur-X PDF) shows
    `Keine Befunde` / `No findings`, the fields, the PDF page and the
    attachment `lieferschein.pdf`; the XML view shows coloured elements.
  - Saving the attachment and "Save XML as…" use the Windows file dialog and
    write the file.
  - With the Windows region format German (Germany), the app starts in
    German; with English (United States), in English; the language switch
    works and is kept after a restart.
  - The zoom buttons, the main menu button and the status icon show their
    symbols, not placeholders.
  - With Windows set to dark mode, the app is dark or light; the manual check
    records which (see requirement).
  - The file `%LOCALAPPDATA%\e-invoice-viewer\recent` exists after opening a
    file; nothing is written into the unpacked folder.
- [x] Given the installer run as a normal user in the VM, then no
      administrator prompt appears, the app is in the Start menu, and
      "Open with" for a `.pdf` and an `.xml` file lists `E-Invoice Viewer`;
      the default app for `.pdf` is the same as before (manual check).
  - Choosing it there opens the file.
  - Installing the same version again, or a newer one, keeps a single entry
    in Settings → Apps.
  - After uninstalling, the program folder, the Start menu entry and the
    "Open with" entry are gone, and `%LOCALAPPDATA%\e-invoice-viewer` is
    still there.
- [x] Given `sample-c.xml` on a mapped network drive or a UNC path in the VM
      (e.g. a shared folder of the VM host), then it opens like a local file;
      given `e-invoice-viewer.exe https://example.com/a.pdf`, the message of
      0012 for non-local addresses is shown (manual check).
- [x] Proposed file names for saving (automated, on Linux and Windows):

      | attachment name | proposed name |
      |---|---|
      | `Lieferschein: 11-2026?.pdf` | `Lieferschein_ 11-2026_.pdf` |
      | `CON.pdf` | `_CON.pdf` |
      | `nul` | `_nul` |
      | `bericht.pdf.` | `bericht.pdf` |
      | `a<b>c*d"e.xml` | `a_b_c_d_e.xml` |
      | `lieferschein.pdf` | `lieferschein.pdf` |

      A `|` is replaced by `_` as well. `/` and `\` keep their meaning from
      0013 (only the last path part is used), so `../x/CON.pdf` proposes
      `_CON.pdf`. Names that are proposed unchanged today stay unchanged when
      they contain none of these characters.
- [x] Given the ZIP, then `bin\` contains `e-invoice-viewer.exe`, and the
      package list names every DLL in it with its MSYS2 package; a DLL that
      belongs to no package fails the build.
- [ ] Given the workflow file, then a check of its logic shows: the tag is
      checked against `Cargo.toml` before building; the Windows job runs the
      tests before packaging; the release job needs the AppImage and the
      Windows job and creates one draft release with the AppImage, the ZIP,
      the installer and both package lists; all actions are pinned by commit
      hash; only the release job may write. The run on GitHub is a manual
      check by the author.
- [x] On Linux nothing changes: the AppImage build, `scripts/install.sh` and
      all tests behave as before.
- [x] The app starts from the ZIP and from the installation within 3 s in
      the VM (manual check; the limit of 0007 is relaxed for a VM).

## Out of scope

- Code signing (the SmartScreen warning remains), Microsoft Store, MSI
  packages, automatic updates.
- 32-bit Windows and Windows on ARM.
- A local Windows build; the build runs only on GitHub Actions.
- Opening a file from Explorer in the already running app (see ADR 0005,
  single instance); each may open its own process.
- macOS.

## Notes

- Paths in tests that are written for Unix (e.g. `/tmp/x/…`) are adapted or
  limited to Unix where Windows paths behave differently; tests of Linux
  packaging run only on Linux.
- The app id `de.loges.EInvoiceViewer` and the icon stay the same; the
  Windows icon (`.ico`) is generated from the SVG icon.
- The time limit for the start is 3 s in a VM, because a VM without graphics
  acceleration renders in software.
- Test files for the VM: `crates/core/tests/fixtures/cii/sample-a.xml`,
  `…/cii/sample-c.xml` and `target/manual-0013/a-mit-lieferschein.pdf`
  (generated locally, see 0013); the author copies them into the VM.
- Review on 2026-11-19 (by Claude, accepted by the author), changes to the
  first draft:
  - On Windows, GLib takes the language from the user's locale (region
    format), not from the display language; the requirement and AC2 say so.
  - On Windows, GIO detects file types by extension and registry, not with
    the shared MIME database; the symbolic icons are therefore checked
    explicitly (on Linux, missing MIME detection hid all SVG icons, 0020).
  - Dark mode on Windows depends on libadwaita's Windows support; it is
    recorded in the manual check instead of promised.
  - The file name table no longer uses `/` inside a name (0013 already cuts
    the directory part), and a name that becomes empty falls back to
    `Anhang-<n>`.
  - A release is all or nothing across both platforms.

## Progress

- **Done**:
  - Core: proposed file names are valid on Windows (`attachments.rs`).
  - App: Windows GUI subsystem; `crates/app/build.rs` embeds the icon and
    version information with `windres` on Windows; the `.ico` comes from
    `scripts/make-ico.sh`. The recent files label uses the system's path
    separator after `~`.
  - `scripts/windows/bundle.sh`: the folder (exe, DLLs found with `ntldd`,
    GdkPixbuf loaders and cache, GSettings schemas, GtkSourceView data, the
    symbolic icons used, the app icon, licences, package list) and the ZIP;
    fails for a DLL without MSYS2 package.
  - `scripts/windows/installer.iss`: Inno Setup, per user, Start menu,
    "Open with" via `OpenWithProgids`, everything removed on uninstall.
  - `.github/workflows/release.yml` replaces `appimage.yml`: jobs
    `appimage`, `windows` (MSYS2 UCRT64, tests, build, bundle, installer)
    and `release` (one draft release with all files).
  - `.gitattributes`: LF in every checkout (the Windows runner converted
    fixtures to CRLF otherwise); third-party rule files byte-exact.
  - Tests with Unix paths run only on Unix or got Windows variants; the
    catalogue test accepts CRLF from xgettext on Windows.
  - README: Windows section with the SmartScreen note.
- **Verified** (2026-11-19):
  - Linux: `cargo test --workspace` 222 passed; new: T1
    (`proposed_names_are_valid_on_windows`), T2 (`windows_icon_has_the_usual_sizes`),
    T3 (`installer_is_per_user_and_never_default`), T4
    (`release_workflow_builds_both_platforms`); fmt, clippy and the
    architecture check clean; the AppImage builds locally as before (AC8).
  - GitHub Actions, run 44472078826 on the trial branch `windows`: the
    Windows job runs `cargo test --workspace` successfully and produces the
    artefact `windows` (ZIP, installer, package list, 68 MB) next to the
    AppImage (AC1, P2, P3). Four earlier runs failed and led to the fixes
    above (MSYS2 setup, UCRT64, absolute test paths, CRLF).
- **Manually verified** by the author in a Windows VM (2026-11-19): ZIP and
  installer as described in AC2, AC3, AC4 and AC9, all as expected (whether
  the app follows Windows' dark mode was not reported separately).
- The author decided on `implemented` on 2026-11-19, before the first
  release run with a tag.
- **Next**: AC7 – the workflow run for a tag on GitHub; the workflow logic is
  checked by T4, the Windows job ran successfully on the trial branch.

