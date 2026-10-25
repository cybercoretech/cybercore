# Architecture and product map

Cybercore uses a shared Rust framework for themes, components, effects, packs,
quality checks, and updater operations. Products consume the shared contracts
while keeping their own runtime and presentation layers.

## Core surfaces

| Surface | Responsibility |
|---|---|
| Core crates | Tokens, components, effects, packs, and validation |
| Theme Studio | Local theme authoring and preview |
| Updater engine | Discovery, planning, installation, repair, and rollback |
| Updater TUI | Keyboard-driven operator dashboard |
| Updater Panel | Loopback Control Panel for fleet operations |
| Public docs | Sanitized reference published from Cybercore-owned source |

The updater manifest defines product consumers and shared synchronization tasks.
The public task pages are generated from that manifest so names and coverage
remain aligned.
