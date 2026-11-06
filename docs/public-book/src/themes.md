# Themes and assets

Cybercore ships **612 themes**:

- 600 canonical palettes in 50 families (`schema/themes/<family>/<id>.json`)
- 12 signature mashups (surfaces from one palette, neons from another),
  published in the org registry's `data/mashups.json`

Every Cybercore site uses the same chooser
(`https://cybercoretech.github.io/kit/chooser.js`) and reads the registry at
runtime, so a theme added there appears on every site without per-site edits.

Theme synchronization derives from the canonical public registry, preserves
family metadata, and tracks generated outputs so unrelated local themes are
not removed.

Theme quality checks cover schema validity, rendering, contrast, appearance
variants, and shared component compatibility. Theme Studio is the authoring
surface; products consume the resulting public contracts.

Open the [public Cybercore Theme Chooser](../theme-chooser.html) to browse
families, search the catalog, and preview palette swatches in a separate
surface.
