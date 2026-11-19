---
id: 0026
title: CI - tests on every push and pull request
status: reviewed
created: 2026-11-19
depends_on: [0020, 0022]
---

## Context

On GitHub the tests run only when a release is built (0022). The release
0.2.0 failed only at that point, because `Cargo.lock` had not been updated.
For 1.0, every change is checked when it is pushed (author, 2026-11-19).

## Requirement

- A workflow `ci.yml` runs on every push to any branch and on every pull
  request, on Linux and on Windows.
- **Linux job** (Ubuntu 24.04 with the build dependencies of the README):
  `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `cargo test --workspace --locked`, the architecture check
  (`crates/core` without GUI libraries) and the packaging tests that need
  `desktop-file-validate` and `appstreamcli`.
- **Windows job**: `cargo test --workspace --locked` in MSYS2 UCRT64, set up
  as in the release workflow.
- Failures are readable without signing in: a failing step reports its last
  output lines as an annotation (as `scripts/windows/ci-step.sh` does).
- Cargo's build cache is kept between runs (GitHub's own cache action), so
  that a run after a small change takes minutes, not the full build time.
- Only GitHub's own actions and `msys2/setup-msys2`, pinned by commit hash;
  the workflow only reads (`contents: read`).
- The README shows the state of the CI as a badge.

## Acceptance criteria

- [ ] Given a push to a branch, then the workflow runs both jobs; a test that
      fails on either system fails the run, and its annotation shows the
      failing test.
- [ ] Given a commit whose `Cargo.lock` does not match `Cargo.toml`, then
      the Linux job fails at `--locked` (checked once on a trial branch).
- [ ] A second run without changes uses the cache (shown in the log) and the
      Linux job takes under 10 minutes.
- [ ] A static check of the workflow (like T4 of 0022) shows: both triggers,
      both jobs with the steps named above, pinned actions, read-only
      permissions.
- [ ] The README shows the CI badge.

## Out of scope

- Building the AppImage and the Windows packages on every push (the release
  workflow does that).
- Fuzzing in CI (0025 runs locally).
- Required status checks or branch protection (settings of the GitHub
  repository, the author's).

## Notes

- The trial branch `windows` of 0022 is no longer needed once this runs.

- Review on 2026-11-19: reviewed by Claude, accepted by the author together
  with the test list.

## Test list (approved 2026-11-19)

- T1 `ci_workflow_checks_every_push` (app, `packaging.rs`): static check of
  `ci.yml`: triggers, jobs and steps, pinned actions, read-only permissions,
  cache (AC4).
- T2 `readme_shows_ci_badge` (AC5).
- P1 a push to a trial branch runs both jobs green; a deliberately broken
  test and a stale `Cargo.lock` on a trial branch fail the run with a
  readable annotation; a second run uses the cache (AC1–AC3).
