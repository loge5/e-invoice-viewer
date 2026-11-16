# 0001 — GTK4/libadwaita, Poppler for PDF, licence GPL-3.0-or-later

- Status: accepted
- Date: 2026-11-16

## Context

E-Invoice Viewer is a Linux/GNOME desktop viewer. It has to render PDF pages.
The author chose GTK4 as the UI toolkit; there is precedent in
`command-palette` (gtk4-rs).

## Decision

- UI: GTK 4 + libadwaita via `gtk4-rs` / `libadwaita-rs`.
- PDF rendering: Poppler via its GLib bindings (rendering into a Cairo surface).
- The application is licensed **GPL-3.0-or-later**, as Poppler is GPL.

## Alternatives

| Option | Why not |
|---|---|
| PDFium (`pdfium-render`) | Permissive licence, but needs a separately shipped binary library that is not packaged on Arch; extra distribution effort |
| MuPDF | AGPL, stricter than GPL without benefit here |
| hayro (pure Rust) | No native dependency, but young; rendering fidelity for real-world invoices is unproven. Worth re-evaluating later |

## Consequences

- Poppler (`poppler-glib`) is a runtime dependency, available on every
  GNOME system.
- All dependencies must be GPL-3.0 compatible (checked with
  `cargo deny check licenses`).
- Rendering lives in `crates/app`; `crates/core` stays licence-neutral and free
  of GUI dependencies.
