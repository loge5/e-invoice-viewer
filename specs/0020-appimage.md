---
id: 0020
title: Packaging - AppImage, own icon and release workflow
status: reviewed
created: 2026-11-18
depends_on: [0012, 0019]
---

## Context

The project is public on GitHub. To use the app today, one has to install
Rust and the GTK development packages and build it. People who only want to
look at an invoice need a single download that runs. The author decided on
2026-11-18 to offer an AppImage, built in an Ubuntu 24.04 container, also by a
GitHub Actions workflow, and to give the app its own icon (see
[ADR 0004](../docs/adr/0004-appimage-built-in-container.md)).

## Requirement

- **One file** `E-Invoice_Viewer-<version>-x86_64.AppImage` runs on current
  x86_64 Linux desktops (glibc 2.39 or newer: Ubuntu 24.04, Debian 13,
  Fedora 40, current Arch, openSUSE Tumbleweed) after making it executable,
  without installing anything else. `<version>` is the version in
  `Cargo.toml`.
  - Like every AppImage, it uses what every desktop system has: the C
    library, the graphics drivers (OpenGL/EGL/GLES/Vulkan), fontconfig,
    FreeType and the fonts, the shared MIME database, the display server and
    D-Bus. Everything else (GTK,
    libadwaita, Poppler, GtkSourceView with its XML highlighting, GLib,
    Cairo, Pango, …) is inside the AppImage.
- **Same app**: the AppImage behaves like the installed build: PDF and XML,
  fields, XML view with highlighting, PDF pages, validation, attachments,
  the file dialogs for opening and saving, recent files, window size, both
  languages and the language switch. It opens no network connection, and it
  writes no file that the installed build does not write. It uses the same
  settings in `~/.config/e-invoice-viewer`.
- **Build** with one command, `scripts/appimage.sh`, on a machine with Docker.
  - It builds in an `ubuntu:24.04` container, pinned by digest, with the
    Ubuntu packages taken from a fixed date of the Ubuntu snapshot archive.
  - The Rust toolchain in the container is the version of `rust-version` in
    `Cargo.toml`; crates come from `Cargo.lock` (`--locked`).
  - Every tool it downloads (rustup, linuxdeploy, linuxdeploy-plugin-gtk) is
    fixed in version and SHA-256; a wrong checksum aborts the build without
    an AppImage.
  - It needs no administrator rights on the host beyond using Docker, and
    files it creates in the checkout belong to the user who ran it.
  - Running it again from the same commit gives an AppImage with the same
    content.
- **Release workflow**: a GitHub Actions workflow runs the same script.
  - For a pushed tag `v<version>` it creates a **draft** GitHub release for
    that tag with the AppImage attached; the author publishes it.
  - Started by hand, it keeps the AppImage as a workflow artefact.
  - The tag must match the version in `Cargo.toml`, otherwise the workflow
    fails before building.
  - It uses only GitHub's own actions and the preinstalled `gh` command, each
    action pinned by commit hash, and asks only for the permissions it needs
    (`contents: write` only for the release step).
- **Own icon**: the app has an icon of its own (scalable SVG, readable at
  16 px, in the style of GNOME app icons), named after the app id
  `de.loges.EInvoiceViewer`. The AppImage, the installed build and the
  windows of both use it. Where the desktop shows icons from desktop entries
  (app grid, window list on Wayland), the AppImage's icon appears once it is
  integrated (Gear Lever, AppImageLauncher).
- **Desktop metadata** exists once in the repository, under `data/`: the
  desktop entry and an AppStream metainfo file (name, summary, description in
  English and German, licence, screenshot, homepage, release 0.1.0). The
  AppImage and `scripts/install.sh` use them; the install script only fills
  in the path of the binary.
- **Bundled libraries are named**: the AppImage contains a list of the Ubuntu
  packages it bundles, with their versions, licences and where their source
  is available; the same list is attached to the release.
- **README**: a section "Download" explains how to run the AppImage (including
  `--appimage-extract-and-run` without FUSE), and the third-party notes name
  the bundled libraries.

## Acceptance criteria

- [x] Given a checkout with Docker available, when `scripts/appimage.sh` runs,
      then `target/appimage/E-Invoice_Viewer-0.1.0-x86_64.AppImage` exists, is
      executable, belongs to the user and is at most 80 MB.
  - Given a wrong SHA-256 for a downloaded tool, the script stops with a
    message naming the tool, exit code not 0, and no AppImage.
  - Two runs from the same commit give AppImages whose extracted files are
    identical (names and contents).
