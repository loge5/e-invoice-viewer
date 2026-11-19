---
id: 0029
title: Project docs - changelog, release guide, rule updates, requirements
status: draft
created: 2026-11-19
depends_on: [0020, 0022]
---

## Context

The project has a README and specs, but no changelog, no written release
procedure and no description of how the bundled rules are updated. The
release 0.2.0 failed because a step was forgotten. `docs/requirements.md`
still says "Linux/GNOME only" and "Flatpak out of scope". For 1.0 the
project documentation is complete and current (author, 2026-11-19).

## Requirement

- **`CHANGELOG.md`** in the format of "Keep a Changelog", with the versions
  0.1.0 and 0.2.1 (0.2.0 was never published) and a section "Unreleased";
  user-facing changes only, in English.
- **`docs/releasing.md`**: the steps of a release in order: changelog entry,
  version in `Cargo.toml`, `cargo update --workspace`, AppStream release
  entry, commit, tag, push, publish the draft release; and what the tag check
  and the release workflow verify.
- **The tag check** (`scripts/appimage/check-tag.sh`) also requires a
  changelog section for the version.
- **`docs/rule-updates.md`**: how to update the bundled CEN rules, the
  XRechnung rules and configuration, and the XML schemas: where they come
  from, which files to replace, which version strings to change, how to
  regenerate the KoSIT reference results, which tests to run. It also says
  how users learn about new rule versions (release notes; the app never
  checks online).
- **`docs/requirements.md`** reflects the decisions made since: Linux and
  Windows (decision 13), AppImage and Windows ZIP/installer as distribution,
  no Linux packages and no code signing before 1.0 (decision 15), XML schema
  check (decision 5), Extended profile (0027).
- **README**: links to the changelog and the release guide for contributors;
  the SmartScreen note stays; the Windows signing status is stated in
  "Limitations".

## Acceptance criteria

- [ ] `CHANGELOG.md` exists with 0.1.0, 0.2.1 and Unreleased; its 0.2.1
      section names Windows support and the new fields of 0021.
- [ ] `scripts/appimage/check-tag.sh v0.2.1` passes; with a version that has
      no changelog section it fails with a message naming `CHANGELOG.md`
      (automated test).
- [ ] `docs/releasing.md` and `docs/rule-updates.md` exist, and every command
      in them is a command of the repository (checked by reading; the rule
      update procedure is the one used for 0009, 0010 and 0023).
- [ ] `docs/requirements.md` contains no statement that contradicts an
      implemented spec (checked by reading against specs 0001–0026).
- [ ] The README links to `CHANGELOG.md` and `docs/releasing.md`.

## Out of scope

- A user manual beyond the README.
- Translating the documentation.

## Test list (proposed)

- T1 `tag_check_requires_changelog_entry` (app, `packaging.rs`): a temporary
  copy of the checked files with and without a section for the version
  (AC2).
- T2 `readme_links_changelog_and_release_guide` (AC5).
- Reading checks for AC1, AC3 and AC4, recorded in the spec.
