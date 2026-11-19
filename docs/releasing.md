# Releasing E-Invoice Viewer

A release is a tag `v<version>` on GitHub. The release workflow
(`.github/workflows/release.yml`) then builds the AppImage, the Windows ZIP
and the Windows installer and creates a **draft** release with them; the
author publishes it.

## Steps

1. **Changelog**: move the entries of `## [Unreleased]` in `CHANGELOG.md`
   into a new section `## [<version>] - <date>` and update the compare links
   at the end.
2. **Version**: set `version` in `[workspace.package]` of `Cargo.toml`.
3. **Lock file**: update the workspace crates in `Cargo.lock`:

   ```bash
   cargo update --workspace --offline
   ```

4. **AppStream**: add a `<release version="<version>" date="<date>">` entry
   with an English and a German description at the top of `<releases>` in
   `data/de.loges.EInvoiceViewer.metainfo.xml`.
5. **Check** before committing:

   ```bash
   scripts/appimage/check-tag.sh v<version>
   cargo test --workspace --locked
   ```

6. **Commit** as `chore(release): <version>`, push `main`, then tag and push
   the tag:

   ```bash
   git tag v<version>
   git push github main v<version>
   ```

7. **Publish**: when the workflow is done, check the draft release on GitHub
   (AppImage, ZIP, installer, both package lists) and publish it.

## What is checked automatically

- `scripts/appimage/check-tag.sh`, the first step of both build jobs, fails
  unless the tag equals `v` + the version in `Cargo.toml`, `Cargo.lock` has
  that version for both workspace crates, the AppStream file has a release
  entry for it, and `CHANGELOG.md` has a section for it.
- The Windows job runs all tests before packaging.
- A release is all or nothing: if one platform fails, no draft is created.
  Fix the cause and release the next patch version (a published tag is not
  moved).
