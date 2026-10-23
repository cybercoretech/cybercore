<p align="center">
  <img src="assets/header-neon.svg" width="900" alt="Cybercore — schema, palette, tokens and paths: shared source of truth">
</p>

<p align="center">
  <a href="https://cybercoretech.github.io/cybercore/">Live theme gallery & docs →</a>
</p>

[![crates.io](https://img.shields.io/crates/v/cybercore.svg)](https://crates.io/crates/cybercore)
[![downloads](https://img.shields.io/crates/d/cybercore.svg)](https://crates.io/crates/cybercore)
[![docs.rs](https://img.shields.io/docsrs/cybercore)](https://docs.rs/cybercore)
[![cybercore-core](https://img.shields.io/crates/v/cybercore-core.svg)](https://crates.io/crates/cybercore-core)
[![cybercore-effects](https://img.shields.io/crates/v/cybercore-effects.svg)](https://crates.io/crates/cybercore-effects)
[![cybercore-components](https://img.shields.io/crates/v/cybercore-components.svg)](https://crates.io/crates/cybercore-components)
[![cybercore-publish](https://img.shields.io/crates/v/cybercore-publish.svg)](https://crates.io/crates/cybercore-publish)

The master definition repository and underpinning system engine of the
Cybercore Systems Framework. The single source of truth for global
configuration schemas, the CYBERGRID color palette, and canonical
filesystem paths shared across every downstream project — cyberdeck,
cyberdock, cyberterm, cyberplug, omniscient, and whatever comes next.

## 🛠️ The actual source of truth

**`schema/cybergrid.json`** — not any Rust source file. This is
deliberate: JSON is readable by every language in the framework, not
just Rust. cyberdock's C++/QML can load it directly at runtime with
Qt's `QJsonDocument`; Rust projects get it embedded at compile time
(below) for zero runtime cost and zero drift risk.

Everything else in this repo is a typed loader for that one file, not
an owner of the data. **Edit the JSON. Never hand-edit palette values
or paths anywhere else.**

## ✨ Tool status (feature `status`) — since v0.6.0

Every Cybercore tool can publish one JSON status snapshot for the TUI hub and
the Omarchy HUD to read:

```toml
[dependencies]
cybercore = { version = "0.6", features = ["status"] }
```

```rust
use cybercore::status::{self, Health, ToolStatus};

let snapshot = ToolStatus {
    tool: "sigilward".into(),
    version: env!("CARGO_PKG_VERSION").into(),
    health: Health::Watch,
    summary: "2 files drifted".into(),
    ..ToolStatus::default()
};
status::write(&snapshot)?; // atomic: readers never see partial JSON
```

Files go to `$XDG_RUNTIME_DIR/cybercore/<tool>.json` (falling back to
`~/.local/state/cybercore/status/`). The contract is `schema/status.json`.

## 🎨 Design tokens (fonts, sizing, spacing) — since v0.3.0

Colours are `cybergrid.json`. **Everything else about the feel** — font
families, a type scale, spacing, radii, motion, z-index — lives in
**`css/cybertokens.css`** (with a `schema/tokens.json` mirror for
non-CSS consumers).

```rust
// a ready-to-serve :root {} stylesheet
let css = cybercore::tokens::CSS;
```

Load order in any project: `reset → cybertokens.css → theme colours →
components`. Every Cybercore surface pulls the same `--font-mono`,
`--fs-*`, `--space-*`, `--radius-*` so a new site or tool matches the
rest without copy-paste.

## 📦 Modular crates and resource packs

Cybercore is being extracted incrementally into focused crates while the
original `cybercore` crate remains the compatibility source of truth:

| Crate | Responsibility |
| --- | --- |
| [`cybercore-core`](https://crates.io/crates/cybercore-core) | Shared contracts, themes, validation, paths, and pack metadata |
| [`cybercore-effects`](https://crates.io/crates/cybercore-effects) | Governed effects, fallbacks, and reduced-motion adapters |
| [`cybercore-components`](https://crates.io/crates/cybercore-components) | Shared component styles and behavior contracts |
| [`cybercore-publish`](https://crates.io/crates/cybercore-publish) | Dry-run publish plans and consumer packaging contracts |

Resource packs live in `packs/` and use immutable manifests with compatibility,
provenance, licensing, accessibility, performance, preview, and checksum
metadata. Current packs include Cybercore Effects and CyberDark Wallpapers;
Fonts, SVGs, Snippets, Components, and Themes have governed family scaffolds
ready for their first resources.

Before publishing a crate, run `cargo publish --dry-run -p <crate>`. Publish
dependencies in order: `cybercore-core`, `cybercore-effects` and
`cybercore-components`, then `cybercore-publish`. Publishing requires a
configured crates.io credential and is never performed by a build or publish
plan automatically.

## 🎨 Shared design-system components

The reusable components are served as `cybercore::components::CSS` and cover
window chrome, theme pickers, tables, controls, empty states, rendered
markdown, and modal viewers. See the
[design-system guide](docs/design-system.md) for the load order and class
contract.

The browser distribution lives in `web/` and is generated into the Cybercore
Tech Pages `kit/v1/` directory with `tools/build-web-kit.sh`. It preserves the
legacy `window.SITE` theme contract while adding allowlisted motion profiles,
effect recipes, component recipes, and asset/provenance metadata in
`schema/webkit.json`. Static sites should consume the versioned Web Kit rather
than copying `app.js`, tokens, or component styles into each repository.

## 🎨 CYBERGRID theme engine — since v0.7.0

The `cybercore::theme` module turns the embedded palette collection into a
shared runtime catalog. It keeps the existing eleven color roles for current
apps, adds optional light appearance variants and validated design tokens,
and stores user themes outside the binary so every Cybercore app can use them.

```rust
use cybercore::theme::{Appearance, ThemeCatalog};

let mut themes = ThemeCatalog::load()?;
let active = themes.get(themes.active_id()).expect("active theme");
let css = active.document.to_css(themes.active_appearance());
themes.select("tokyo-night")?;
```

Theme documents are portable JSON: they include a stable id, metadata,
palette, optional `variants.light`, and typography/density/corners/motion
settings. Legacy flat palette JSON remains importable. Custom themes are saved
under `$XDG_CONFIG_HOME/cybercore/themes/` (or `~/.config/cybercore/themes/`);
set `CYBERCORE_CONFIG_DIR` to override the shared config root. The selected
theme and dark/light appearance are recorded in `active-theme.json`.

The Hub exposes the shared catalog through its theme picker. The standalone
creator is the `cybercore-theme-studio` workspace package; run it with
`cargo run -p cybercore-theme-studio` for JSON import/export, contrast
feedback, and shared persistence. The `cybercore-theme` CLI checks contrast,
appearance variants, and portable packs. See
[`docs/theme-engine.md`](docs/theme-engine.md) for the document contract,
storage rules, API, and integration notes. Install the checker with:

```sh
curl -fsSL https://cybercoretech.github.io/install.sh | sh
```

This requires Rust/Cargo and installs only the `cybercore-theme` CLI. For a
direct Cargo install, use `cargo install --locked cybercore --bin
cybercore-theme`. Add `cybercore = "0.8"` to a Rust project to use the library.
Studio provides live previews, per-appearance contrast feedback, stable-ID
editing for custom themes, and **Duplicate as new** to safely derive themes
from built-ins or existing custom themes. Imports remain drafts until saved;
built-in themes cannot be overwritten. The Studio uses the active CYBERGRID
palette for its own interface, groups themes into editable family packs, and
accepts both color-picker and direct hex input for each semantic color.
Theme families can be exported as portable `.cyberpack.json` bundles and
installed together with validation and explicit collision policies. Packs
carry version/license/compatibility metadata and palette previews. Consumer
compatibility automation covers Hub, Cyberdesk, DaemonHall, Dockspace, and
Cyberterm.

## ✨ For Rust projects

Add as a dependency:

```toml
[dependencies]
cybercore = "0.8"
```

Requires Rust 1.85 or newer. For local development against an unpublished
change, use a `[patch.crates-io]` entry instead of committing a `path`
dependency, so the project still builds from a clean checkout.

Then:

```rust
use cybercore::palette;
use cybercore::paths;

println!("{}Hello{}", palette::acid_green(), palette::RESET);
let sysops = paths::sysops_root(); // ~/.sysops, expanded
```

The JSON is embedded into your binary via `include_str!` at compile
time — no runtime file dependency, and the exact schema version your
binary was built against travels with it.

## ✨ For C++/QML projects (cyberdock)

Load `schema/cybergrid.json` directly:

```cpp
QFile file(":/cybercore/schema/cybergrid.json");
file.open(QIODevice::ReadOnly);
QJsonDocument doc = QJsonDocument::fromJson(file.readAll());
QString acidGreen = doc["palette"]["acid_green"].toString();
```

Or in QML, since JSON is just JS object syntax:

```qml
property var cybergrid: JSON.parse(cybergrid_json_string)
color: "#" + cybergrid.palette.acid_green
```

(Wiring the actual file into a Qt resource bundle or reading it from a
known filesystem path is left to cyberdock's own build — this repo
just guarantees the JSON's shape and location.)

## 📐 Schema

```json
{
  "schema_version": 1,
  "palette": {
    "bg": "0e091d",
    "acid_green": "c8e967",
    "hot_pink": "FD3E6A",
    "purple": "9147a8",
    "cyan": "14B9B5",
    "orange": "FF7F41",
    "red": "f93d3b",
    "white": "ffffff"
  },
  "paths": {
    "sysops_root": "~/.sysops",
    "cargo_target_root": "~/.cargo-target",
    "arch_sys_root": "~/.arch-sys",
    "omarchy_config_root": "~/.config/omarchy"
  }
}
```

Hex values have no leading `#` (each consumer prepends what it needs —
a `#` for CSS/QML, nothing for raw RGB parsing). Paths use `~/` for
home-relative locations; `cybercore::paths` expands this for Rust
consumers.

## ✨ Versioning

`schema_version` bumps on any breaking change to the JSON's shape
(renaming or removing a field). Adding a new field is non-breaking —
consumers using an older `cybercore` crate version simply won't see
the new field yet, nothing errors.

Tag releases (`v0.1.0`, `v0.2.0`, ...) so downstream `Cargo.toml`
dependencies can pin to a known-good version rather than tracking
`main` directly.
The crate release workflow and required checks are documented in
[docs/releasing.md](docs/releasing.md).

## ✨ Layout

    schema/cybergrid.json   — the actual source of truth
    src/lib.rs               — module declarations
    src/schema.rs            — embeds + parses the JSON once per process
    src/theme.rs             — versioned themes, shared catalog, validation + CSS
    src/palette.rs           — true-color ANSI helpers + raw hex accessor
    src/paths.rs              — canonical filesystem locations, ~-expanded

## 📐 Adding to the schema

1. Add the field to `schema/cybergrid.json`
2. Add the matching field to the relevant struct in `src/schema.rs`
3. Expose it through `src/palette.rs` or `src/paths.rs` (or a new
   module, if it's a new category of shared data)
4. Every Rust project depending on this crate picks it up on their
   next `cargo update` + rebuild. Non-Rust consumers see it the next
   time they read the JSON.

## 🔐 License

MIT

### 🎨 Browser and in-memory themes

The default `fs` feature enables native theme persistence. Disable default features
to use `ThemeCatalog::builtin()`, `insert_custom`, `set_active`,
`set_active_appearance`, `remove_custom_entry`, and `import_pack_in_memory` without
filesystem access. Existing default-feature consumers keep their current behavior.
The optional `status` feature remains a native API and is independent of `fs`.

The [`wasm/` bindings](wasm/README.md) expose theme validation, quality reports, CSS,
family packs, and collision-aware import to browser apps such as DarkskyForge.
