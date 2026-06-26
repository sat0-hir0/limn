# Limn — Visual assets

App icons for Limn, organised by operating system.

## Contents

```
assets/
├── appicon-macos.svg          ← master vector (Big Sur squircle)
├── appicon-windows.svg        ← master vector (full-bleed square)
├── appicon-linux.svg          ← master vector (circular badge)
└── appicons/
    ├── macos/limn-{16,32,64,128,256,512,1024}.png
    ├── windows/limn-{16,24,32,48,64,128,256}.png
    └── linux/limn-{16,24,32,48,64,128,256,512}.png
```

The mark is the lowercase letter `l` (the first letter of *limn*),
rendered as a single ink-coloured rectangle on a paper-coloured field.
Each operating system gets the crop shape its platform conventions
expect — a squircle for macOS, a full-bleed square for Windows, a
circular badge for Linux — but the inner mark is identical across all
three so the brand reads as one.

| Tone | Hex | Role |
|---|---|---|
| Paper | `#f0f3f4` | Background tile |
| Ink | `#16191c` | The `l` mark |

## How to regenerate

The `.svg` files at the top of `assets/` are the masters. The PNGs
under `assets/appicons/<os>/` are rasterised from those masters at the
sizes each platform asks for. To regenerate, open the SVG in any
vector tool (Inkscape, Affinity Designer, Figma) and export at the
desired pixel size.

## How to use

- **macOS (`.app` bundle)**: combine `assets/appicons/macos/*.png`
  with `iconutil` to produce `Limn.icns`, then drop it into
  `Limn.app/Contents/Resources/`. Packaging is not yet wired up.
- **Windows (`.exe` / MSIX)**: combine `assets/appicons/windows/*.png`
  with `ImageMagick` (`magick convert ... Limn.ico`) or
  `rcedit`. Packaging is not yet wired up.
- **Linux (`.desktop` + hicolor)**: copy `assets/appicons/linux/*.png`
  into `/usr/share/icons/hicolor/<size>/apps/limn.png` at install
  time. SVG can be used directly at `scalable/apps/limn.svg`.
  Packaging is not yet wired up.
- **Inside the running app**: gpui's `WindowOptions` does not currently
  expose an app-icon field; the icon files above are kept ready for when
  that path is available.

## Licence

All assets in this directory are licensed under the same terms as the
rest of Limn — **Apache-2.0** (see [`../LICENSE`](../LICENSE)).

The artwork is composed of plain geometric shapes only; it does not
embed any font binaries, so no font licence (e.g. SIL OFL) carries
through. The brand wordmark (which *does* rely on IBM Plex Sans) is
not included here. If a wordmark is ever added, it would need to
ship in outlined form so the same property holds.
