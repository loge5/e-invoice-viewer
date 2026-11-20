# Requirements summary

Result of step 1 (clarify) of the idea → spec pipeline. Decided by the author on 2026-11-16: name **E-Invoice Viewer** (initially XRechnungsLupe), GTK4,
display only. Everything else marked *(default)* was proposed by the AI and
accepted without further discussion. It can be revisited at any time.

## Idea

A fast, stable, strictly local desktop viewer for German/European electronic
invoices (ZUGFeRD / Factur-X PDFs and XRechnung). It shows the PDF and the
structured invoice fields side by side, shows validation findings directly next
to the affected field, and can display the raw XML.

Motivation: the existing Java viewer (Quba) has an unstable layout (sidebar cut
off on resize, missing redraws), always reopens the last PDF without a visible
way to close it, and crashed on Linux.

## Capabilities

1. Open an invoice file:
   - a ZUGFeRD / Factur-X PDF (PDF/A-3 with embedded XML), or
   - a plain XML invoice in CII or UBL syntax *(default)*.
2. Show the PDF and the structured invoice fields side by side. For plain XML
   there is no PDF, so only the fields are shown.
3. Show validation findings (error / warning) directly next to the affected
   field, plus a summary of all findings below the fields.
4. Show the raw XML of the invoice.

## Decisions & defaults

| # | Topic | Decision |
|---|---|---|
| 1 | Input formats | ZUGFeRD/Factur-X PDF; plain XML in CII and UBL *(default)* |
| 2 | Profiles | Full field view for EN 16931 (Comfort), XRechnung, Basic and Extended. Minimum and Basic WL are shown with a notice that they are not complete invoices *(default)*. For 1.0, every EN 16931 business term is shown (spec 0024), and content beyond EN 16931 (Extended) is planned (spec 0027) |
| 3 | Editing | **Display only.** Creating or editing invoices is out of scope |
| 4 | Attachments | Embedded attachments of the invoice (e.g. delivery note) are listed and can be saved (spec 0013) |
| 5 | Rule set | XML schema (CII D16B, UBL 2.1; spec 0023, ADR 0006), EN 16931 business rules (CEN) and XRechnung CIUS (BR-DE-*). ZUGFeRD-specific profile rules are planned for 1.0 (spec 0027), subject to the licence of the FeRD artefacts |
| 6 | Validation runtime | A self-contained binary: no Java, no network. Technical feasibility is clarified in a spike first *(default)* |
| 7 | PDF/A conformance | Out of scope. Only checked: an invoice XML is embedded and can be extracted *(default)* |
| 8 | Field view | Own structured view grouped by sections (seller, buyer, payment, line items, totals, …), every field labelled with its BT/BG number *(default)* |
| 9 | Field ↔ XML | Fields and XML elements are linked in both directions (spec 0014) |
| 10 | Windows & reopening | One invoice per window. **No** automatic reopening of the last file on start; a "recent files" list instead *(default)* |
| 11 | Opening files | File dialog, drag & drop, command line argument, "Open with" from the file manager or Windows Explorer |
| 12 | Language | The 24 official EU languages (German and English since 2026-11-18, the others since 2026-11-20, not checked by native speakers), following the system language, switchable in the app; translations as gettext `.po` files ([ADR 0003](adr/0003-translations-gettext-embedded.md), specs 0017–0019, 0030) |
| 13 | Toolkit & platforms | **GTK4 + libadwaita**. Linux (decided 2026-11-16) and, since 2026-11-19, Windows 10/11 x64 ([ADR 0005](adr/0005-windows-build-msys2-github-actions.md), spec 0022). No macOS |
| 14 | PDF rendering & licence | Poppler; the app is therefore licensed **GPL-3.0-or-later** *(default)* |
| 15 | Distribution | AppImage for Linux ([ADR 0004](adr/0004-appimage-built-in-container.md)), portable ZIP and installer for Windows (ADR 0005), and `scripts/install.sh` from source. Decided 2026-11-19: no Linux distribution packages (AUR, Flathub) and no Windows code signing before 1.0 |

## Assumptions & implicit constraints

- **Strictly offline.** Invoices contain personal and business data. The app
  never opens network connections, and the view never loads external content.
- **Read-only on the user's files.** It never writes to or next to the opened
  file. The only writes are explicit "save as" actions (attachments, XML export).
- **Stable layout.** Resizing never cuts off or leaves parts of the window
  unpainted; minimum window size is enforced; window geometry that is invalid
  on the current screens is never restored.
- **Robust against bad input.** Malformed PDFs or XML never crash the app.
  They lead to an understandable error message.
- **Performance.** A typical invoice (1–5 pages, < 1 MB) is displayed within
  2 s on the reference machine.
- **Reference machine** for all time limits in specs: the author's laptop,
  Intel Core Ultra 7 155H, 30 GB RAM, Arch Linux, GNOME.
- **Sample invoices** used in acceptance criteria are defined in
  [fixtures.md](fixtures.md).

## Dependencies on existing systems / external knowledge

- **KoSIT / CEN validation artefacts:** EN 16931 Schematron rules (CII + UBL,
  EUPL 1.2) and the XRechnung Schematron (BR-DE-*, Apache 2.0); the rule
  texts (codes, messages) are the reference for findings. The XML schemas
  CII D16B (UN/CEFACT) and UBL 2.1 (OASIS) from the KoSIT configuration.
  How they are updated: [rule-updates.md](rule-updates.md).
- **EN 16931 semantic model:** BT/BG numbers, field names and cardinalities,
  and their mapping to CII and UBL XPaths.
- **Factur-X / ZUGFeRD specification:** file names of the embedded XML
  (`factur-x.xml`, `zugferd-invoice.xml`, `xrechnung.xml`) and the profile
  identifiers (guideline IDs in BT-24).

## Biggest technical risk (resolved)

Resolved by the spike of spec 0001: option 1, see
[ADR 0002](adr/0002-validation-engine.md). The original assessment:

The official rules are written as Schematron with XSLT 2.0 / XPath 2.0, and
Rust has no mature processor for that (in Java this is done with Saxon).
Options to evaluate in the spike:

1. Evaluate the Schematron asserts with a Rust XPath engine (e.g. `xee`,
   XPath 3.1) directly against the XML.
2. Implement the rules natively in Rust (large, but no foreign engine).
3. Optionally call an external validator (KoSIT/Java) if available. This
   contradicts decision 6 and is only a fallback.
