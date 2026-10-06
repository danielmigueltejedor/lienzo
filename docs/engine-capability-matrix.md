# Lienzo engine capability matrix

Status on 2026-10-06. "C++ engine" is the Patchy code `lienzo-gnome` links. "Rust engine" is `engine/lienzo-engine`, which has no pixel, document, or file implementation yet. "PhotoCraft" is `storytold/photocraft` at `909efc0f6df0a684d270019384e355c850642e42`, inspected as a workspace, not vendored.

Best implementation stays "C++ engine" until a comparison records another result. GNOME coverage is the control migration in [gnome-architecture.md](gnome-architecture.md), not a second engine.

| Feature | C++ engine | PhotoCraft crate (upstream) | Rust engine | Best | Migration | Tests |
|---|---|---|---|---|---|---|
| Document model | `patchy::Document` | `photocraft-doc` | none | C++ engine | keep | core |
| PSD/PSB | `patchy_psd` | `photocraft-psd` | none | C++ engine | compare before any port | core, Photoshop open checks |
| PXD | `src/formats/pxd_document_io` | not seen in the crate list | none | C++ engine | keep | `pxd` core tests |
| Compositor | CPU reference in `patchy_render` | `photocraft-compose`, `photocraft-gpu` (wgpu 30) | none | C++ engine | GPU only if it matches the CPU pins | render tests, Testy |
| 16/32-bit, CMYK, Lab, ICC, HDR | partial, see format and color docs | `cms`, `color`, `raster`, `raw` | none | open | benchmark, do not assume the Rust crates win | not compared |
| Paint and brushes | `paint_brush_*`, brush tips | `photocraft-paint` | none | C++ engine | keep the canary | `tool_write_paths_digest_baseline` |
| Retouch | `retouch_brush`, heal membrane, spot-heal map | `photocraft-algo` (not compared) | none | C++ engine | GNOME still lacks spot heal and patch | core |
| Text | calibrated Qt pipeline, Txt2 | `photocraft-text` | none | C++ engine | do not add a third shaper | UI and PSD text tests |
| Vectors | `src/core` vector model | `photocraft-vector`, `geom` | none | C++ engine | keep fixtures | vector tests |
| Smart objects and filters | C++ engine | not established by this audit | none | C++ engine | GNOME does not expose them yet | core |
| Filters, liquify, warp | C++ engine, much of the gesture still in `src/ui` | `photocraft-ops` (not compared) | none | C++ engine | port the gesture to GNOME, not a new algorithm | core and UI |
| Adjustment layers | C++ engine, GNOME dialog | not compared | none | C++ engine | keep | core |
| Automation | Qt scripting and MCP | `photocraft-automation` | none | C++ engine | class B, stays out of the Rust crate for now | UI and Python |
| Plugins | Windows 8BF host | `photocraft-plugins` | none | not decided | 8BF is not a Linux feature | partial |
| Provenance | git history plus weekly sync | upstream repository | observed SHAs only | Rust record plus C++ history | keep both | `cargo test -p lienzo-engine` |

PhotoCraft's own README says the project is early alpha. That is not a reason to ignore it, and it is not a reason to import it.
