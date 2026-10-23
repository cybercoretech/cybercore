# Changelog

All notable changes to `cybercore` are documented in this file.

## Unreleased

### Added

- Versioned Cybercore Web Kit source and generator for browser theme runtime,
  shared effect recipes, motion profiles, component recipe metadata, asset
  variants, and provenance requirements.
- Static Web Kit output is generated for `kit/v1/`; Pages consumers can share
  the runtime without copying `app.js` between repositories.

### Changed

- Reworked the semantic palettes for 19 visually overlapping themes without
  removing any theme IDs or families. The revised schemes distinguish their
  names and intended moods, and the catalog now has no pairs within the strict
  palette-similarity threshold used by the cleanup check.

## [0.8.1] - 2026-10-18

### Changed

- Refresh crates.io repository and homepage metadata for the `cybercoretech`
  organization; no API or runtime changes.

## [0.8.0] - 2026-10-05

### Added

- `cybercore-theme check` reports text, accent, border, and focus contrast for
  dark and light appearance; `--strict` requires every check to pass and an
  explicit light palette. `validate-pack` checks portable bundles before use.
- Theme-pack metadata now supports pack release, license, homepage, Cybercore
  compatibility, and validated color previews while old format-1 packs remain
  readable.
- Consumer compatibility workflow tests Cyberdeck Hub, Cyberdesk, DaemonHall,
  Dockspace, and Cyberterm on a weekly schedule or by manual dispatch.
- Shared component styles now include action buttons, navigation, cards,
  alerts, form fields, visible keyboard focus, and reduced-motion behavior.
- Web consumers receive shared-catalog change events; Cyberterm refreshes only
  when the catalog revision changes.

## [0.7.0] - 2026-10-04

### Added

- `cybercore::theme`: versioned portable theme documents, validation,
  dark/light palette variants, allowlisted design tokens, contrast reports,
  CSS generation, built-in family metadata, shared storage, and safe theme-pack
  import/export.
- `cybercore-theme-studio` workspace binary: standalone local visual editor
  with library selection, CRUD, dark/light editing, import/export, preview,
  and contrast feedback.
- Theme Studio's curated pack library can copy reviewed built-in families into
  the user's catalog with isolated IDs, leaving built-ins and active selection
  untouched.
- Web consumers sync shared theme changes while running; Cyberterm refreshes
  its catalog while running.
- Release-gate tests validate every embedded theme and assert the shared
  component CSS contract. CI checks the declared Rust 1.85 minimum alongside
  stable.

## [0.6.0] - 2026-10-04

### Added

- CI now checks, lints, and tests every workspace package so Theme Studio is
  included in the standard Actions workflow.
- `status` feature: the Cybercore tool status contract (`schema/status.json`)
  and `cybercore::status`, with `ToolStatus`, `Health`, `Metric`, `Event`,
  `Action` and atomic `write` / `write_in`, plus `read`, `path`, `dir` and
  `validate_tool_name`.
  - Status files live in `$XDG_RUNTIME_DIR/cybercore/<tool>.json`, falling
    back to `$HOME/.local/state/cybercore/status/`. There is no shared `/tmp`
    fallback. On Unix the directory is `0700` and files are `0600`.
  - Each write goes to a unique temporary file opened with `create_new`
    (existing files and symlinks are never followed), is synced, and is
    renamed into place. Tool names are restricted to letters, digits, `-` and
    `_`.
  - Off by default, so cybercore without `status` still depends only on serde.
- `publish_status` example (`--features status`).

### Added

- `components` module and `css/cybercomponents.css`: shared cards, status
  badges, data tables, form inputs, popup viewers and the theme-picker
  dropdown, coloured entirely from the active CYBERGRID theme.
- Brand assets (logos, header art, favicon, social card) under `assets/`.

### Fixed

- Removed a global `main {}` rule from the component CSS that broke the
  layout of every cyberdesk page.

### Changed

- Minimum supported Rust version is now **1.85**. 0.4.x declared 1.70, but
  its dependencies (serde, syn) no longer build on Rust 1.70, so the old value
  was not achievable.
- README: depend on `cybercore = "0.5"` from crates.io instead of a git tag.

## [0.4.1] - 2026-10-02

### Changed

- Metadata patch: repository and homepage links point to
  `github.com/cybercoretech/cybercore`. No code changes from 0.4.0.

Earlier releases are tagged in Git (`v0.1.0` to `v0.4.0`).
