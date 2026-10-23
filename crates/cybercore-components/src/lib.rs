//! Shared component resources for Cybercore consumers.

pub const COMPONENT_CSS: &str = include_str!("../assets/cybercomponents.css");

pub fn styles() -> &'static str {
    COMPONENT_CSS
}
