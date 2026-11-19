# 0005 — Windows: MSYS2 build on GitHub Actions, portable ZIP and installer

- Status: accepted
- Date: 2026-11-19
- Specs: [0022](../../specs/0022-windows-zip-and-installer.md)

## Context

The app is Linux-only so far: the AppImage (ADR 0004) and `scripts/install.sh`.
The author decided on 2026-11-19 that it should also run on Windows:

- offered both as a **portable ZIP** (unpack and start) and as an
  **installer**;
- built **only on GitHub Actions**; there is no local Windows build;
- tested by the author in a **Windows VM**.

The libraries the app needs — GTK ≥ 4.12, libadwaita ≥ 1.5, Poppler (GLib),
GtkSourceView 5 — are available for Windows as prebuilt MinGW packages of
MSYS2. Validation rules and translations are embedded in the binary
(ADR 0002, ADR 0003). The app code uses no Unix-only interfaces: settings live
in `glib::user_config_dir()` (on Windows `%LOCALAPPDATA%`), file dialogs are
`GtkFileDialog` (on Windows the native dialog).

## Decision

1. **Toolchain**: MSYS2 on the `windows-2025` GitHub runner
   (`msys2/setup-msys2`, pinned by commit hash), environment **MINGW64**, with
   the packages `gtk4`, `libadwaita`, `poppler`, `gtksourceview5`, `gettext`
   and `pkgconf`. Rust comes from rustup in the version of `rust-version`,
   target `x86_64-pc-windows-gnu`, which matches the MINGW64 C runtime
   (msvcrt). MSYS2 recommends UCRT64 for new projects; if MINGW64 packages
   are retired, the build moves to UCRT64 with MSYS2's own Rust package, which
   is built for the UCRT runtime.
2. **A Windows app, not a console program**: the binary is built for the
   Windows GUI subsystem (no console window), with the app icon embedded as a
   Windows resource. `build.rs` compiles the resource with `windres` from
   MSYS2 when the target is Windows; no new crate. The `.ico` file is
   generated once from the SVG icon by a script and committed.
3. **One folder layout** for both deliveries: `bin/` with the executable and
   every DLL it loads (found with `ntldd`), `lib/` with the GdkPixbuf loaders
   and their cache, `share/` with the compiled GSettings schemas, the
   GtkSourceView data, the symbolic icons the app uses and the app icon,
   plus the licence texts and a list of the bundled MSYS2 packages with
   their versions and source addresses. GTK finds these relative to the
   executable; no environment variables or registry entries are needed. The
   GSettings schemas are required: GTK's file dialog aborts without them.
4. **Portable ZIP**: the folder, zipped. It writes nothing besides the app's
   settings, like every other build.
5. **Installer: Inno Setup**, per user, without administrator rights: the
   folder into `%LOCALAPPDATA%\Programs\E-Invoice Viewer`, a Start menu entry,
   registration under "Open with" for `.pdf` and `.xml` (never as default
   application, like 0012), and an uninstaller that removes all of it except
   the user's settings. Inno Setup is installed on the runner in a fixed
   version.
6. **One release workflow**: the existing AppImage workflow becomes
   `release.yml` with jobs for the AppImage, the Windows build and the draft
   release, which gets all files of both platforms.

## Alternatives

| Option | Why not |
|---|---|
| MSVC with gvsbuild | Native Microsoft toolchain, but GTK, libadwaita, Poppler and GtkSourceView must be built from source on every run (or cached), which takes long and breaks more often |
| Cross-compiling from Linux (MinGW) | Would allow a local build, but MinGW packages of libadwaita, Poppler and GtkSourceView are not reliably available in Linux distributions; the author wants only GitHub builds anyway |
| MSIX / Microsoft Store | Needs signing and a Store account |
| WiX (MSI) instead of Inno Setup | MSI is better for managed company deployment, but WiX is considerably more complex for a per-user install with file associations; can be added later |
| `embed-resource` crate for the icon | Works the same, but is a new dependency; `windres` is already part of MSYS2 |

## Consequences

- **New build-time tools** on the runner: MSYS2 with the packages above,
  Inno Setup. Nothing changes for Linux builds; no new crate.
- **Not reproducible to the byte**: MSYS2 is a rolling distribution; old
  package versions cannot be pinned as easily as the Ubuntu snapshot of the
  AppImage. The package list of each release records exactly what was
  bundled.
- **Not signed**: Windows SmartScreen warns on first start of an unsigned
  installer or executable ("Unknown publisher"). The README explains this.
  Code signing is a possible later step and needs a certificate.
- **Single instance**: GLib's single-instance mechanism uses D-Bus, which
  Windows does not have; files opened from Explorer may open in a separate
  process each instead of a new window of the running one. Accepted.
- The bundled LGPL libraries and Poppler (GPL) are distributed like in the
  AppImage: unmodified, with licence texts and source addresses (MSYS2 keeps
  the source of every package version under `repo.msys2.org/mingw/sources`).
- A release needs both platforms to build; a broken Windows build blocks the
  AppImage release as well, and vice versa.
- **Tests**: `cargo test --workspace` runs on Windows in the workflow; tests
  of Linux packaging (desktop entry, AppStream, `install.sh`, AppImage build
  scripts) run only on Linux.
