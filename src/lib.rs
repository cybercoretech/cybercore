//! cybercore — the shared definition repository for the Cybercore
//! Systems Framework. The single source of truth for the CYBERGRID
//! palette, canonical filesystem paths, and (over time) other schema
//! shared across cyberdeck, cyberdock, cyberterm, cyberplug,
//! omniscient, and whatever comes next.
//!
//! The actual data lives under `schema/`, not in this source code — this
//! crate is a typed, compile-time-embedded loader, not the data's owner.
//! `schema/cybergrid.json` holds metadata (schema version, active theme,
//! paths); each theme's 11-color palette is its own file under
//! `schema/themes/<family>/<slug>.json`. `build.rs` merges all of it into
//! one embedded blob at compile time — edit the JSON files, add or remove
//! theme files freely; everything else follows, no Rust code to touch.

pub mod components;
pub mod control_plane;
pub mod pack;
pub mod palette;
pub mod paths;
pub mod schema;
#[cfg(feature = "status")]
pub mod status;
pub mod theme;
pub mod tokens;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod quality_tests;
