# Cybercore Resource Packs

Resource packs are immutable, reviewable collections consumed by the Cybercore
Control Plane. Each pack directory contains a `manifest.json`; paths,
categories, tags, provenance, compatibility, accessibility, performance
budgets, previews, and SHA-256 checksums are part of the publish contract.

The initial packs stay in this monorepo so crate and pack changes can be tested
together. A pack can move to its own repository later when it needs independent
ownership, release cadence, or asset storage.

```text
packs/
  cybercore-effects/
  cybercore-images/
  cybercore-fonts/
  cybercore-svgs/
  cybercore-snippets/
  cybercore-components/
  cybercore-themes/
```

The remaining families are scaffolded and accept the same immutable
`manifest.json` contract:

```text
packs/cybercore-fonts/
packs/cybercore-svgs/
packs/cybercore-snippets/
packs/cybercore-components/
packs/cybercore-themes/
```

Pack families are intentionally kept in this repository until they need
independent ownership, release cadence, or asset storage. Each family can be
published independently without creating another GitHub repository.
