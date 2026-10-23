//! Stable extraction surface for Cybercore's shared contracts.
//!
//! The first extraction keeps the existing `cybercore` implementation as the
//! compatibility source of truth. Future contract changes land here first,
//! then the legacy crate can re-export this surface without breaking users.

pub use cybercore::{
    components, control_plane, pack, palette, paths, schema, theme, tokens, VERSION,
};

#[cfg(feature = "status")]
pub use cybercore::status;

