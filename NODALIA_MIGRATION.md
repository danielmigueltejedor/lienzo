# Nodalia migration: Lienzo

Category: priority app.
Status: in progress, as of 2026-11-13. Evidence: lienzo#39 (Cargo dependencies from
getnodalia/foundation) and this pull request (CI on Foundation's reusable workflows).

## Current state

Native Rust image editor (egui on wgpu, web build through WASM), default branch `nodalia`. Cargo
git dependencies all come from getnodalia/foundation. The gate, FreeBSD, Nix, packaging-lint and
AGENTS.md checks call getnodalia/foundation workflows pinned by commit; every other action is
pinned by SHA. Linux, macOS (DMG on a standard runner) and Windows installers come from Lienzo's own
release workflows.

Known red: the Linux gate (GPU canvas tests under lavapipe) and the Windows test legs fail on the
base branch too (seen on lienzo#38 and lienzo#39); they are not caused by the migration.

## Desired state

No workflow, dependency or document depends on storytold; the gate and Windows tests are green;
the interface uses Nodalia Design throughout.

## Inherited dependencies still pending

| Dependency | Where | Replacement | Blocked on |
|---|---|---|---|
| craft-common-actions `release.yml`, `refresh-contributors.yml` | `.github/workflows/release.yml` | Lienzo's own `release-*.yml`, plus a contributors step | checking whether `release.yml` is still needed |
| craft-common-actions `windows-arm64.yml` | `.github/workflows/windows-arm64.yml` | a Foundation workflow | writing it |
| storytold/photocraft-corpus | `xtask/src/corpus_pins.rs`, `docs/development.md` | a Foundation vendor branch with the same commits | mirroring it |
| Weekly sync from the upstream project | `.github/workflows/upstream-sync.yml` | none: intended, renames upstream names | — |

## Technical debt

- SonarCloud analysis was removed with this migration; CodeQL and Scorecard remain.
- The perf comparison job used the upstream project's main branch as its baseline; it now uses
  the merge base with `origin/nodalia`.

## Reusable Nodalia components

getnodalia/foundation (vendored crates, fonts, reusable workflows); getnodalia/nodalia-design
(theme, icons, `nodalia-update`).

## Risks and priorities

1. The red gate hides new regressions: fix the GPU canvas tests first.
2. Release still partly runs inherited workflows.

## Recommended order

1. Make the Linux gate and Windows tests green on `nodalia`.
2. Decide on `release.yml` and replace `windows-arm64.yml`.
3. Mirror photocraft-corpus into Foundation and repoint the pins.

## What the first agent should do

1. Read AGENTS.md and this file.
2. Reproduce the failing GPU canvas tests (`crates/ui-egui/tests/it/live_stroke_canvas.rs`).