- [x] Given clean containers of `ubuntu:24.04`, `debian:13`, `fedora:43` and
      `archlinux`, each with only Mesa (OpenGL/EGL/GLES), fontconfig, one
      font, the shared MIME database, a virtual display and D-Bus installed, when the AppImage is started with
      `--appimage-extract-and-run` and CII sample A (`sample-a.xml`) and the
      Factur-X PDF of sample A are opened, then the app shows them, and
      `ldd` reports no missing library for any library in the AppImage.
- [x] Given the AppImage on the author's desktop, when sample A
      (`target/manual-0013/a-mit-lieferschein.pdf`) is opened through the file
      dialog, then it shows the same as the installed build: `No findings` /
      `Keine Befunde`, the fields, the PDF page, the attachment
      `lieferschein.pdf`, and the XML view with coloured elements and
      attributes (manual check).
  - Saving the attachment and "Save XML as…" work through the file dialog.
  - The language switch works, and the choice is shared with the installed
    build (same settings directory).
  - With the desktop in dark style, the app is dark.
- [x] Given the AppImage opening sample A and validating it, then no
      `AF_INET` or `AF_INET6` socket is opened (`strace -f -e trace=socket`),
      and the files it writes (`strace -f -e trace=openat,creat,rename`) are
      the same as those of the installed build doing the same, apart from the
      AppImage's own mount point.
- [x] Given `data/de.loges.EInvoiceViewer.desktop`, then
      `desktop-file-validate` reports no error; given
      `data/de.loges.EInvoiceViewer.metainfo.xml`, then
      `appstreamcli validate --no-net` reports no error.
- [x] Given `scripts/install.sh`, then the installed desktop entry is the one
      from `data/` with the binary's path, has `Icon=de.loges.EInvoiceViewer`,
      the icon is installed under
      `~/.local/share/icons/hicolor/scalable/apps/`, and the app grid and the
      window list show the new icon (manual check); `--uninstall` removes the
      icon as well.
- [x] The icon is recognisable at 16, 32 and 128 px, in light and dark style
      (manual check by the author).
- [x] Given the AppImage, then it contains the list of bundled packages with
      version, licence and source location, and every bundled library belongs
      to a package on the list.
- [ ] Given the workflow file, then a check of its logic shows: it runs
      `scripts/appimage.sh`; started by hand, it uploads the AppImage as an
      artefact; for a tag `v0.1.0`, it creates a draft release with the
      AppImage and the package list; a tag `v9.9.9` that does not match
      `Cargo.toml` fails before building; all actions are pinned by commit
      hash. The run on GitHub is a manual check by the author, who pushes the
      tag.
- [x] The app starts from the AppImage within 2 s on the reference machine,
      like the installed build (0007).

## Out of scope

- Other architectures (aarch64).
- Flatpak / Flathub, distribution packages (AUR, .deb, .rpm).
- Automatic updates (zsync update information) and signing the AppImage.
- Integrating the AppImage into the application menu or "Open with" by the
  app itself; tools such as Gear Lever or AppImageLauncher do that.
- Running tests in the workflow (a separate CI is a possible later spec).
- Pushing tags or publishing releases; the author does that.

## Notes

- The bundled libraries come unmodified from Ubuntu 24.04; their licences
  (LGPL, GPL for Poppler) allow redistribution, provided the source is
  available, which the package list points to.
- The icon replaces `x-office-document` everywhere, also for the installed
  build (0012).
- Review on 2026-11-18 (by Claude, accepted by the author), changes to the
  first draft:
  - The test containers get the libraries every desktop has (Mesa,
    fontconfig, a font): AppImage tools deliberately do not bundle them, so
    a container "without further packages" could never pass.
  - "Writes nothing except the settings" was impossible: GTK, Mesa and
    fontconfig write caches. The criterion is now "nothing the installed
    build does not write".
  - Same content on a second run needs Ubuntu packages from a fixed snapshot
    date, not only a pinned image.
  - Ubuntu 24.04's Rust is older than the required 1.92, so the toolchain is
    installed with rustup, pinned.
  - The release is a draft; the workflow uses only pinned GitHub actions and
    `gh`.
  - The icon in the window list depends on a desktop entry, which an AppImage
    has only once integrated.
  - A list of bundled packages for the licence obligations.
  - File dialogs added to the manual check; `fedora:41` (end of life)
    replaced by `fedora:43`.
