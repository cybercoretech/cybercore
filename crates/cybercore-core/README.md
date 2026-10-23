<p align="center">
  <img src="https://raw.githubusercontent.com/cybercoretech/cybercore/feat/wasm-theme-core/crates/cybercore-core/assets/cybercore-core-card.svg" alt="Cybercore Core — schema, themes, validation, paths and pack contracts">
</p>

<p align="center">
  <a href="https://crates.io/crates/cybercore-core"><img src="https://img.shields.io/crates/v/cybercore-core.svg" alt="crates.io"></a>
  <a href="https://docs.rs/cybercore-core"><img src="https://img.shields.io/docsrs/cybercore-core" alt="docs.rs"></a>
  <a href="https://github.com/cybercoretech/cybercore/actions"><img src="https://img.shields.io/github/actions/workflow/status/cybercoretech/cybercore/release.yml?label=checks" alt="checks"></a>
</p>

# ✨ cybercore-core

The stable extraction surface for the Cybercore Systems Framework. It exposes
the shared contracts that every Cybercore tool needs: themes, palettes, design
tokens, canonical paths, resource manifests, validation, and governed packs.

`cybercore-core` is intentionally compatible with the original `cybercore`
crate. The first release re-exports the tested `cybercore` implementation while
the modular Control Plane surface grows independently.

## 🚀 Install

```toml
[dependencies]
cybercore-core = "0.1"
```

## ✨ What it provides

| Module | Use |
| --- | --- |
| `theme` | Load, validate, select, export, and compose CYBERGRID themes |
| `schema` | Access the embedded shared schema and theme metadata |
| `palette` | Work with the eleven semantic color roles |
| `tokens` | Serve the shared typography, spacing, motion, and component tokens |
| `paths` | Resolve canonical Cybercore filesystem locations |
| `control_plane` | Read and validate workspace manifests and site profiles |
| `pack` | Govern resource-pack metadata, provenance, and compatibility |
| `components` | Access the shared component stylesheet |

```rust
use cybercore_core::{control_plane::ControlPlaneWorkspace, theme::ThemeCatalog};

let workspace = ControlPlaneWorkspace::new("my-surface", "My Surface");
workspace.validate()?;

let themes = ThemeCatalog::builtin()?;
println!("{} built-in themes", themes.len());
```

## 🧭 Control Plane contract

The workspace model is metadata-first. Resources carry an ID, kind, version,
status, path, license, provenance, tags, dependencies, and platform metadata.
Site profiles bind a repository to a theme and a governed resource selection.

```rust
use cybercore_core::control_plane::ControlPlaneWorkspace;

let workspace = ControlPlaneWorkspace::load(".cybercore/workspace.json")?;
if let Some(theme) = workspace.resource("theme.cyberpunk-neon") {
    println!("{} {}", theme.title, theme.version);
}
```

## 🧩 Compatibility and guarantees

- Rust 1.85+
- MIT licensed
- No network access or runtime service required
- Deterministic validation and serialization
- Theme data remains schema-driven rather than duplicated in application code
- Compatible with the legacy `cybercore` crate during the extraction phase

## ✨ Links

- [Cybercore theme gallery and Control Plane](https://cybercoretech.github.io/cybercore/)
- [API documentation](https://docs.rs/cybercore-core)
- [Framework repository](https://github.com/cybercoretech/cybercore)
- [Control Plane schema](https://github.com/cybercoretech/cybercore/blob/feat/wasm-theme-core/schema/control-plane.json)
