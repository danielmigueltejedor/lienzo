# Lienzo app icon

**Figure:** Diego Velázquez. The painter, for an image editor.

**Style:** a black-and-white portrait cut out over a full-bleed colour field, head and shoulders, in
the Nodalia icon family (one historical figure per app, one colour per app).

**Tile:** 512 × 512 viewBox, rounded square `rx=112`, clipped; no border or roundel.

**Provenance:** the portrait is "Velázquez Self-portrait.jpg" from
[Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Velázquez_Self-portrait.jpg) (public domain). The figure was separated from its
background, converted to black and white and placed over the app colour.
Licence: see [LICENSE.txt](LICENSE.txt).

**Linux:** the hicolor icons (`hicolor/scalable` and every PNG size) put the tile on the Nodalix
app grid, like the Colloid icons around it: a 224 px tile inside a 256 px canvas (16 px margin),
a 51.5 px corner radius and a soft shadow. The scalable SVG is made by `tools/nodalia-app-icon.py`
in [nodalix-icon-theme](https://github.com/danielmigueltejedor/nodalix-icon-theme) and
`packaging/icons.sh` renders the PNG sizes from it. macOS, the Windows `.ico` and the
1024 px render keep the full-bleed tile. The window and taskbar icon on Windows is the 256 px
hicolor PNG, so it has the Linux margin and shadow.

## Files

- `photocraft.svg`: canonical master (traced at 2048 px).
- `photocraft-small.svg`: lighter trace (traced at 1024 px).
- `photocraft-1024.png`: 1024 px render on the macOS grid.
- `photocraft.icns`: macOS bundle icon (`CFBundleIconFile`).
- `photocraft.ico`: Windows icon, 16–256 px, embedded in the `.exe` by `apps/photocraft/build.rs`.
- `hicolor/<size>/apps/ai.storyteller.photocraft.png` (16–512) and `hicolor/scalable/...svg`:
  Linux icon theme.

The app also sets the window icon and Wayland app ID at runtime (`apps/photocraft/src/app_icon.rs`,
`main.rs`). On Windows the taskbar shows the icon of the Start Menu shortcut that launches the
exe, so the MSI's `<Icon Id>` keeps the `.exe` extension (ICE50); `app_icon.rs` documents the
details and tests the `.ico` sizes and the WiX icon references.

## Regenerate

The icon is a raster portrait, so the SVG files embed an image instead of traced paths. The
derived files in this directory were rendered directly from the portrait. `packaging/icons.sh`
still works on `photocraft.svg` (resvg renders embedded images), but its output will differ slightly
from the committed files in resampling.
