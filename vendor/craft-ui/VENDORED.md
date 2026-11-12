# craft-ui (vendored)

Copy of `crates/ui` from storytold/craft-libs at `1b34eba41e62138fc2438d2734523b302840b45d`
(MIT OR Apache-2.0), used through `[patch]` in the workspace `Cargo.toml`.

The only change: `DockStyle::floating_radius` (and `ChromeStyle::radius`), the corner radius of a
floating window's fill and outline. It is 0 by default, so nothing changes unless an app sets it;
Lienzo sets it for the Nodalia themes so the floating Properties card is round like every other
island. Files touched: `src/docking/chrome.rs`, `src/docking/surface.rs`.

To update: copy the new `crates/ui/src` over this one, re-apply the change above, and move the
revision in the workspace `Cargo.toml`, in this file and in the header of `Cargo.toml`.
