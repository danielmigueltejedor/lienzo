# macOS DMG window

What Finder shows when the DMG opens: a 660 x 400 pt background with `Lienzo.app` and the
`Applications` link side by side. `make-dmg.sh` copies these files into the image; nothing here is
generated at build time.

| File | What |
|---|---|
| `background.svg` | Source of the background, drawn by `tools/dmg-background/draw_background.py` in [nodalia-design](https://github.com/getnodalia/nodalia-design): two Nodalia islands on the window colour, the left one in the Lienzo colour (`#EE4F78`) with layers, a brush stroke and a brush, the right one white under the icons. Its text is outlined from Sora (SIL Open Font License 1.1), so rendering it needs no fonts. Original work, same licence as the rest of the repository. |
| `background.tiff` | The background at 1x (660 x 400 px, 72 dpi) and 2x (1320 x 800 px, 144 dpi) in one HiDPI TIFF (16-colour palette, Deflate, sRGB). Goes to `.background/background.tiff`. |
| `dmg-layout.DS_Store` | Finder's view settings for the volume: window size, icon size 128, `Lienzo.app` at (326, 205), `Applications` at (574, 205), and the background. Goes to `.DS_Store` in the image. |
| `generate.py` | Writes `background.tiff` and `dmg-layout.DS_Store` from the SVG and the layout above. |

## The volume name has no version

The mounted volume is called `Lienzo`, not `Lienzo <version>`. The layout points at the background
through an alias that includes the volume name, and Finder resolves it by that name: with a
versioned name the window keeps its size and icon positions but shows no background. The DMG file
name (`Lienzo-<version>.dmg`) still carries the version.

## Rules

- Finder draws the icon labels in black in light and dark mode when a window has a background, so
  the area under both icons stays light (the white island).
- Nothing goes inside the 128 pt icon boxes or the label strip under them.

## Regenerate

```sh
pip install 'pillow>=12' ds_store==1.3.3 mac_alias==2.2.3 fonttools
python3 ../nodalia-design/tools/dmg-background/draw_background.py lienzo packaging/macos/dmg/background.svg
python3 packaging/macos/dmg/generate.py   # needs resvg on PATH
```

Both outputs are byte-for-byte reproducible and carry nothing from the machine that ran it.
