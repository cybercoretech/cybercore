<p align="center">
  <img src="assets/cybercore-publish-card.svg" alt="Cybercore Publish — safe release plans for resource packs and Web Kit consumers">
</p>

<p align="center">
  <a href="https://crates.io/crates/cybercore-publish"><img src="https://img.shields.io/crates/v/cybercore-publish.svg" alt="crates.io"></a>
  <a href="https://docs.rs/cybercore-publish"><img src="https://img.shields.io/docsrs/cybercore-publish" alt="docs.rs"></a>
</p>

# cybercore-publish

Dry-run-first publishing contracts for Cybercore resource packs and Web Kit
consumers. This crate describes what should be released without silently
deploying, merging, or mutating a consumer repository.

## Install

```toml
[dependencies]
cybercore-publish = "0.1"
```

## Create an explicit plan

```rust
use cybercore_publish::PublishPlan;

let plan = PublishPlan::dry_run(
    "cybercore-web-kit",
    vec![
        "theme.dark-futurism-neon-wasteland".into(),
        "effect.hud-scanline".into(),
    ],
);

assert!(plan.dry_run);
println!("{}", serde_json::to_string_pretty(&plan)?);
```

The generated contract is serializable and intentionally small:

```json
{
  "version": "1",
  "target": "cybercore-web-kit",
  "resources": ["theme.dark-futurism-neon-wasteland"],
  "dry_run": true
}
```

## Publishing principles

- Plan generation is not deployment
- Consumer repositories are never changed implicitly
- Resource IDs remain explicit and reviewable
- Release metadata can be inspected before a publish step
- Pack compatibility and provenance stay part of the Control Plane workflow

Use this crate alongside `cybercore-core` for resource discovery and
`cybercore-components` / `cybercore-effects` for the assets being packaged.

## Guarantees

- Rust 1.85+
- MIT licensed
- Deterministic, serde-compatible plan output
- No network access or credentials required to generate a plan
- Safe to use in CI as a review gate before deployment

## Links

- [Publishing guide](https://github.com/cybercoretech/cybercore/blob/feat/wasm-theme-core/docs/releasing.md)
- [API documentation](https://docs.rs/cybercore-publish)
- [Cybercore Control Plane](https://cybercoretech.github.io/cybercore/)
