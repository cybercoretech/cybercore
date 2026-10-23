<p align="center">
  <img src="https://raw.githubusercontent.com/cybercoretech/cybercore/feat/wasm-theme-core/crates/cybercore-effects/assets/cybercore-effects-card.svg" alt="Cybercore Effects — governed motion recipes with fallback and reduced-motion variants">
</p>

<p align="center">
  <a href="https://crates.io/crates/cybercore-effects"><img src="https://img.shields.io/crates/v/cybercore-effects.svg" alt="crates.io"></a>
  <a href="https://docs.rs/cybercore-effects"><img src="https://img.shields.io/docsrs/cybercore-effects" alt="docs.rs"></a>
</p>

# ✨ cybercore-effects

Governed visual effect recipes for Cybercore web, WASM, and desktop surfaces.
Effects are shipped with an explicit normal variant, a safe fallback, and a
reduced-motion path so motion never becomes an accessibility afterthought.

## 🚀 Install

```toml
[dependencies]
cybercore-effects = "0.1"
```

## ✨ HUD scanline effect

The first extracted effect is the reviewed HUD scanline pack:

```rust
use cybercore_effects::hud_scanline_css;

let css = hud_scanline_css(false);
let accessible_css = hud_scanline_css(true);
```

`hud_scanline_css(false)` returns the full effect. Passing `true` returns the
reduced-motion stylesheet, which removes animated transitions while preserving
the readable visual hierarchy.

The crate also exposes the source constants when a consumer needs to assemble
its own stylesheet pipeline:

```rust
use cybercore_effects::{
    HUD_SCANLINE_CSS,
    HUD_SCANLINE_FALLBACK_CSS,
    HUD_SCANLINE_REDUCED_MOTION_CSS,
};
```

## 📐 Effect contract

Every governed effect is expected to provide:

1. A named manifest and provenance record in `packs/`
2. A normal presentation
3. A rendering-safe fallback
4. A reduced-motion variant
5. A consumer-safe export through the crate or Web Kit

The CSS is theme-driven and uses the shared CYBERGRID variables rather than
hard-coded application colors.

## ✨ Guarantees

- Rust 1.85+
- MIT licensed
- No JavaScript runtime or network dependency
- Reduced-motion behavior is explicit and testable
- Safe to embed in Web, WASM, and native resource pipelines

## ✨ Links

- [Effect pack source](https://github.com/cybercoretech/cybercore/tree/feat/wasm-theme-core/packs/cybercore-effects)
- [API documentation](https://docs.rs/cybercore-effects)
- [Cybercore Control Plane](https://cybercoretech.github.io/cybercore/)
