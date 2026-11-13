# Release notes

Every version has its notes here, written by hand: `docs/release-notes/vX.Y.Z.md`. They are the
text of the GitHub release, so people reading the release page see exactly this file.

## Rules, enforced by CI

- Bumping `version` in `[workspace.package]` of `Cargo.toml` needs `docs/release-notes/vX.Y.Z.md`
  in the same pull request, or the **Release notes are written** check fails
  (`scripts/check-release-notes.sh`, also run by the release workflows before building anything).
- A release is created from this file. When a file here changes on `nodalia`, the **Release notes**
  workflow copies it into the release of the same tag. It can also be run by hand with a tag.
- No generated lists ("What's Changed", "Full Changelog"), no placeholders, no mentions of forks,
  upstream or other products' names.

## Format

1. A one-sentence summary in Spanish, then the same in English in italics.
2. Optional `> [!IMPORTANT]` callout (Lienzo is at an early stage…).
3. `## Español` with `### Novedades` (bullets) and optionally `### Correcciones` and `### Notas`.
4. `## English` with the same sections (`### What's new`, `### Fixes`, `### Notes`).
5. `## Descargas · Downloads`: a table of the real file names of this version, plus the checksum
   files (`SHA256SUMS…`).

A first release uses `### Qué incluye` / `### What's in it` instead of the "what's new" headings.
Copy the latest file as a starting point.
