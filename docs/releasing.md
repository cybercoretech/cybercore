# ✨ Releasing Cybercore

The crate package is `cybercore`; the Theme Studio binary is a local workspace
application and is intentionally not published as a crate.

Before cutting a release, update `Cargo.toml`, `CHANGELOG.md`, and the README
version examples. Run the complete CI checks:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc --all-features
cargo +1.85.0 check --workspace --all-targets --all-features
cargo package --list
cargo package
```

The GitHub Actions release workflow runs on a `v<version>` tag or manual
dispatch. Set the `CARGO_REGISTRY_TOKEN` repository Actions secret to a
crates.io token with permission to publish `cybercore`. The workflow verifies
that the version matches `Cargo.toml`, tests and packages the crate, then
publishes unless that exact version is already on crates.io.

After publishing and updating the five consumer dependency pins, run the
`Consumer compatibility` workflow from the Actions tab. It tests Cyberdeck
Hub, Cyberdesk, DaemonHall, Dockspace, and Cyberterm against their locked
crates.io dependencies. For pre-release checks, use a temporary Cargo patch to
the checkout under review.

For a local release, authenticate with `cargo login`, inspect the package with
`cargo publish --dry-run`, then publish with `cargo publish`. Never put the
token in source control or workflow text.
