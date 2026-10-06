# Nodalix creative suite architecture

Audited 2026-10-06 from GitHub default-branch commits. SHAs below are the commits that were read. A later commit is not integrated until it is named in [upstream.md](upstream.md).

Each Nodalix application owns its engine. An upstream is an input: research, commits, and a compatibility reference. It is not a dependency that defines the product. The shipping question is: if that upstream vanished tomorrow, could Nodalix keep developing the engine from the copy it already has, with its tests and its license notices. The answer is yes only after that copy lives in the Nodalix repository. A remote Git dependency does not count.

Public names are Lienzo, Pluma, Toma, Luz, Papel, Pulso, Pliego, Forja, and Vinilo. Upstream names stay out of branding, interface strings, and public APIs. Internal C++ names that already match Patchy stay, because that is how Lienzo still accepts Patchy commits. New Rust APIs use `lienzo::`, `pluma::`, `toma::`, `luz::`, `papel::`, `pulso::`, `pliego::`, and `forja::`.

## Inventory

| Application | Repository | Exists | What the tree actually is | App id verified |
|---|---|---|---|---|
| Lienzo | [danielmigueltejedor/lienzo](https://github.com/danielmigueltejedor/lienzo) | yes | Patchy C++ engine plus `lienzo-gnome` (GTK 4 / libadwaita). PhotoCraft `909efc0f` is imported at `engine/photocraft`; the desktop binary name there is `lienzo`. GitHub `main` is `cf0fcebabd3563b44b3df500ea865d7f67380f42`. | `com.nodalix.lienzo` |
| Pluma | [danielmigueltejedor/pluma](https://github.com/danielmigueltejedor/pluma) | yes | VectorCraft `571881a830830bf8f221aa8e2635eb999db90708`. Desktop binary `pluma`. Previous Graphite commit `68c3c759` stays as history. GTK shell is not wired. | not verified |
| Forja | [danielmigueltejedor/forja](https://github.com/danielmigueltejedor/forja) | yes | Open CAD Studio at `ab0682153dd63af6185c21ea4ef6e2f1230d25c3` (2026-09-29). Package name is still `OpenCADStudio`. UI is Iced, not libadwaita. | not verified |
| Vinilo | [danielmigueltejedor/vinilo](https://github.com/danielmigueltejedor/vinilo) | yes | Rust GNOME player. Crates `aguja`, `vinilo-core`, `vinilo`, `vinilod`. No Craft upstream. `3ba8d4f7881a285baea5c138abe71e64e5fdeb6f`. | not verified |
| Toma | [danielmigueltejedor/toma](https://github.com/danielmigueltejedor/toma) | yes | FilmCraft `771b614`. Desktop binary `toma`. Root import on `main`. GTK shell is not wired. | not verified |
| Luz | [danielmigueltejedor/luz](https://github.com/danielmigueltejedor/luz) | yes | LightCraft `265248c`. Desktop binary `luz`. Root import on `main`. GTK shell is not wired. | not verified |
| Papel | [danielmigueltejedor/papel](https://github.com/danielmigueltejedor/papel) | yes | PrintCraft `55c5817`. Desktop binary `papel`. Root import on `main`. GTK shell is not wired. | not verified |
| Pulso | [danielmigueltejedor/pulso](https://github.com/danielmigueltejedor/pulso) | yes | EffectCraft `2fadd14`. Desktop binary `pulso`. Root import on `main`. GTK shell is not wired. | not verified |
| Pliego | [danielmigueltejedor/pliego](https://github.com/danielmigueltejedor/pliego) | yes | DesignCraft `d582277`. Desktop binary `pliego`. Root import on `main`. GTK shell is not wired. | not verified |

There is no suite monorepo. This document lives in Lienzo because Lienzo is the first pilot. Other applications keep their own `docs/architecture.md` when their repositories diverge from upstream.

## Upstreams observed

| Upstream | Commit | License | Shape |
|---|---|---|---|
| [SethRobinson/Patchy](https://github.com/SethRobinson/Patchy) | `4816d450e6a94e954017629745c06044186b3dec` | MIT | C++/Qt document engine. This is the code Lienzo ships. |
| [storytold/photocraft](https://github.com/storytold/photocraft) | `909efc0f6df0a684d270019384e355c850642e42` | MIT OR Apache-2.0 | Rust workspace, edition 2024, `unsafe_code = forbid`, wgpu 30. Early alpha. UI is egui. |
| [storytold/vectorcraft](https://github.com/storytold/vectorcraft) | `571881a830830bf8f221aa8e2635eb999db90708` | MIT OR Apache-2.0 | Rust workspace imported into Pluma. Crates include `geom`, `pathops`, `svg`, `pdf`, `render`, `text`, `ui-egui`. Audit read was `ce9f3aa7`; the clone HEAD moved before import. |
| [storytold/filmcraft](https://github.com/storytold/filmcraft) | `771b614ad4c3fbefda5d668ce4384504b8c13f6d` | MIT OR Apache-2.0 | Rust workspace. Timeline, codecs, GPU, captions, export. UI is egui. |
| [storytold/lightcraft](https://github.com/storytold/lightcraft) | `265248c86fa3267709aea45633d0d135331e180c` | MIT OR Apache-2.0 | Rust workspace. RAW, catalog, develop, color, GPU. UI is egui. |
| [storytold/printcraft](https://github.com/storytold/printcraft) | `55c581760ab389b88f08997ab235eecdf1a1b002` | MIT OR Apache-2.0 | Rust workspace. PDF COS, annotations, forms, signatures, preflight. UI is egui. |
| [storytold/effectcraft](https://github.com/storytold/effectcraft) | `2fadd14f09bb064cab8f1f8fe85af601a08b6e53` | MIT OR Apache-2.0 | Rust workspace. Keyframes, effects, expressions, media, GPU. UI is egui. |
| [storytold/designcraft](https://github.com/storytold/designcraft) | `d582277cc79c4bc9cee174b25b13fb11e6957aa5` | MIT OR Apache-2.0 | Rust workspace. Pages, text, IDML, PDF, EPUB. UI is egui. |
| [GraphiteEditor/Graphite](https://github.com/GraphiteEditor/Graphite) | `e38fab22455c45068fcd1eb976afb0c088fa222d` | Apache-2.0 | Previous Pluma tree, commit `68c3c759`. No longer the engine. |
| [HakanSeven12/OpenCADStudio](https://github.com/HakanSeven12/OpenCADStudio) | `7aafd9c710a11762426119c8303206f2de4eba9d` | GPL-3.0 | Actual Forja lineage. Iced UI. Newer than Forja (`2026.40.1` vs Forja `2026.39.0`). |

PhotoCraft crates: `algo`, `automation`, `cms`, `codecs`, `color`, `compose`, `doc`, `engine`, `format`, `geom`, `gpu`, `io`, `ops`, `paint`, `plugins`, `psd`, `raster`, `raw`, `tablet`, `testkit`, `text`, `ui-egui`, `vector`. Apps: `photocraft`, `photocraft-cli`, `photocraft-web`. The GPU crate depends on `wgpu` 30 and the compose/raster/doc crates. The README calls the product early alpha.

Open CAD Studio is not one crate. `Cargo.toml` pins external git crates: `opencadkernel` `6146d31`, `opencadkernel-constraints` `6146d31`, `opencadcodec` `bff95f5`, `opencadgraph` `9635ba7`. Forja pins older revisions of the same crates (`7dc1e79`, `7dc1e79`, `8464e0a`, `2e40a7b`). The application sources under `src/` include `entities/`, `scene/`, `snap.rs`, `gpu_backend.rs`, `shaders/`, `command.rs`, and `plugin/`. Entity files include line, polyline, arc, circle, ellipse, spline, text, mtext, dimension, hatch, raster image, viewport, solid, solid3d, and mesh. `crates/ocs_plugin_api` 0.2.1 is GPL-3.0-only. Its default surface is a manifest and ribbon contract. The `host` feature pulls `opencadcodec` and dynamic loading. That is not a verified binary plugin ABI for Forja. Do not promise that an Open CAD Studio plugin loads in Forja.

## License wall

MIT and MIT OR Apache-2.0 code can meet inside Lienzo, Pluma, Toma, Luz, Papel, Pulso, and Pliego if every copied file keeps its copyright and license header.

GPL-3.0 code cannot be copied into those engines. Forja and Vinilo are GPL-3.0 today. A shared crate that links the Open CAD Studio kernel, `ocs_plugin_api`, or Vinilo becomes GPL-3.0. Do not create `nodalix-color` or `nodalix-gpu` by lifting code from Forja. A shared crate is allowed only when the code is written by Nodalix under a permissive license, or is already under MIT or Apache-2.0, and more than one application already needs the same behavior.

Owning an engine does not delete third-party notices.

## Owned engine, per application

| Application | Shipping architecture | Future engine | Upstream inputs | Migration |
|---|---|---|---|---|
| Lienzo | `lienzo-gnome` still links the C++ engine. `engine/photocraft` is PhotoCraft `909efc0f`, and its desktop binary is named `lienzo`. | GTK 4 / libadwaita calls the Rust engine after the PSD corpus matches. Do not delete `src/core` until then. | Patchy for the corpus that already passes. PhotoCraft for the Rust engine now in the tree. | Compare before swapping the GTK shell onto the Rust compositor. |
| Pluma | VectorCraft `571881a` in [danielmigueltejedor/pluma](https://github.com/danielmigueltejedor/pluma). Desktop binary `pluma`. Shell is still the imported egui UI. | GTK 4 / libadwaita calls this engine. The engine crate does not take GTK types. | VectorCraft. Graphite `68c3c759` is the previous history, not a second engine. | Later VectorCraft commits are compared before they replace files. |
| Forja | Open CAD Studio fork. Iced, not GTK. Package name `OpenCADStudio`. | `forja` engine with no GTK types. UI becomes GTK 4 / libadwaita / Wayland, in the Forja repository. | Open CAD Studio and its kernel/codec/graph pins. | 2D first. Do not block 3D, solids, or constraints by baking Iced or ribbon widgets into the engine. |
| Toma | FilmCraft `771b614` in [danielmigueltejedor/toma](https://github.com/danielmigueltejedor/toma). Desktop binary `toma`. Shell is the imported egui UI. | GTK 4 / libadwaita, engine free of GTK. | FilmCraft | Compare later commits. Do not invent a second timeline. |
| Luz | LightCraft `265248c` in [danielmigueltejedor/luz](https://github.com/danielmigueltejedor/luz). Desktop binary `luz`. Shell is the imported egui UI. | GTK 4 / libadwaita, engine free of GTK. | LightCraft | Compare later commits. |
| Papel | PrintCraft `55c5817` in [danielmigueltejedor/papel](https://github.com/danielmigueltejedor/papel). Desktop binary `papel`. Shell is the imported egui UI. | GTK 4 / libadwaita, engine free of GTK. | PrintCraft | PDF documents. Not page layout. |
| Pulso | EffectCraft `2fadd14` in [danielmigueltejedor/pulso](https://github.com/danielmigueltejedor/pulso). Desktop binary `pulso`. Shell is the imported egui UI. | GTK 4 / libadwaita, engine free of GTK. | EffectCraft | Motion graphics. Not the Toma timeline until a shared crate is justified. |
| Pliego | DesignCraft `d582277` in [danielmigueltejedor/pliego](https://github.com/danielmigueltejedor/pliego). Desktop binary `pliego`. Shell is the imported egui UI. | GTK 4 / libadwaita, engine free of GTK. | DesignCraft | Editorial layout. Not Papel. |
| Vinilo | Rust GNOME player | stays an application, not a Craft engine | none required | Do not fold it into a graphics engine. |

## How an upstream commit enters

Never `git reset --hard` onto upstream, and never replace an engine directory with a tarball. The sequence is detect, compare, classify, test, benchmark, adapt, integrate, validate.

Classes: import directly, adapt, already implemented, Nodalix implementation is better, merge both, not relevant, breaking and needs investigation. When both sides implement the same behavior, the recorded result is keep Nodalix, take upstream, hybrid, or reimplement. The comparison is correctness, visual fidelity, format compatibility, p50/p95/p99 where a benchmark exists, CPU, GPU, RAM, VRAM, allocations, startup, open, save, export, interaction latency, maintainability, and test coverage. A missing benchmark means the decision stays open. It does not mean upstream wins.

Tracking fields for each upstream, stored in that application's `docs/upstream.md`: last observed commit, last evaluated commit, last integrated commit, skipped, adapted, conflicting. Lienzo's copy is [upstream.md](upstream.md).

Automation may fetch, classify, run tests, and open a pull request. It may not merge an engine change by itself. Pull request titles look like `upstream(lienzo): evaluate PhotoCraft abc..def` and list the commits, files, direct imports, adaptations, skips, tests, and benchmarks. Generic fixes can be offered back upstream. Nodalix does not wait for that offer to land.

## Benchmarks and capability matrix

Lienzo's corpus is the existing core tests, the PSD compatibility checks, and Testy. Those numbers stay in their own docs. This file does not invent a score. PhotoCraft is not in that corpus yet. A compositor comparison is added only as a reproducible bench that runs both implementations on the same fixture and records the metrics above.

Each application gets `docs/engine-capability-matrix.md` when its repository exists. Lienzo's matrix is [engine-capability-matrix.md](engine-capability-matrix.md).

Forja viewport targets, when that work starts in the Forja repository: pan and zoom with partial redraw, then timed draws at 10k, 100k, and 1M entities. Snapping (endpoint, midpoint, center, intersection, perpendicular, tangent, nearest, grid, extension, parallel, quadrant) is a separate subsystem. The spatial index is chosen by that benchmark, not in advance. Commands such as LINE, PLINE, CIRCLE, ARC, MOVE, COPY, ROTATE, SCALE, TRIM, EXTEND, OFFSET, FILLET, CHAMFER, MIRROR, and ARRAY stay available from a command line and from GTK actions. The GTK UI must be usable without typing those names.

DWG and DXF need a versioned corpus: open, render, edit, save, round trip, reopen, and preservation of entities, layers, blocks, text, dimensions, and hatches. Comparison with LibreCAD or FreeCAD is allowed. A proprietary application is used only within its license.

## Shared work and interchange

Do not add a shared crate until two applications contain the same code and the license wall allows the move. Candidates, none of which exist yet: color, GPU scheduling, fonts, codecs, document identity, automation, and file IO. CAD geometry does not share a path type with Pluma or Pliego just because both have Bezier curves. Precision and entity identity differ.

Interchange is by existing files first: SVG, PDF, and DXF between Pluma and Forja; PSD and placed files between Lienzo, Pluma, and Pliego; RAW and rendered images between Luz and Lienzo; rendered media between Pulso and Toma; PDF between Forja and Papel. Drag and drop comes after those files round-trip. There is no suite container format.

Color is consistent where the documents share a space: sRGB, linear RGB, Display P3, Rec.709, Rec.2020, HDR, ICC, CMYK, Lab, render intents, and soft proof. Forja does not have to host a photo develop pipeline to open a drawing.

GPU backends for new engines are wgpu, with Vulkan, Metal, and DX12 behind it, and WebGPU only for an application that already has a web target. A CPU path remains where tests or determinism need it. Lienzo's CPU compositor stays the reference until a GPU path matches it.

New engine code is Rust. C++ stays while its tests are the ones that pass. A rewrite that only changes the language is not a migration.

UI for Linux applications is GTK 4, libadwaita, and Wayland. The engine crate does not depend on GTK, Qt, Iced, or egui. Forja's Iced shell is upstream UI. Replacing it is Forja work, and it does not start by reskinning ribbons.

## CI and security

Lienzo's C++ gate stays `ninja -C build/linux-dev lienzo_gnome` on this checkout. `engine/lienzo-engine` is `cargo test -p lienzo-engine` from `engine/`. The workflow `.github/workflows/lienzo-engine.yml` runs that test. It does not merge upstream.

Parsers for PSD, PDF, SVG, DWG, DXF, fonts, images, and video are hostile input. Fuzz those parsers when the implementation is ours. `unsafe` in new Rust is an explicit decision. PhotoCraft forbids `unsafe_code` in its workspace. Lienzo can keep that lint in its own crate without importing PhotoCraft.

## Roadmap

1. This audit and the Lienzo tracking files. Done in this change.
2. Lienzo engine standard: provenance crate, tests, no pixel port yet. Done in `engine/lienzo-engine`.
3. Move one Lienzo subsystem into that crate only after the existing corpus passes. Not started.
4. Pluma engine is VectorCraft `571881a`, binary `pluma`. GTK shell is not wired.
5. Forja: split engine from Iced, then a libadwaita shell, in the Forja repository. Not started. Forja is behind upstream and still builds as Open CAD Studio.
6. Toma, Luz, Papel, Pulso, and Pliego exist on GitHub with the imported engines and product binary names. GTK shells are not wired. `cargo test` of those workspaces was not run for this import.
7. Extract a shared crate only after duplicated permissive code exists.
8. File interchange between applications after the engines can round-trip the formats above.

`main` of each application stays buildable. Legacy and the new engine live side by side until parity, tests, format compatibility, and performance say the legacy path can go. Lienzo's C++ engine is that legacy path, and it is still the one `lienzo-gnome` runs.
