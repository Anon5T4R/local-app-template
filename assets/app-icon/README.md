# App icon

**This is a placeholder.** A grey tile with a question mark, so a app born from this template is
never mistaken for a finished one. Replace it before the first release.

## Files

| File | What it is |
|---|---|
| `localapp.svg` | the master: the full-bleed 512 px tile (corner radius 100) Windows and Linux use as is |
| `localapp-1024.png` | the SVG at 1024 px, for stores, README and the issue tracker |
| `hicolor/<size>x<size>/apps/com.localapp.app.png` | the freedesktop icon theme tree (16 to 512 px) and the runtime window icon (256 px) |
| `hicolor/scalable/apps/com.localapp.app.svg` | the scalable theme entry (a copy of the master) |
| `localapp.ico` | the Windows icon: 16, 20, 24, 32, 40, 48, 64, 128 and 256 px, packed by `cargo xtask ico` |

Every one of them is generated from the SVG by `packaging/icons.sh`; the outputs are committed, so
building and packaging never need `resvg`.

## Replacing it (a born app)

1. Draw the app's icon as a 512 px square SVG, full bleed, corner radius 100 - the suite's frame.
   Windows and Linux show it as is, so the artwork must read at 16-48 px.
2. Point `packaging/icons.sh` at the new SVG and rename the files it writes (the script names the
   app icon after the app id: `com.<yourapp>.app`).
3. Run `packaging/icons.sh` (needs `resvg`), then `cargo xtask assets` and commit the outputs with
   this README, `assets/ATTRIBUTION.md` and `packaging/icons.sh` in the same commit.
4. QA it the way the suite does: screenshot of the window and the taskbar at 100 % and 200 %
   scaling, the Start menu, the file manager. (The placeholder is exempt: no QA round wasted on it.)