- Corrections during implementation (2026-11-18):
  - The base set of the system (requirement and AC2) also names GLES and
    the shared MIME database. Without GLES, GTK aborts at start; without the
    MIME database, GdkPixbuf cannot tell an SVG file. Both belong to every
    desktop, and Mesa ships GLES; the clean containers lacked them.
  - linuxdeploy leaves out libraries it expects on every system; HarfBuzz,
    FriBidi, com_err and libgpg-error were missing in clean containers and
    are now bundled explicitly.
  - The symbolic icons the app uses come with the AppImage (from Ubuntu's
    Adwaita icon theme, listed with the packages), because a system without
    the Adwaita theme showed placeholders instead of the zoom buttons.
  - The start hook of linuxdeploy-plugin-gtk sets `GDK_BACKEND=x11` and a
    fixed `GTK_THEME`; the build removes both, so the app runs natively on
    Wayland and libadwaita's dark style follows the system.
  - After `scripts/install.sh`, GNOME Shell kept the placeholder icon: icon
    themes are rescanned only when the theme directory itself changes. The
    install script now touches `~/.local/share/icons/hicolor` after
    installing and uninstalling; the install test checks it.
- Open: the homepage and screenshot address in the metainfo need the GitHub
  address of the repository (placeholder `OWNER`).

## Progress

- **Done**:
  - `data/`: desktop entry, AppStream metainfo (English and German) and the
    app icon `de.loges.EInvoiceViewer.svg`; the app sets it as its window
    icon.
  - `scripts/install.sh` installs the desktop entry from `data/` (filling in
    the binary's path) and the icon, and removes both on `--uninstall`;
    `SOURCE_BINARY` installs a given binary (for the test).
  - `scripts/appimage.sh` builds the image `scripts/appimage/Dockerfile`
    (`ubuntu:24.04` by digest, packages from snapshot `20261117T000000Z`,
    Rust 1.92 via rustup, linuxdeploy `1-alpha-20251107-1` and the GTK plugin
    at commit `7a3fbc3`, each with SHA-256 through `scripts/appimage/fetch.sh`)
    and runs `scripts/appimage/build.sh` in it as the calling user.
  - The build writes `bundled-packages.txt` (package, version, licence,
    Launchpad source address) and the packages' copyright files into the
    AppImage, and fails if a bundled library belongs to no package.
  - `.github/workflows/appimage.yml`: builds on tags `v*` and by hand,
    checks the tag with `scripts/appimage/check-tag.sh` first, uploads the
    artefact, and for a tag creates a draft release with `gh`; actions pinned
    by commit hash, `contents: write` only in the release job.
  - README: section "Download", install from source, licence notes for the
    bundled libraries.
- **Verified** (2026-11-18):
  - `cargo test --workspace`: all green; new: 8 tests in
    `crates/app/tests/packaging.rs` (T1–T6: AC1, AC5, AC6, AC9).
  - P1: `scripts/appimage.sh` gives
    `target/appimage/E-Invoice_Viewer-0.1.0-x86_64.AppImage`, 41 MB, owned
    by the user.
  - P2: two clean builds from the same sources give byte-identical AppImages
    (same SHA-256), so also identical extracted files.
  - P3: in clean `ubuntu:24.04`, `debian:13`, `fedora:43` and `archlinux`
    containers with the base set, `ldd` finds every library of the AppImage,
    and the app shows `sample-a.xml` and the Factur-X PDF of sample A
    (screenshots read: fields, PDF page, zoom buttons).
  - P4 (Arch container, strace, same sample, fresh home): neither the
    AppImage nor the installed build opens an `AF_INET`/`AF_INET6` socket;
    the AppImage writes only files the installed build writes as well (its
    settings, Mesa's shader cache). Seen on the way: if the X display is
    gone, libxcb probes `localhost:6099` – both builds alike, local only.
  - P5: the build checks that every bundled library belongs to a listed
    package (107 lines in the list).
  - P6: in the container (software rendering), the opened file is recorded
    0.8 s after the start. Not measured on the author's desktop, because the
    author's viewer was running.
  - `desktop-file-validate` and `appstreamcli validate --no-net` pass (T1,
    T2); `cargo fmt`, `clippy -D warnings` and the architecture check are
    clean.
- **Manually verified** by the author (2026-11-18): after
  `scripts/install.sh`, the app grid and the dash show the new icon (AC6);
  the AppImage on the desktop with file dialogs, language switch, dark style
  and Wayland (AC3); the icon at 16, 32 and 128 px, light and dark (AC7);
  the start time (AC10).
- **Next**: AC9 – the workflow run on GitHub, once the author pushes a tag
  `v0.1.0`; the workflow logic itself is checked by the tests (T6).
