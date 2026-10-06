# Lienzo upstream tracking

Rules: [creative-suite-architecture.md](creative-suite-architecture.md). Dates are the audit day. Integrated means the commit's behavior is in this tree, not that the SHA is an ancestor of a fork.

## Patchy

Repository: `SethRobinson/Patchy`. License: MIT.

| Field | Commit or note |
|---|---|
| Last observed | `4816d450e6a94e954017629745c06044186b3dec` (2026-10-06, "doc tweaks") |
| Last evaluated | same SHA, repository metadata and the existing weekly sync only. The patch series between GitHub `main` `cf0fceba` and this SHA was not replayed in this change. |
| Last integrated | The C++ engine in this checkout. The weekly workflow `.github/workflows/weekly-patchy-sync.yml` cherry-picks non-protected commits onto Lienzo `main`. This document does not claim that workflow has applied `4816d450`. |
| Skipped | Protected identity paths: packaging, README, `AGENTS.md`, release scripts, desktop id. |
| Adapted | `src/ui-gnome` calls the engine. It is not a Patchy file. |
| Conflicting | None recorded in this audit. |

Patchy remains the engine `lienzo-gnome` calls. PhotoCraft is the Rust engine imported beside it.

## PhotoCraft

Repository: `storytold/photocraft`. License: MIT OR Apache-2.0. UI in that repository is egui. Lienzo does not take that UI.

| Field | Commit or note |
|---|---|
| Last observed | `909efc0f6df0a684d270019384e355c850642e42` (2026-10-06, background jobs in the app) |
| Last evaluated | `909efc0f6df0a684d270019384e355c850642e42` |
| Last integrated | That commit is imported at `engine/photocraft`. `lienzo-gnome` still links the C++ engine. The Rust app builds from `engine/photocraft` and its binary name is `lienzo`. |
| Skipped | none in the import. Later syncs still classify commit by commit. |
| Adapted | The desktop binary name is `lienzo`. Crate names stay as imported so the workspace still builds. |
| Conflicting | The C++ compositor and the PhotoCraft compositor both exist. The GTK shell still calls the C++ one. Replacing that call is a later port, after the PSD corpus is compared. |

`engine/lienzo-engine` records these two SHAs. It does not wrap the imported crates.
