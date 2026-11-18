# 0004 — Distribution as AppImage, built in an Ubuntu 24.04 container

- Status: accepted
- Date: 2026-11-18
- Specs: [0020](../../specs/0020-appimage.md)

## Context

The project is public. Until now it can only be built from source and
installed with `scripts/install.sh`, which needs Rust, the GTK development
packages and GNU gettext. Users who only want to view an invoice need a
download that runs without building anything.

The author decided on 2026-11-18:

- the app is offered as an **AppImage**;
- it is built in an **Ubuntu 24.04 container**, so that it runs on current
  distributions, not only on ones as new as the build machine (Arch, glibc
  2.44);
- the build may download **linuxdeploy** and **linuxdeploy-plugin-gtk**, pinned
  to a version and a SHA-256 checksum;
- a **GitHub Actions** workflow builds the AppImage and attaches it to a
  release;
- the app gets its **own icon**.

The app needs GTK ≥ 4.12, libadwaita ≥ 1.5, Poppler ≥ 0.82 (GLib) and
GtkSourceView 5. Ubuntu 24.04 ships GTK 4.14, libadwaita 1.5, Poppler 24.02 and
GtkSourceView 5.12. Validation rules and translations are already embedded in
the binary (ADR 0002, ADR 0003), so the binary has no data files of its own.

## Decision

1. **AppImage for x86_64**, named `E-Invoice_Viewer-<version>-x86_64.AppImage`,
   with the version from `Cargo.toml`.
2. **Built in a container** from the image `ubuntu:24.04` (pinned by digest),
   with packages from a fixed date of the Ubuntu snapshot archive, so that a
   rebuild bundles the same libraries. The oldest supported system is
   therefore glibc 2.39 (Ubuntu 24.04, Debian 13, Fedora 40 and newer). Ubuntu's
   own Rust is too old (the project needs 1.92), so the toolchain is
   installed with rustup in the version of `rust-version`.
3. **Bundling with linuxdeploy and linuxdeploy-plugin-gtk.** They copy the
   shared libraries the binary needs, GTK's modules, the GdkPixbuf loaders and
   GSettings schemas, and add a start hook that points GTK to the bundled
   files. Data that the plugin does not know (GtkSourceView's language and
   style definitions for the XML view, the symbolic icons the app uses) is
   copied by our script, and libraries linuxdeploy leaves out but a minimal
   desktop may lack (HarfBuzz, FriBidi) are bundled explicitly. The plugin's
   hook forces X11 and a fixed theme; the build removes both settings. Both tools are
   downloaded at build time only, in a fixed version, and checked against a
   SHA-256 written in the script; a mismatch aborts the build.
4. **One script**, `scripts/appimage.sh`, runs the whole build on the host
   (it starts the container) and is used unchanged by the GitHub Actions
   workflow, so a local build and a release build are the same.
5. **Desktop metadata** lives in the repository under `data/`: the desktop
   entry, the icon (SVG) and an AppStream metainfo file. `scripts/install.sh`
   uses the same desktop entry and icon.

## Alternatives

| Option | Why not |
|---|---|
| Build on the host (Arch) | Runs only on distributions with glibc ≥ 2.44 |
| Copy libraries by hand, package with `appimagetool` only | Fewer foreign tools, but GTK's modules, loaders, schemas and environment would have to be maintained in our own script |
| Flatpak / Flathub | The usual way for GNOME apps, sandboxed, with updates; but needs a manifest, Poppler built as a module and Flathub review. Not chosen by the author for now; the metainfo and icon made here are reusable for it |

## Consequences

- **New build-time tools** (not dependencies of the app): Docker on the build
  machine; rustup, linuxdeploy and linuxdeploy-plugin-gtk (MIT) downloaded
  during the build. Nothing changes at run time; the app still opens no network
  connection.
- The AppImage **contains LGPL libraries** (GTK, libadwaita, GLib,
  GtkSourceView …) and Poppler (GPL) in unmodified form from Ubuntu 24.04. A
  package list with versions, licences and source locations is part of the
  AppImage and of each release; the README names them next to the existing
  third-party notes.
- AppImages deliberately use the host's graphics drivers (including GLES),
  fontconfig, FreeType, fonts and MIME database; these are not bundled.
- The bundled GTK uses the bundled Adwaita style; the system's GTK theme is
  not applied. Dark mode still follows the system through libadwaita.
- Running an AppImage needs FUSE on the host, or the option
  `--appimage-extract-and-run`. This is the usual AppImage behaviour.
- The AppImage does not integrate itself into the application menu or
  "Open with"; tools such as Gear Lever or AppImageLauncher do that.
