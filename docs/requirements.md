# Requirements summary

Result of step 1 (clarify) of the [idea → spec](../ai-harness/docs/idea-to-spec.md)
pipeline. Decided by the author on 2026-11-16: name **E-Invoice Viewer** (initially XRechnungsLupe), GTK4,
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
| 2 | Profiles | Full field view for EN 16931 (Comfort), XRechnung, Basic and Extended. Minimum and Basic WL are shown with a notice that they are not complete invoices *(default)* |
| 3 | Editing | **Display only.** Creating or editing invoices is out of scope |
| 4 | Attachments | Embedded attachments of the invoice (e.g. delivery note) are listed and can be saved. Comes in a later slice *(default)* |
| 5 | Rule set | EN 16931 business rules (CEN) plus XRechnung CIUS (BR-DE-*) *(default)*. ZUGFeRD-specific profile rules are a later extension |
| 6 | Validation runtime | A self-contained binary: no Java, no network. Technical feasibility is clarified in a spike first *(default)* |
| 7 | PDF/A conformance | Out of scope. Only checked: an invoice XML is embedded and can be extracted *(default)* |
| 8 | Field view | Own structured view grouped by sections (seller, buyer, payment, line items, totals, …), every field labelled with its BT/BG number *(default)* |
| 9 | Field ↔ XML | Selecting a field highlights the matching XML element. Comes in a later slice *(default)* |
| 10 | Windows & reopening | One invoice per window. **No** automatic reopening of the last file on start; a "recent files" list instead *(default)* |
| 11 | Opening files | File dialog, drag & drop, command line argument, "Open with" from the file manager *(default)* |
| 12 | Language | German UI, prepared for translation *(default)* |
| 13 | Toolkit | **GTK4 + libadwaita**, Linux/GNOME only |
| 14 | PDF rendering & licence | Poppler; the app is therefore licensed **GPL-3.0-or-later** *(default)* |
| 15 | Distribution | Local `cargo build`/install first; AUR package later; Flatpak out of scope for now *(default)* |

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

- **KoSIT / CEN validation artefacts:** EN 16931 Schematron rules (CII + UBL)
  and the XRechnung Schematron (BR-DE-*). Published under the Apache 2.0
  licence; the rule texts (codes, messages) are the reference for findings.
- **EN 16931 semantic model:** BT/BG numbers, field names and cardinalities,
  and their mapping to CII and UBL XPaths.
- **Factur-X / ZUGFeRD specification:** file names of the embedded XML
  (`factur-x.xml`, `zugferd-invoice.xml`, `xrechnung.xml`) and the profile
  identifiers (guideline IDs in BT-24).

## Biggest technical risk

The official rules are written as Schematron with XSLT 2.0 / XPath 2.0, and
Rust has no mature processor for that (in Java this is done with Saxon).
Options to evaluate in the spike:

1. Evaluate the Schematron asserts with a Rust XPath engine (e.g. `xee`,
   XPath 3.1) directly against the XML.
2. Implement the rules natively in Rust (large, but no foreign engine).
3. Optionally call an external validator (KoSIT/Java) if available. This
   contradicts decision 6 and is only a fallback.
