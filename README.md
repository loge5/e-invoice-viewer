# E-Invoice Viewer

**A fast, strictly local desktop viewer for electronic invoices — ZUGFeRD,
Factur-X and XRechnung — for Linux and Windows.**

Open an invoice and see the PDF and the structured invoice data side by side,
with every field labelled by its EN 16931 business term. Validation findings
appear right next to the field they concern. Nothing leaves your machine.

![E-Invoice Viewer showing a ZUGFeRD invoice: the PDF on the left, the invoice fields grouped by section on the right](docs/screenshot.png)

<p align="center">
  <a href="https://github.com/loge5/e-invoice-viewer/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/loge5/e-invoice-viewer/actions/workflows/ci.yml/badge.svg"></a>
  <img alt="Licence: GPL-3.0-or-later" src="https://img.shields.io/badge/licence-GPL--3.0--or--later-blue">
  <img alt="Rust 1.92+" src="https://img.shields.io/badge/rust-1.92%2B-orange">
  <img alt="GTK4 + libadwaita" src="https://img.shields.io/badge/GTK4-libadwaita-4a86cf">
  <img alt="Platform: Linux | Windows" src="https://img.shields.io/badge/platform-Linux%20%7C%20Windows-lightgrey">
</p>

## Why

E-invoices are XML. The PDF is what people read, the XML is what counts, and
the two do not always agree. E-Invoice Viewer shows both at once, tells you
what the official rules say about the XML, and does so without Java, without
an online service and without uploading your business data anywhere.

## Features

- **PDF and fields side by side.** All pages of the PDF, zoomable, next to the
  invoice data grouped into sections: invoice and references, seller, buyer,
  delivery and invoicing period, line items, payment, allowances and charges,
  VAT breakdown, totals. Every value carries its business term
  number (`BT-1`, `BG-4`, …) and is formatted for humans (dates, amounts,
  IBANs, code lists).
- **Validation built in.** Checks against the XML schema (CII D16B, UBL 2.1),
  the official EN 16931 business rules and the XRechnung CIUS (`BR-DE-*`). Findings are shown at the affected field
  or section, with a summary of all errors, warnings and notes.
- **Raw XML view.** The invoice XML, readably formatted, with search. Fields
  and XML are linked both ways: *Show in XML* jumps from a field or finding to
  its element, and the line below the XML tells you which field the cursor is
  in.
- **Attachments.** Lists files embedded in the PDF and supporting documents in
  the invoice (BG-24), and saves them where you choose.
- **Plain XML invoices, too.** CII and UBL (invoice and credit note) files open
  without a PDF.
- **German and English.** The interface and the invoice content (labels,
  code lists, number and date formats) follow the system language, or the
  language chosen in the app's main menu.
- **Desktop integration.** Drag and drop, *Open with* from the file manager
  or Windows Explorer, a recent files list, one invoice per window. The
  window size is remembered, but never restored in a way that hides the
  window.

## Strictly local, read-only

Invoices contain personal and business data. E-Invoice Viewer therefore:

- **never opens a network connection** and never loads external content —
  links in an invoice are shown as text, never fetched;
- **never writes to or next to the opened file** — files are only written
  through an explicit *Save as…*;
- **runs the validation inside the app** — no Java runtime, no external
  validator, no web service.

## Supported formats

| Input | Details |
|---|---|
| ZUGFeRD / Factur-X PDF | Embedded `factur-x.xml`, `zugferd-invoice.xml` or `xrechnung.xml` |
| XRechnung | CII and UBL |
| UN/CEFACT CII | `CrossIndustryInvoice` |
| OASIS UBL 2.1 | `Invoice` and `CreditNote` |

Profiles EN 16931 (Comfort), XRechnung, Basic and Extended are shown in full.
Minimum and Basic WL are shown with a note that they are not complete
invoices; they are checked against the XML schema, but the EN 16931 rules are
not applied to them.

## Validation

