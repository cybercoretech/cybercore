# 🎨 Cybercore theme engine for browsers

The core's default `fs` feature preserves native persistence. Browser consumers
use `default-features = false` and the in-memory catalog API. The bindings do not
access disk, the environment, network, or browser storage.

Build with Rust's `wasm32-unknown-unknown` target and wasm-bindgen CLI 0.2.129:

```sh
cargo build --locked -p cybercore-theme-wasm --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir wasm/pkg /path/to/target/wasm32-unknown-unknown/release/cybercore_theme_wasm.wasm
```

Initialize the generated ES module before calling its functions. `builtin_themes`
returns a JSON array of theme documents. `validate_theme` returns a quality report;
`theme_css` returns CSS for `dark` or `light`. `export_family`, `pack_conflicts`, and
`import_pack` take a JSON array of custom documents alongside their other inputs.
Import policies are `reject`, `skip`, and `replace_custom`. Built-ins cannot be
replaced. Imports return `{report, themes}` with the complete new custom collection.
All invalid input throws an error string. Persistence belongs to the consuming app.

Check the core separately with `cargo test -p cybercore --no-default-features`:
workspace-wide native builds may unify the Studio's default `fs` feature.
