# ✨ Cybercore Web Kit

The Web Kit is the static browser distribution of Cybercore's shared theme
engine, tokens, components, and effect recipes. Its source lives beside the
Rust theme engine so browser sites and native applications share the same
semantic contract.

## 🧩 Compatibility

The runtime keeps the existing `window.SITE` contract:

```html
<script>
  window.SITE = {
    defaultTheme: "neon-night",
    mashups: []
  };
</script>
<script src="https://cybercoretech.github.io/kit/cybercore.js"></script>
```

Sites may opt into the additive Web Kit contract:

```html
<script>
  window.SITE.web = {
    motion: "full",
    effects: {
      "scanline": { "intensity": 0.25 },
      "grid-glow": { "opacity": 0.35 }
    }
  };
</script>
```

Elements opt into recipes explicitly:

```html
<div data-cc-effect="scanline"></div>
<article data-cc-component="detail-dialog">...</article>
```

Every effect has a static fallback and honors
`prefers-reduced-motion`. Recipe IDs are allowlisted in
`schema/webkit.json`; theme documents and site configuration never execute
arbitrary CSS or JavaScript.

## ✨ Distribution

`tools/build-web-kit.sh` copies the canonical runtime, tokens, components,
effect styles, and Web Kit schema into the Cybercore Tech Pages `kit/`
directory. Consumers use the versioned Pages URL rather than copying
`app.js` into each repository.

The generated directory is intentionally static and dependency-free so it
works on GitHub Pages, local previews, and offline mirrors.