The app first checks the invoice against its **XML schema** with libxml2
([ADR 0006](docs/adr/0006-xml-schema-validation-libxml2.md)), then evaluates
the **official, unmodified Schematron rules** with a pure-Rust XPath 3.1
engine ([xee](https://github.com/Paligo/xee)):

| Rule set | Version |
|---|---|
| XML schemas | UN/CEFACT CII D16B, OASIS UBL 2.1 |
| CEN EN 16931 validation artefacts (CII and UBL) | 1.3.17 |
| KoSIT XRechnung Schematron | 2.6.0 (XRechnung 3.0.2) |

The results are tested against the reference output of the official KoSIT
validator for its public test invoices. How and why this works is described in
[ADR 0002](docs/adr/0002-validation-engine.md).

> [!NOTE]
> The validation is meant to help you read and check invoices. It is not a
> certified validation, and this project is not affiliated with KoSIT or CEN.

## Download

Every [release](https://github.com/loge5/e-invoice-viewer/releases/latest)
has ready-to-run files for Linux and Windows; nothing else needs to be
installed.

### Linux

Each release has an **AppImage** for x86_64 Linux: one file that runs on
current distributions (glibc 2.39 or newer — Ubuntu 24.04, Debian 13,
Fedora 40, Arch, openSUSE Tumbleweed and newer) without installing anything.
Download `E-Invoice_Viewer-<version>-x86_64.AppImage` from the
[releases page](https://github.com/loge5/e-invoice-viewer/releases/latest),
then:

```bash
chmod +x E-Invoice_Viewer-*-x86_64.AppImage
./E-Invoice_Viewer-*-x86_64.AppImage path/to/invoice.pdf
```

On systems without FUSE, add `--appimage-extract-and-run`. To add the
AppImage to the app grid and to *Open with*, use a tool such as
[Gear Lever](https://flathub.org/apps/it.mijorus.gearlever) or AppImageLauncher.
The AppImage uses the same settings as an installed build.

### Windows

For 64-bit Windows 10 (1809 or later) and Windows 11, each release has two
files:

- `E-Invoice_Viewer-<version>-windows-x86_64-setup.exe` installs the app for
  the current user, without administrator rights: Start menu entry, and
  *Open with* for PDF and XML files (never as the default app). Uninstall it
  in Settings → Apps.
- `E-Invoice_Viewer-<version>-windows-x86_64.zip` is portable: unpack it
  anywhere and start `bin\e-invoice-viewer.exe`.

The program is not signed, so Windows SmartScreen may warn about an unknown
publisher on the first start. Choose **More info → Run anyway** if you
downloaded the file from this repository's releases page. The language
follows the Windows region format, or the choice in the app's menu.

### Building the AppImage

To build the AppImage yourself (needs Docker):

```bash
scripts/appimage.sh
```

It builds in an Ubuntu 24.04 container and writes
`target/appimage/E-Invoice_Viewer-<version>-x86_64.AppImage`.

## Installation from source (Linux)

The Windows files are built only by the release workflow
(`.github/workflows/release.yml`, MSYS2 on GitHub Actions); there is no
documented local Windows build.

### 1. Install the build dependencies

You need Rust 1.92 or newer ([rustup](https://rustup.rs)) and the development
files of GTK 4.12+, libadwaita 1.5+, Poppler (GLib), GtkSourceView 5 and
libxml2, GNU gettext for the translations, and libclang (Clang) to generate
the libxml2 bindings.

```bash
# Arch Linux
sudo pacman -S --needed base-devel clang gettext gtk4 libadwaita poppler-glib gtksourceview5 libxml2
```

```bash
# Debian / Ubuntu (24.04 or newer)
sudo apt install build-essential gettext libclang-dev libgtk-4-dev libadwaita-1-dev libpoppler-glib-dev libgtksourceview-5-dev libxml2-dev
```

```bash
# Fedora
sudo dnf install clang-devel gcc gettext gtk4-devel libadwaita-devel poppler-glib-devel gtksourceview5-devel libxml2-devel
```

### 2. Build and install

Clone this repository, then install for the current user (no root needed):

```bash
git clone https://github.com/loge5/e-invoice-viewer.git
cd e-invoice-viewer
scripts/install.sh
```

This builds a release binary, copies it to `~/.local/bin/e-invoice-viewer` and
adds a desktop entry and the app icon, so that the app shows up in the app grid and under
*Open with* for PDF and XML files. It never makes itself the default
application. To remove it again:

```bash
scripts/install.sh --uninstall
```

To just try it without installing:

```bash
cargo run --release -p e-invoice-viewer -- path/to/invoice.pdf
```

## Usage

Open an invoice with **Open…**, by dropping it onto the window, with
*Open with* from the file manager or Explorer, or on the command line:

```bash
e-invoice-viewer invoice.pdf
```

| Shortcut | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>O</kbd> | Open an invoice |
| <kbd>Ctrl</kbd>+<kbd>W</kbd> | Close the invoice, or the window |
| <kbd>Ctrl</kbd>+<kbd>Q</kbd> | Quit |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Search in the XML |
| <kbd>Ctrl</kbd>+<kbd>+</kbd> / <kbd>Ctrl</kbd>+<kbd>-</kbd> | Zoom the PDF in / out |
| <kbd>Ctrl</kbd>+<kbd>0</kbd> | Fit the PDF to the pane width |

The app speaks German and English. It follows the system language (on
Windows: the region format) and falls back to English; the main menu
(<kbd>F10</kbd>) lets you choose the language, which is remembered. Settings
and the recent files list are kept in `~/.config/e-invoice-viewer` on Linux
and in `%LOCALAPPDATA%\e-invoice-viewer` on Windows.

## Limitations

- Linux (x86_64) and Windows 10/11 (x64) only; no macOS. There are no
  distribution packages yet (an AUR package is planned), and the Windows
  files are not signed.
- Display only: invoices cannot be created or edited.
- PDF/A conformance and ZUGFeRD-specific profile rules are not checked.

## How it is built

The workspace has two crates with a strict boundary:

- [`crates/core`](crates/core) — all domain logic, no GUI dependencies:
  reading PDFs and XML, detecting syntax and profile, mapping to the field
  model, validation, attachments.
- [`crates/app`](crates/app) — the GTK4 / libadwaita user interface, presentation
  only.

The project is developed spec by spec: every feature starts as a reviewed
specification in [`specs/`](specs/) with acceptance criteria, which are traced
to automated tests or documented manual checks. Requirements are summarised in
[docs/requirements.md](docs/requirements.md), architecture decisions live in
[docs/adr/](docs/adr/).

## Development

Changes are listed in [CHANGELOG.md](CHANGELOG.md); how a release is made is
described in [docs/releasing.md](docs/releasing.md), and how the bundled rules
are updated in [docs/rule-updates.md](docs/rule-updates.md).

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
# crates/core must not depend on GUI libraries: prints nothing
cargo tree -p e-invoice-viewer-core -e normal | grep -E "gtk|glib|poppler|adw"
```

Test invoices are synthetic or taken from public sample sets
([docs/fixtures.md](docs/fixtures.md)); real invoices are never committed.

## Licence

E-Invoice Viewer is free software under the
[GNU General Public License v3.0 or later](LICENSE),
because it uses Poppler ([ADR 0001](docs/adr/0001-gtk4-poppler-gpl.md)).

Bundled third-party material keeps its licence:

- CEN EN 16931 validation artefacts — European Union Public Licence 1.2
  ([`crates/core/rules/cen-1.3.17`](crates/core/rules/cen-1.3.17))
- KoSIT XRechnung Schematron and validator configuration — Apache License 2.0
  ([`crates/core/rules`](crates/core/rules))
- XML schemas UN/CEFACT CII D16B and OASIS UBL 2.1 — redistributed unchanged
  under their copyright notices
  ([`crates/core/rules/xsd-2026-08-31`](crates/core/rules/xsd-2026-08-31))
- KoSIT test invoices — Apache License 2.0
  ([`crates/core/tests/fixtures/kosit`](crates/core/tests/fixtures/kosit))

The AppImage additionally contains unmodified libraries from Ubuntu 24.04,
mainly GTK 4, libadwaita, GLib, Pango, Cairo, GtkSourceView (LGPL) and Poppler
(GPL). The file `bundled-packages.txt`, attached to each release and included
in the AppImage, lists every bundled package with its version, licence and
the address of its source; the licence texts are in the AppImage under
`usr/share/doc`. The Windows files contain unmodified MSYS2 (MinGW)
builds of the same libraries, listed in `bundled-packages-windows.txt` with
their licences and source addresses.
