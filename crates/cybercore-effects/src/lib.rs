//! Small, governed effect recipes that can be adapted by Web, WASM, and GUI consumers.

pub const HUD_SCANLINE_CSS: &str = include_str!("../assets/hud-scanline/effect.css");
pub const HUD_SCANLINE_FALLBACK_CSS: &str =
    include_str!("../assets/hud-scanline/fallback.css");
pub const HUD_SCANLINE_REDUCED_MOTION_CSS: &str =
    include_str!("../assets/hud-scanline/reduced-motion.css");

pub fn hud_scanline_css(reduced_motion: bool) -> &'static str {
    if reduced_motion {
        HUD_SCANLINE_REDUCED_MOTION_CSS
    } else {
        HUD_SCANLINE_CSS
    }
}
