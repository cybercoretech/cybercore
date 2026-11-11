# Cybercore Brand Kit

The shared Cybercore family mark is a segmented hexagonal **C** with a cyan inner circuit, an acid-green core bar, and a magenta signal point. It is designed to read clearly at favicon scale and as a full wordmark.

## Files

- `logos/symbol.svg` — standalone square mark; use for app icons, avatars, and favicons.
- `logos/horizontal.svg` — primary dark-surface wordmark with descriptor.
- `logos/horizontal-mono.svg` — single-color lockup for print and low-color surfaces.
- `logos/stacked.svg` — centered vertical lockup for square layouts.
- `logos/header.svg`, `logos/wordmark.svg`, `logos/mark.svg` — header lockup, wordmark, and compact mark.
- `marks/` — family marks: core, security, automation, terminal.
- `icons/` — `symbol-<size>.svg` masters and PNG exports at 16, 32, 48, 64, 96, 128, 192, 256, 384, and 512 px.
- `projects/` and `projects.json` — one icon per project, with title, optional tile, and public upstream link.
- `tiles/`, `headers/`, `cards/` — project tiles, section headers, and card/badge art.
- `identity/` — raster identity marks.
- `avatars/` — `panel`, `round`, and `mono` avatars as SVG plus 64, 128, 256, and 512 px PNGs.

## Color and placement

Use the primary lockup on dark backgrounds (`#080D14` to `#101827`). Keep clear space equal to the height of the green core bar. Do not stretch, rotate, recolor individual pieces, or add effects to the mark. Use the monochrome lockup where color reproduction is unavailable. The mark is shared by Cybercore Tech and its projects; product names may appear beside it, but should not replace the CYBERCORE wordmark.

SVG files are the source of truth. PNGs are convenience exports for software that does not accept SVG. The `logos/` copies are the preferred assets for websites; the `icons/` files are sized for app and browser UI.

Theme-aware copies recolour by role through `--cc-brand-*` variables; never recolour individual pieces by hand.
