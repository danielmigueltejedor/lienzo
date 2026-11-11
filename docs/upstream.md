# Lienzo upstream

| Field | Value |
|---|---|
| Upstream | `storytold/photocraft` |
| Last observed | `de7e1eec45eddbb434df2fdd27abba12aac24949` |
| Last evaluated | `de7e1eec45eddbb434df2fdd27abba12aac24949` |
| Last integrated | `de7e1eec45eddbb434df2fdd27abba12aac24949` imported with full history on 2026-11-10 |
| Skipped | none in the baseline import |
| Adapted | app icon; every PhotoCraft name renamed to Lienzo (`.github/rebrand.toml`) |
| Conflicting | none recorded |

Lienzo shares history with upstream, but never merges `upstream/main` directly: upstream still
calls the app PhotoCraft, and its crates, paths, app id and environment variables say so.

## How upstream arrives renamed

`scripts/rebrand.py` renames upstream's names to Lienzo's, in file contents and in paths, from the
rules in `.github/rebrand.toml` (literal replacements in order, spans to keep such as links and
upstream's repository name, files to skip such as `NOTICE`, and lines that keep Lienzo's own value
such as the version). A line marked `rebrand: keep` is left alone, for compatibility code that has
to name the old product.

The branch `upstream-lienzo` mirrors upstream with Lienzo's names. Each of its commits holds the
rebranded tree of one upstream commit, has the previous mirror commit and that upstream commit as
parents, and names it in an `Upstream-Commit:` trailer. The weekly sync
(`.github/workflows/upstream-sync.yml`) adds a mirror commit for upstream's new head and merges
that, so upstream's changes arrive already speaking Lienzo, and a conflict is a real one.

The first mirror commit is upstream `de7e1eec` renamed; it was merged with `-s ours` when the
whole repository was renamed, so only later upstream changes come through it.

`python3 scripts/rebrand.py check` (run in CI) lists anything still using an upstream name. The
script is not Lienzo-specific: another app built on an upstream project copies it with its own
rule file.
Before this import Lienzo was built on Patchy (C++/Qt); that tree stays on the `main` branch
until the default branch is switched.
