<div align="center">
  <a href="https://getnodalia.com">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/danielmigueltejedor/nodalia-web/main/assets/brand/nodalia-wordmark-white.svg">
      <img src="https://raw.githubusercontent.com/danielmigueltejedor/nodalia-web/main/assets/brand/nodalia-wordmark-black.svg" width="150" alt="Nodalia">
    </picture>
  </a>
  <br><br>
  <img src="https://raw.githubusercontent.com/danielmigueltejedor/lienzo/nodalia/assets/app-icon/hicolor/512x512/apps/ai.storyteller.photocraft.png" width="160" alt="Lienzo">
  <h1>Lienzo</h1>
  <p><strong>An image editor for the Nodalia suite, built on the PhotoCraft engine.</strong></p>

  <p>
    <img src="https://img.shields.io/badge/status-early%20stage-orange" alt="Early stage">
    <img src="https://img.shields.io/badge/engine-PhotoCraft-EE3B6B" alt="PhotoCraft engine">
    <img src="https://img.shields.io/badge/Rust-native-dea584?logo=rust&logoColor=black" alt="Native Rust">
    <img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-2ea44f" alt="MIT OR Apache-2.0">
  </p>

  <p>
    <a href="#overview">Overview</a> ·
    <a href="#what-you-get-today">Today</a> ·
    <a href="#build-and-run">Build</a> ·
    <a href="#project-status">Status</a> ·
    <a href="#keeping-up-with-upstream">Upstream</a> ·
    <a href="#the-nodalia-suite">Suite</a> ·
    <a href="#license">License</a>
  </p>
</div>

---

## Overview

**Lienzo** is the image editor of [Nodalia](https://getnodalia.com). It is built on the engine of
[PhotoCraft](https://github.com/storytold/photocraft), which lives in this repository and is kept up to
date with upstream.

> [!IMPORTANT]
> **Lienzo is at an early stage.** What runs today is the upstream application.
> Nodalia's own interface is not wired in yet and there are no published builds.

## What you get today

- The PhotoCraft engine.
- The upstream desktop application. Crate and binary names are still upstream's.
- The engine's own feature description, kept as [`docs/upstream-readme.md`](docs/upstream-readme.md).

## Build and run

You need a recent stable Rust toolchain.

```bash
git clone https://github.com/danielmigueltejedor/lienzo.git
cd lienzo
cargo run --release -p photocraft
```

The crate is still called `photocraft`; it will be renamed with the product.

## Project status

| Area | State |
|---|---|
| Engine | Imported from `storytold/photocraft` and tracked in [`docs/upstream.md`](docs/upstream.md) |
| Product name | Pending: crates and binary still carry upstream's name (`photocraft`) |
| App icon | Done: portrait of Diego Velázquez |
| Nodalia interface | Not started |
| Shared interface resources | Planned: the Nodalia apps will share interface components and styles |
| Installable packages | None published |

## Keeping up with upstream

[`docs/upstream.md`](docs/upstream.md) records which upstream commit is integrated. Updates are
merges of upstream's `main`, so Lienzo's own changes are kept and conflicts are resolved in the open.

## App icon

Nodalia app icons are black-and-white portraits of historical figures over the app colour.
Lienzo's is Diego Velázquez, the painter. Provenance and licence are in [`assets/app-icon/`](assets/app-icon/).

## The Nodalia suite

Lienzo is one of the applications of [Nodalia](https://getnodalia.com), an independent software
project developed in public by [Daniel Miguel Tejedor](https://danielmigueltejedor.com).

## License

`MIT OR Apache-2.0`, as upstream. The licence files and `NOTICE` of PhotoCraft are kept.
See [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).

## Credits

Lienzo would not exist without [PhotoCraft](https://github.com/storytold/photocraft) and its contributors.
