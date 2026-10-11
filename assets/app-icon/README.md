# Trowel app icon

**Mark:** a trowel blade (an ivory stroke: the blade's edge rising to its tang, with a round
bolster at the bend) resting on a clay datum line, with two dashed level lines above the datum.

**Palette:**

| Colour | Hex | Used for |
|---|---|---|
| Ivory | `#FAF9F5` | the blade (solid) and the two level lines (dashed, 55 % opacity) |
| Clay | `#D97757` | the datum line |
| Slate | `#141413` | the full-bleed tile |

**Tile:** `viewBox="0 0 512 512"`, a rounded square with `rx=112`, clipped. The macOS files add Apple's
transparent margin (tile = 824/1024 of the canvas); Windows and Linux use the tile edge to edge.

**Provenance:** an original SVG drawn in code by 권 (the project owner) for Trowel: a handful of
paths and one circle, no traced bitmap, no third-party or Adobe material. `vectorcraft.svg` is the
canonical artwork (the file names are kept so packaging needs no change). Licence: see `LICENSE.txt`
(MIT OR Apache-2.0).

## Files

| File | What | Used by |
|---|---|---|
| `vectorcraft.svg` | master vector (about 0.9 KB) | source for everything below |
| `vectorcraft-small.svg` | the same file (the artwork is already small) | `hicolor/scalable` |
| `vectorcraft-1024.png` | full tile, 1024 px | docs, store listings |
| `vectorcraft-macos-512.png` | tile with Apple margin, 512 px | runtime Dock icon (`apps/vectorcraft/src/main.rs`) |
| `vectorcraft.icns` | macOS icon set, 16 to 1024 px; **still the previous dragon render** until `packaging/icons.sh` runs on macOS (`iconutil`) | `cargo xtask bundle` (`CFBundleIconFile`) |
| `vectorcraft.ico` | Windows icon, 16 to 256 px | `apps/vectorcraft/build.rs` (embedded in the `.exe`) |
| `hicolor/<size>/apps/ai.storyteller.vectorcraft.png` | Linux theme icons, 16 to 512 px; the 256 one is also the runtime icon on Windows and Linux, the 128 one the brand mark in the app bar and About box (`crates/ui-egui/src/brand.rs`) | `packaging/linux/ai.storyteller.vectorcraft.desktop` |
| `hicolor/scalable/apps/ai.storyteller.vectorcraft.svg` | Linux scalable icon | as above |

## How it reaches each OS

- **macOS:** the app sets the Dock / app-switcher icon at runtime (`ViewportBuilder::with_icon`); the
  `.app` from `cargo xtask bundle` carries `vectorcraft.icns`.
- **Windows:** `build.rs` embeds `vectorcraft.ico` and VERSIONINFO with `winresource` (taskbar, Start
  menu, Explorer, Alt-Tab); the runtime icon covers the title bar.
- **Linux:** install `packaging/linux/ai.storyteller.vectorcraft.desktop` to `share/applications/` and
  `hicolor/` to `share/icons/`. The app sets its Wayland app ID / X11 class to
  `ai.storyteller.vectorcraft`, so the dock matches the window to the launcher.

## Regenerate

Edit or replace `vectorcraft.svg` (and `vectorcraft-small.svg`), then run `packaging/icons.sh`
(needs `resvg`; `iconutil` on macOS for the `.icns`). It writes every PNG, the `.ico` (via
`cargo xtask ico`) and the `.icns`.
