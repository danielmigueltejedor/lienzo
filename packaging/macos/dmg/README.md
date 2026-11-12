# macOS DMG window

What Finder shows when the DMG opens: a 660 × 400 pt background with the app icon and the
`Applications` link side by side. `package.sh` copies these files into the image; nothing here is
generated at build time, so the DMG still builds with `hdiutil makehybrid` (no mounted device, no
Finder scripting on CI, nothing extra installed in the signing job).

| File | What |
|---|---|
| `background.png` | Source of the background, 1320 × 800 px (the 2x size of the 660 × 400 pt window), designed for Lienzo in the Nodalia Design Language: a rose card with the app mark, a white card with the name, the arrow and the install hint. Original work. |
| `background.tiff` | The background at 1x (660 × 400 px, 72 dpi, scaled down from the PNG) and 2x (1320 × 800 px, 144 dpi) in one HiDPI TIFF (48-colour palette, Deflate, sRGB; about 30 KB). Goes to `.background/background.tiff`. |
| `dmg-layout.DS_Store` | Finder's view settings for the volume: window size, icon size 112, Lienzo.app at (307, 205), `Applications` at (587, 205), either side of the arrow, and the background. Goes to `.DS_Store` in the image; named so it isn't mistaken for (or ignored like) a Finder-generated `.DS_Store`. |
| `generate.py` | Writes `background.tiff` and `dmg-layout.DS_Store` from the PNG and the layout above. |

## The volume name has no version

The mounted volume is called `Lienzo`, not `Lienzo <version>`. The layout points at the background
through an alias that includes the volume name, and Finder resolves it by that name: with a
versioned name the window keeps its size and icon positions but shows no background (tested).
The DMG file name (`lienzo-<version>-macos-<arch>.dmg`) still carries the version.

## Rules

- **Finder draws the icon labels in black in light and dark mode** when a window has a background,
  so the area under both icons stays light (the white card).
- **Nothing goes inside the icon boxes:** artwork keeps 10 pt clear of each 112 pt icon box and of
  the label strip under it.

## Regenerate

Replace `background.png` (keep it 1320 × 800) or change the layout constants in `generate.py`, then run, on any OS:

```sh
pip install 'pillow>=12' ds_store==1.3.3 mac_alias==2.2.3
python3 packaging/macos/dmg/generate.py
```

Both outputs are reproducible and carry nothing from the machine that ran it: `dmg-layout.DS_Store`
is written from scratch, its background alias holding only the volume name and
`/.background/background.tiff`. The window is 660 × 432 with Finder's 32 pt
title bar; the content area is 660 × 400.
