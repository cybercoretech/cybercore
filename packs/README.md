# 📦 Cybercore Resource Packs

Resource packs are immutable, reviewable collections consumed by the Cybercore
Control Plane. Each pack directory contains a `manifest.json`; paths,
categories, tags, provenance, compatibility, accessibility, performance
budgets, previews, and SHA-256 checksums are part of the publish contract.

The initial packs stay in this monorepo so crate and pack changes can be tested
together. A pack can move to its own repository later when it needs independent
ownership, release cadence, or asset storage.

```text
packs/
  cybercore-effects/
  cybercore-images/
  cybercore-fonts/
  cybercore-svgs/
  cybercore-snippets/
  cybercore-components/
  cybercore-themes/
```

`packs.json` lists the eight official packs and their version floors.
`tools/export-distribution.py packs <cybercore-packs checkout>` turns these
sources (plus `schema/`, `web/`, `css/` and `assets/`) into the published
`cybercore.*` packs; see `docs/distribution.md`. Hand-written sources:

| Folder | Pack |
| --- | --- |
| `cybercore-effects/` | standalone effects for `cybercore.effects` |
| `cybercore-images/cyberdark-wallpapers/` | `cybercore.cyberdark-wallpapers` |
| `cybercore-snippets/` | `cybercore.snippets` (index: `snippets.json`) |
| `cybercore-fonts/` | `cybercore.fonts` (index: `fonts.json`) |
| `cybercore-web-templates/` | `cybercore.web-templates` |

Themes, components and SVG assets are generated from `schema/`, `css/` +
`web/chooser.*` and `assets/`; their folders here only hold a README.
