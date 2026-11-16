# AGENTS.md

Entry point for AI coding agents working on **E-Invoice Viewer**. Keep it short;
the detail lives in the linked docs. Read them before planning or coding.

## Project

A strictly local GTK4 desktop viewer for electronic invoices (ZUGFeRD /
Factur-X PDFs, XRechnung CII/UBL): PDF and invoice fields side by side,
validation findings next to the affected field, raw XML view. Display only.
See [docs/requirements.md](docs/requirements.md).

## Method: ai-harness

This project follows the vendored [ai-harness](ai-harness/README.md):

- Specs live in [`specs/`](specs/), format: [spec-format](ai-harness/docs/spec-format.md).
- Pipelines: [idea → spec](ai-harness/docs/idea-to-spec.md), then
  [implement](ai-harness/docs/implement.md), one reviewed spec at a time.
- Every session: [session lifecycle](ai-harness/docs/session-lifecycle.md).
- **Guidelines (read before any code):**
  [architecture](ai-harness/guidelines/architecture.md) ·
  [coding + tools check](ai-harness/guidelines/coding.md).
- Decisions: [`docs/adr/`](docs/adr/).

Only specs with `status: reviewed` may be implemented. The author decides at
every gate; the AI proposes.

## Hard boundaries

🚫 **Never:**
- Open network connections or load external content in any view.
- Write to or next to the opened invoice file. Writing happens only through
  explicit "save as" actions.
- Put domain logic (parsing, mapping, validation) into `crates/app`.
- Add GUI dependencies to `crates/core`.
- Commit real invoices. Test fixtures must be synthetic or from public sample
  sets with a compatible licence.

⚠️ **Ask first:** new dependencies (especially those with licence impact),
changes to the crate split, changes to the validation engine.

## Definition of Done (summary)

- The spec's acceptance criteria are met, each traced to a test or to a
  documented manual check for GUI-bound criteria.
- The tools check in [coding.md](ai-harness/guidelines/coding.md) is clean.
- The spec is set to `implemented`; docs/ADRs are updated where relevant.
