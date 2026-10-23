<p align="center">
  <img src="https://raw.githubusercontent.com/cybercoretech/cybercore/feat/wasm-theme-core/crates/cybercore-components/assets/cybercore-components-card.svg" alt="Cybercore Components — theme-driven shared UI contracts">
</p>

<p align="center">
  <a href="https://crates.io/crates/cybercore-components"><img src="https://img.shields.io/crates/v/cybercore-components.svg" alt="crates.io"></a>
  <a href="https://docs.rs/cybercore-components"><img src="https://img.shields.io/docsrs/cybercore-components" alt="docs.rs"></a>
</p>

# cybercore-components

Shared, governed component styles for Cybercore web, WASM, and desktop
consumers. The crate packages the reusable CSS contract for cards, badges,
tables, controls, empty states, rendered Markdown, modal viewers, and the
theme-picker surface.

Components remain theme-driven: consumers provide the CYBERGRID variables and
the same components repaint without copying application-specific colors.

## Install

```toml
[dependencies]
cybercore-components = "0.1"
```

## Serve the component stylesheet

```rust
use cybercore_components::styles;

let css = styles();
assert!(css.contains(".cc-card"));
```

`COMPONENT_CSS` is also public for consumers that assemble a larger Web Kit:

```rust
use cybercore_components::COMPONENT_CSS;

let document = format!("<style>{COMPONENT_CSS}</style>");
```

## Load order

The component contract expects the following order:

```text
reset → cybertokens.css → theme colors → cybercomponents.css
```

The stylesheet reads semantic variables such as `--bg`, `--fg`, `--panel`,
`--line`, `--cyan`, `--pink`, and the shared typography/spacing tokens. It does
not own a palette, so a site can switch themes without changing component
markup.

## Design guarantees

- Keyboard-safe focus states
- Theme-driven colors and spacing
- Reduced-motion-compatible transitions
- No JavaScript dependency
- Rust 1.85+
- MIT licensed

## Links

- [Component source](https://github.com/cybercoretech/cybercore/tree/feat/wasm-theme-core/css)
- [API documentation](https://docs.rs/cybercore-components)
- [Cybercore Web Kit](https://github.com/cybercoretech/cybercore/tree/feat/wasm-theme-core/web)
