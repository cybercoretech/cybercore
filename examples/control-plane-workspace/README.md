# 🧭 Control Plane sample workspace

This sample demonstrates the shared `.cybercore/workspace.json` contract used
by Theme Studio, DarkskyForge, and future Control Plane workspaces.

It intentionally contains metadata paths for resources that an editor would
create later. The workspace manifest can still be validated without those
optional resource files:

```sh
cargo test -p cybercore control_plane
cargo run -p cybercore --example inspect_control_plane
```

The sample includes:

- a CyberVault theme
- a DarkskyForge phone asset
- a reusable phone-shell component
- a Web Kit telemetry effect
- a verified install-console snippet
- CyberVault Android and Cyberdeck Hub site profiles
