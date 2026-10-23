//! The Cybercore theme engine: versioned theme documents, validation,
//! contrast reports, built-in and user theme catalogs, selection, and CSS.
//!
//! Existing palette files remain the built-in source of truth. Custom
//! themes use [`ThemeDocument`] and live under the user's Cybercore config
//! directory, so consumers can load them without rebuilding their binary.

use crate::schema::{self, Palette};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
#[cfg(feature = "fs")]
use std::fs::{self, OpenOptions};
#[cfg(feature = "fs")]
use std::io::Write;
#[cfg(feature = "fs")]
use std::path::{Path, PathBuf};
#[cfg(feature = "fs")]
use std::time::{SystemTime, UNIX_EPOCH};

pub const FORMAT_VERSION: u32 = 1;
pub const THEME_PACK_FORMAT_VERSION: u32 = 1;
const BUILTIN_FAMILY: &str = "default";

/// A saved theme, portable as JSON and independent of a specific app.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemeDocument {
    pub format_version: u32,
    pub metadata: ThemeMetadata,
    /// The established CYBERGRID color roles. New design properties are
    /// additive so older Cybercore consumers can still use this palette.
    pub palette: Palette,
    #[serde(default)]
    pub variants: BTreeMap<Appearance, Palette>,
    #[serde(default)]
    pub design: DesignTokens,
}

/// A portable bundle of themes that can be shared and installed together.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemePackDocument {
    pub format_version: u32,
    pub metadata: ThemePackMetadata,
    pub themes: Vec<ThemeDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemePackMetadata {
    /// Stable package identifier used in filenames and import reports.
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    /// Pack release version, independent of the theme document format.
    #[serde(default = "default_pack_release")]
    pub version: String,
    /// SPDX identifier or a human-readable license name.
    #[serde(default)]
    pub license: String,
    /// Optional source or project URL.
    #[serde(default)]
    pub homepage: String,
    /// Cybercore release this pack was checked against.
    #[serde(default = "default_engine_compatibility")]
    pub compatibility: String,
    /// Small palette preview used by library and import screens.
    #[serde(default)]
    pub preview: Vec<String>,
}

/// What to do when an imported pack contains IDs already in the catalog.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackConflictPolicy {
    /// Reject the pack without writing any themes.
    #[default]
    Reject,
    /// Keep existing themes and import only new IDs.
    Skip,
    /// Replace colliding custom themes; built-ins are always protected.
    ReplaceCustom,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PackConflicts {
    pub custom: Vec<String>,
    pub builtin: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PackImportReport {
    pub imported: Vec<String>,
    pub skipped: Vec<String>,
}

impl ThemePackDocument {
    pub fn validate(&self) -> Result<(), ThemeError> {
        if self.format_version != THEME_PACK_FORMAT_VERSION {
            return Err(ThemeError::UnsupportedPackVersion(self.format_version));
        }
        validate_id(&self.metadata.id)?;
        if self.metadata.name.trim().is_empty() || self.metadata.name.len() > 96 {
            return Err(ThemeError::InvalidDocument(
                "pack name must contain 1 to 96 characters".into(),
            ));
        }
        for (label, value) in [
            ("pack description", &self.metadata.description),
            ("pack author", &self.metadata.author),
            ("pack version", &self.metadata.version),
            ("pack license", &self.metadata.license),
            ("pack homepage", &self.metadata.homepage),
            ("Cybercore compatibility", &self.metadata.compatibility),
        ] {
            if value.len() > 256 || value.chars().any(char::is_control) {
                return Err(ThemeError::InvalidDocument(format!(
                    "{label} must be at most 256 characters and contain no control characters"
                )));
            }
        }
        if !self.metadata.homepage.is_empty()
            && !(self.metadata.homepage.starts_with("https://")
                || self.metadata.homepage.starts_with("http://"))
        {
            return Err(ThemeError::InvalidDocument(
                "pack homepage must be an http or https URL".into(),
            ));
        }
        if self.metadata.version.trim().is_empty() || self.metadata.compatibility.trim().is_empty()
        {
            return Err(ThemeError::InvalidDocument(
                "pack version and Cybercore compatibility must not be empty".into(),
            ));
        }
        if !engine_compatibility_supported(&self.metadata.compatibility) {
            return Err(ThemeError::InvalidDocument(format!(
                "theme pack requires Cybercore {}, current version is {}",
                self.metadata.compatibility,
                env!("CARGO_PKG_VERSION")
            )));
        }
        if self.metadata.preview.len() > 5
            || self
                .metadata
                .preview
                .iter()
                .any(|color| !valid_hex_color(color))
        {
            return Err(ThemeError::InvalidDocument(
                "pack preview must contain at most five six-digit hex colors".into(),
            ));
        }
        if self.themes.is_empty() || self.themes.len() > 256 {
            return Err(ThemeError::InvalidDocument(
                "a theme pack must contain 1 to 256 themes".into(),
            ));
        }
        let mut ids = BTreeSet::new();
        for theme in &self.themes {
            theme.validate()?;
            if !ids.insert(&theme.metadata.id) {
                return Err(ThemeError::DuplicateId(theme.metadata.id.clone()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemeMetadata {
    /// Stable, lowercase identifier used by apps and filenames.
    pub id: String,
    pub name: String,
    #[serde(default = "default_family")]
    pub family: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    /// Optional governed wallpaper resources associated with this theme.
    #[serde(default)]
    pub wallpapers: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    #[default]
    Dark,
    Light,
}

impl Appearance {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
}

/// Shared typography, density, corners, and motion settings. These are
/// deliberately enums/allowlisted values so imported themes cannot inject
/// arbitrary CSS into an application's page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DesignTokens {
    pub font_ui: String,
    pub font_mono: String,
    pub font_display: String,
    pub density: Density,
    pub corners: Corners,
    pub motion: Motion,
}

impl Default for DesignTokens {
    fn default() -> Self {
        Self {
            font_ui: "Inter".into(),
            font_mono: "JetBrains Mono".into(),
            font_display: "Oxanium".into(),
            density: Density::Comfortable,
            corners: Corners::Sharp,
            motion: Motion::Full,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Density {
    Compact,
    Comfortable,
    Spacious,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Corners {
    Sharp,
    Soft,
    Rounded,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Motion {
    Full,
    Reduced,
    None,
}

impl ThemeDocument {
    pub fn new(id: impl Into<String>, name: impl Into<String>, palette: Palette) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            metadata: ThemeMetadata {
                id: id.into(),
                name: name.into(),
                family: default_family(),
                description: String::new(),
                author: String::new(),
                wallpapers: Vec::new(),
            },
            palette,
            variants: BTreeMap::new(),
            design: DesignTokens::default(),
        }
    }

    /// Read a current document or a legacy flat 11-color Cybercore palette.
    /// Legacy import needs an ID because the old file shape had none.
    pub fn from_json(input: &str, legacy_id: &str) -> Result<Self, ThemeError> {
        if let Ok(document) = serde_json::from_str::<Self>(input) {
            document.validate()?;
            return Ok(document);
        }
        let palette = serde_json::from_str::<Palette>(input)
            .map_err(|error| ThemeError::InvalidDocument(error.to_string()))?;
        let mut document = Self::new(legacy_id, humanize(legacy_id), palette);
        document.metadata.family = "imported".into();
        document.validate()?;
        Ok(document)
    }

    pub fn validate(&self) -> Result<(), ThemeError> {
        if self.format_version != FORMAT_VERSION {
            return Err(ThemeError::UnsupportedVersion(self.format_version));
        }
        validate_id(&self.metadata.id)?;
        if self.metadata.name.trim().is_empty() || self.metadata.name.len() > 96 {
            return Err(ThemeError::InvalidDocument(
                "name must contain 1 to 96 characters".into(),
            ));
        }
        for (label, value) in [
            ("family", &self.metadata.family),
            ("description", &self.metadata.description),
            ("author", &self.metadata.author),
        ] {
            if value.len() > 256 || value.chars().any(char::is_control) {
                return Err(ThemeError::InvalidDocument(format!(
                    "{label} must be at most 256 characters and contain no control characters"
                )));
            }
        }
        for (mode, palette) in std::iter::once(("default", &self.palette)).chain(
            self.variants
                .iter()
                .map(|(mode, palette)| (mode.as_str(), palette)),
        ) {
            validate_palette(palette, mode)?;
        }
        for (label, font) in [
            ("font_ui", &self.design.font_ui),
            ("font_mono", &self.design.font_mono),
            ("font_display", &self.design.font_display),
        ] {
            if font.is_empty()
                || font.len() > 64
                || !font
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_'))
            {
                return Err(ThemeError::InvalidDocument(format!(
                    "{label} must use letters, numbers, spaces, hyphens, or underscores"
                )));
            }
        }
        Ok(())
    }

    /// Resolve the requested appearance, falling back to the base palette.
    pub fn palette_for(&self, appearance: Appearance) -> &Palette {
        self.variants.get(&appearance).unwrap_or(&self.palette)
    }

    /// Build a contrast report for text and accent colors on common surfaces.
    pub fn contrast_report(&self, appearance: Appearance) -> ContrastReport {
        let palette = self.palette_for(appearance);
        ContrastReport {
            checks: vec![
                ContrastCheck::new("text on background", &palette.white, &palette.bg, 4.5),
                ContrastCheck::new("text on panel", &palette.white, &palette.panel, 4.5),
                ContrastCheck::new("muted text on background", &palette.muted, &palette.bg, 4.5),
                ContrastCheck::new("muted text on panel", &palette.muted, &palette.panel, 4.5),
                ContrastCheck::new(
                    "primary accent on background",
                    &palette.acid_green,
                    &palette.bg,
                    3.0,
                ),
                ContrastCheck::new(
                    "secondary accent on background",
                    &palette.hot_pink,
                    &palette.bg,
                    3.0,
                ),
                ContrastCheck::new(
                    "focus indicator on background",
                    &palette.cyan,
                    &palette.bg,
                    3.0,
                ),
                ContrastCheck::new("border on background", &palette.line, &palette.bg, 3.0),
                ContrastCheck::new("border on panel", &palette.line, &palette.panel, 3.0),
                ContrastCheck::new(
                    "purple accent on background",
                    &palette.purple,
                    &palette.bg,
                    3.0,
                ),
                ContrastCheck::new(
                    "orange accent on background",
                    &palette.orange,
                    &palette.bg,
                    3.0,
                ),
                ContrastCheck::new("red accent on background", &palette.red, &palette.bg, 3.0),
            ],
        }
    }

    /// Run contrast checks for both modes and report whether the light palette
    /// was explicitly designed instead of using the compatibility fallback.
    pub fn quality_report(&self) -> ThemeQualityReport {
        let checks = [Appearance::Dark, Appearance::Light]
            .into_iter()
            .flat_map(|appearance| {
                self.contrast_report(appearance)
                    .checks
                    .into_iter()
                    .map(move |check| AppearanceContrastCheck { appearance, check })
            })
            .collect();
        ThemeQualityReport {
            has_explicit_light_variant: self.variants.contains_key(&Appearance::Light),
            checks,
        }
    }

    /// CSS variables understood by the existing Cybercore apps plus shared
    /// typography, density, corner, and motion tokens.
    pub fn to_css(&self, appearance: Appearance) -> String {
        let p = self.palette_for(appearance);
        let radius = match self.design.corners {
            Corners::Sharp => "2px",
            Corners::Soft => "8px",
            Corners::Rounded => "14px",
        };
        let spacing = match self.design.density {
            Density::Compact => "0.85",
            Density::Comfortable => "1",
            Density::Spacious => "1.15",
        };
        let duration = match self.design.motion {
            Motion::Full => "0.24s",
            Motion::Reduced => "0.08s",
            Motion::None => "0s",
        };
        format!(
            ":root{{--bg:#{bg};--fg:#{fg};--acid:#{acid};--pink:#{pink};--purple:#{purple};--cyan:#{cyan};--orange:#{orange};--red:#{red};--panel:#{panel};--line:#{line};--muted:#{muted};--font-ui:\"{font_ui}\",sans-serif;--font-mono:\"{font_mono}\",monospace;--font-display:\"{font_display}\",sans-serif;--radius:{radius};--density:{spacing};--dur-mid:{duration};--reduced-motion:{reduced};}}\n",
            bg = p.bg.trim_start_matches('#'),
            fg = p.white.trim_start_matches('#'),
            acid = p.acid_green.trim_start_matches('#'),
            pink = p.hot_pink.trim_start_matches('#'),
            purple = p.purple.trim_start_matches('#'),
            cyan = p.cyan.trim_start_matches('#'),
            orange = p.orange.trim_start_matches('#'),
            red = p.red.trim_start_matches('#'),
            panel = p.panel.trim_start_matches('#'),
            line = p.line.trim_start_matches('#'),
            muted = p.muted.trim_start_matches('#'),
            font_ui = self.design.font_ui,
            font_mono = self.design.font_mono,
            font_display = self.design.font_display,
            reduced = if self.design.motion == Motion::Full { "0" } else { "1" },
        )
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ContrastCheck {
    pub label: &'static str,
    pub ratio: f64,
    pub minimum: f64,
    pub passes: bool,
}

impl ContrastCheck {
    fn new(label: &'static str, foreground: &str, background: &str, minimum: f64) -> Self {
        let ratio = contrast_ratio(foreground, background).unwrap_or(0.0);
        Self {
            label,
            ratio,
            minimum,
            passes: ratio >= minimum,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ContrastReport {
    pub checks: Vec<ContrastCheck>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AppearanceContrastCheck {
    pub appearance: Appearance,
    #[serde(flatten)]
    pub check: ContrastCheck,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ThemeQualityReport {
    pub has_explicit_light_variant: bool,
    pub checks: Vec<AppearanceContrastCheck>,
}

impl ThemeQualityReport {
    pub fn failures(&self) -> impl Iterator<Item = &AppearanceContrastCheck> {
        self.checks.iter().filter(|check| !check.check.passes)
    }
}

impl ContrastReport {
    pub fn passes(&self) -> bool {
        self.checks.iter().all(|check| check.passes)
    }
}

pub fn contrast_ratio(foreground: &str, background: &str) -> Option<f64> {
    let luminance = |hex: &str| -> Option<f64> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let channel = |start| {
            let byte = u8::from_str_radix(&hex[start..start + 2], 16).ok()? as f64 / 255.0;
            Some(if byte <= 0.04045 {
                byte / 12.92
            } else {
                ((byte + 0.055) / 1.055).powf(2.4)
            })
        };
        Some(0.2126 * channel(0)? + 0.7152 * channel(2)? + 0.0722 * channel(4)?)
    };
    let (fg, bg) = (luminance(foreground)?, luminance(background)?);
    let (lighter, darker) = if fg >= bg { (fg, bg) } else { (bg, fg) };
    Some((lighter + 0.05) / (darker + 0.05))
}

#[derive(Debug, Clone)]
pub struct ThemeEntry {
    pub document: ThemeDocument,
    pub builtin: bool,
}

/// Theme index with embedded built-ins and optional custom documents.
/// In-memory operations never read environment variables or access the filesystem.
#[derive(Debug, Clone)]
pub struct ThemeCatalog {
    entries: BTreeMap<String, ThemeEntry>,
    active: String,
    appearance: Appearance,
}

impl ThemeCatalog {
    /// Create a catalog from embedded themes, without accessing disk or environment.
    pub fn builtin() -> Result<Self, ThemeError> {
        let schema = schema::load();
        let mut entries = BTreeMap::new();
        for (id, palette) in &schema.themes {
            let mut document = ThemeDocument::new(id.clone(), humanize(id), palette.clone());
            document.metadata.family = schema
                .theme_family(id)
                .unwrap_or(BUILTIN_FAMILY)
                .to_string();
            document.metadata.wallpapers = schema
                .wallpaper_options
                .get(id)
                .cloned()
                .unwrap_or_default();
            document.validate()?;
            entries.insert(
                id.clone(),
                ThemeEntry {
                    document,
                    builtin: true,
                },
            );
        }
        Ok(Self {
            entries,
            active: schema.active.clone(),
            appearance: Appearance::default(),
        })
    }

    /// Load embedded themes, user themes, and the persisted native selection.
    #[cfg(feature = "fs")]
    pub fn load() -> Result<Self, ThemeError> {
        let mut catalog = Self::builtin()?;
        let entries = &mut catalog.entries;
        let root = user_themes_dir()?;
        if root.is_dir() {
            for item in fs::read_dir(&root).map_err(ThemeError::Io)? {
                let item = item.map_err(ThemeError::Io)?;
                let path = item.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    continue;
                }
                let stem = path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                validate_id(stem)?;
                let input = fs::read_to_string(&path).map_err(ThemeError::Io)?;
                let document = ThemeDocument::from_json(&input, stem)?;
                if document.metadata.id != stem {
                    return Err(ThemeError::InvalidDocument(format!(
                        "theme file '{}' contains id '{}'",
                        path.display(),
                        document.metadata.id
                    )));
                }
                if entries.contains_key(stem) {
                    return Err(ThemeError::DuplicateId(stem.into()));
                }
                entries.insert(
                    stem.into(),
                    ThemeEntry {
                        document,
                        builtin: false,
                    },
                );
            }
        }
        let selection = read_active_selection().ok().flatten();
        let preferred = std::env::var("CYBERGRID_THEME")
            .ok()
            .filter(|id| entries.contains_key(id))
            .or_else(|| {
                selection
                    .as_ref()
                    .map(|value| value.id.clone())
                    .filter(|id| entries.contains_key(id))
            })
            .unwrap_or_else(|| catalog.active.clone());
        catalog.active = preferred;
        catalog.appearance = selection.map(|value| value.appearance).unwrap_or_default();
        Ok(catalog)
    }

    pub fn get(&self, id: &str) -> Option<&ThemeEntry> {
        self.entries.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &ThemeEntry)> {
        self.entries.iter().map(|(id, entry)| (id.as_str(), entry))
    }

    pub fn active_id(&self) -> &str {
        &self.active
    }

    pub fn active_appearance(&self) -> Appearance {
        self.appearance
    }

    /// Stable within a process and sensitive to theme documents, selection,
    /// appearance, additions, and removals. Web adapters use this revision to
    /// publish change events without exposing filesystem paths.
    pub fn revision(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.active.hash(&mut hasher);
        self.appearance.as_str().hash(&mut hasher);
        for (id, entry) in &self.entries {
            id.hash(&mut hasher);
            entry.builtin.hash(&mut hasher);
            if let Ok(document) = serde_json::to_vec(&entry.document) {
                document.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Export every built-in and custom theme in a named family as one pack.
    pub fn export_family(&self, family: &str) -> Option<ThemePackDocument> {
        let themes: Vec<_> = self
            .entries
            .values()
            .filter(|entry| entry.document.metadata.family.eq_ignore_ascii_case(family))
            .map(|entry| entry.document.clone())
            .collect();
        if themes.is_empty() {
            return None;
        }
        let pack_name = if family.trim().is_empty() {
            "Uncategorized"
        } else {
            family
        };
        let pack = ThemePackDocument {
            format_version: THEME_PACK_FORMAT_VERSION,
            metadata: ThemePackMetadata {
                id: pack_id(family),
                name: pack_name.to_string(),
                description: format!("Cybercore themes from the {pack_name} family."),
                author: "Cybercore Theme Studio".into(),
                version: "1.0.0".into(),
                license: "MIT".into(),
                homepage: "https://github.com/cybercoretech/cybercore".into(),
                compatibility: "0.8+".into(),
                preview: pack_preview(&themes),
            },
            themes,
        };
        Some(pack)
    }

    /// Report custom and built-in ID collisions without changing the catalog.
    pub fn pack_conflicts(&self, pack: &ThemePackDocument) -> PackConflicts {
        let mut conflicts = PackConflicts {
            custom: Vec::new(),
            builtin: Vec::new(),
        };
        for theme in &pack.themes {
            if let Some(existing) = self.entries.get(&theme.metadata.id) {
                if existing.builtin {
                    conflicts.builtin.push(theme.metadata.id.clone());
                } else {
                    conflicts.custom.push(theme.metadata.id.clone());
                }
            }
        }
        conflicts
    }

    /// Import a validated pack after resolving all ID collisions up front.
    /// Reject policy guarantees no writes occur if any ID conflicts.
    #[cfg(feature = "fs")]
    pub fn import_pack(
        &mut self,
        pack: ThemePackDocument,
        policy: PackConflictPolicy,
    ) -> Result<PackImportReport, ThemeError> {
        self.import_pack_with(pack, policy, Self::save)
    }

    /// Import into memory. Validation and collision checks precede all mutations.
    pub fn import_pack_in_memory(
        &mut self,
        pack: ThemePackDocument,
        policy: PackConflictPolicy,
    ) -> Result<PackImportReport, ThemeError> {
        self.import_pack_with(pack, policy, Self::insert_custom)
    }

    fn import_pack_with(
        &mut self,
        pack: ThemePackDocument,
        policy: PackConflictPolicy,
        insert: fn(&mut Self, ThemeDocument) -> Result<(), ThemeError>,
    ) -> Result<PackImportReport, ThemeError> {
        pack.validate()?;
        let conflicts = self.pack_conflicts(&pack);
        if policy == PackConflictPolicy::Reject
            && (!conflicts.custom.is_empty() || !conflicts.builtin.is_empty())
        {
            return Err(ThemeError::PackConflicts {
                custom: conflicts.custom,
                builtin: conflicts.builtin,
            });
        }
        if policy == PackConflictPolicy::ReplaceCustom && !conflicts.builtin.is_empty() {
            return Err(ThemeError::PackConflicts {
                custom: Vec::new(),
                builtin: conflicts.builtin,
            });
        }

        let mut report = PackImportReport {
            imported: Vec::new(),
            skipped: Vec::new(),
        };
        for theme in pack.themes {
            let id = theme.metadata.id.clone();
            let conflict = self.entries.contains_key(&id);
            if conflict && policy == PackConflictPolicy::Skip {
                report.skipped.push(id);
                continue;
            }
            insert(self, theme)?;
            report.imported.push(id);
        }
        Ok(report)
    }

    /// Save a portable custom theme to the shared user theme folder.
    #[cfg(feature = "fs")]
    pub fn save(&mut self, document: ThemeDocument) -> Result<(), ThemeError> {
        document.validate()?;
        if self
            .entries
            .get(&document.metadata.id)
            .is_some_and(|entry| entry.builtin)
        {
            return Err(ThemeError::ReservedId(document.metadata.id));
        }
        let root = user_themes_dir()?;
        fs::create_dir_all(&root).map_err(ThemeError::Io)?;
        let path = root.join(format!("{}.json", document.metadata.id));
        atomic_write(
            &path,
            &serde_json::to_vec_pretty(&document).map_err(ThemeError::Json)?,
        )?;
        self.insert_custom(document)
    }

    /// Insert or replace a custom theme without persisting it. Built-ins are protected.
    pub fn insert_custom(&mut self, document: ThemeDocument) -> Result<(), ThemeError> {
        document.validate()?;
        if self
            .entries
            .get(&document.metadata.id)
            .is_some_and(|entry| entry.builtin)
        {
            return Err(ThemeError::ReservedId(document.metadata.id));
        }
        self.entries.insert(
            document.metadata.id.clone(),
            ThemeEntry {
                document,
                builtin: false,
            },
        );
        Ok(())
    }

    #[cfg(feature = "fs")]
    pub fn select(&mut self, id: &str) -> Result<(), ThemeError> {
        if !self.entries.contains_key(id) {
            return Err(ThemeError::UnknownTheme(id.into()));
        }
        write_active_selection(id, self.appearance)?;
        self.set_active(id)
    }

    /// Select a theme in memory without persisting it.
    pub fn set_active(&mut self, id: &str) -> Result<(), ThemeError> {
        if !self.entries.contains_key(id) {
            return Err(ThemeError::UnknownTheme(id.into()));
        }
        self.active = id.into();
        Ok(())
    }

    #[cfg(feature = "fs")]
    pub fn set_appearance(&mut self, appearance: Appearance) -> Result<(), ThemeError> {
        write_active_selection(&self.active, appearance)?;
        self.set_active_appearance(appearance);
        Ok(())
    }

    /// Change appearance in memory without persisting it.
    pub fn set_active_appearance(&mut self, appearance: Appearance) {
        self.appearance = appearance;
    }

    #[cfg(feature = "fs")]
    pub fn remove_custom(&mut self, id: &str) -> Result<bool, ThemeError> {
        let Some(entry) = self.entries.get(id) else {
            return Ok(false);
        };
        if entry.builtin {
            return Err(ThemeError::ReservedId(id.into()));
        }
        fs::remove_file(user_themes_dir()?.join(format!("{id}.json"))).map_err(ThemeError::Io)?;
        self.entries.remove(id);
        if self.active == id {
            let fallback = schema::load().active.clone();
            write_active_selection(&fallback, self.appearance)?;
            self.active = fallback;
        }
        Ok(true)
    }

    /// Remove a custom entry in memory, falling back to the embedded default if active.
    pub fn remove_custom_entry(&mut self, id: &str) -> Result<bool, ThemeError> {
        let Some(entry) = self.entries.get(id) else {
            return Ok(false);
        };
        if entry.builtin {
            return Err(ThemeError::ReservedId(id.into()));
        }
        self.entries.remove(id);
        if self.active == id {
            self.active = schema::load().active.clone();
        }
        Ok(true)
    }
}

fn pack_id(family: &str) -> String {
    let mut id = String::new();
    for character in family.to_ascii_lowercase().chars() {
        if character.is_ascii_lowercase() || character.is_ascii_digit() {
            if id.len() < 64 {
                id.push(character);
            }
        } else if !id.is_empty() && !id.ends_with('-') && id.len() < 64 {
            id.push('-');
        }
    }
    let id = id.trim_matches('-');
    if id.is_empty() {
        "theme-pack".into()
    } else {
        id.into()
    }
}

fn validate_id(id: &str) -> Result<(), ThemeError> {
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || id.starts_with('-')
        || id.ends_with('-')
    {
        return Err(ThemeError::InvalidDocument(
            "id must be 1-64 lowercase letters, digits, or hyphens".into(),
        ));
    }
    Ok(())
}

fn validate_palette(palette: &Palette, mode: &str) -> Result<(), ThemeError> {
    for (role, value) in [
        ("bg", &palette.bg),
        ("white", &palette.white),
        ("acid_green", &palette.acid_green),
        ("hot_pink", &palette.hot_pink),
        ("purple", &palette.purple),
        ("cyan", &palette.cyan),
        ("orange", &palette.orange),
        ("red", &palette.red),
        ("panel", &palette.panel),
        ("line", &palette.line),
        ("muted", &palette.muted),
    ] {
        let hex = value.trim_start_matches('#');
        if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ThemeError::InvalidColor {
                mode: mode.into(),
                role: role.into(),
                value: value.clone(),
            });
        }
    }
    Ok(())
}

fn valid_hex_color(value: &str) -> bool {
    let value = value.strip_prefix('#').unwrap_or(value);
    value.len() == 6 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn pack_preview(themes: &[ThemeDocument]) -> Vec<String> {
    themes
        .first()
        .map(|theme| {
            let palette = &theme.palette;
            [
                &palette.bg,
                &palette.acid_green,
                &palette.hot_pink,
                &palette.cyan,
                &palette.purple,
            ]
            .into_iter()
            .map(|color| format!("#{}", color.trim_start_matches('#')))
            .collect()
        })
        .unwrap_or_default()
}

fn default_pack_release() -> String {
    "1.0.0".into()
}

fn default_engine_compatibility() -> String {
    "0.7".into()
}

fn engine_compatibility_supported(requirement: &str) -> bool {
    let minimum = requirement.strip_prefix(">=").unwrap_or(requirement);
    let minimum = minimum.strip_suffix('+').unwrap_or(minimum);
    let mut required = minimum.split('.');
    let (Some(required_major), Some(required_minor), None) =
        (required.next(), required.next(), required.next())
    else {
        return false;
    };
    let (Ok(required_major), Ok(required_minor)) =
        (required_major.parse::<u64>(), required_minor.parse::<u64>())
    else {
        return false;
    };
    let mut current = env!("CARGO_PKG_VERSION").split('.');
    let (Some(current_major), Some(current_minor)) = (current.next(), current.next()) else {
        return false;
    };
    let (Ok(current_major), Ok(current_minor)) =
        (current_major.parse::<u64>(), current_minor.parse::<u64>())
    else {
        return false;
    };
    (required_major, required_minor) <= (current_major, current_minor)
}

#[cfg(feature = "fs")]
fn user_config_dir() -> Result<PathBuf, ThemeError> {
    if let Some(path) = std::env::var_os("CYBERCORE_CONFIG_DIR") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path).join("cybercore"));
    }
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".config/cybercore"))
        .ok_or(ThemeError::MissingConfigHome)
}

#[cfg(feature = "fs")]
fn user_themes_dir() -> Result<PathBuf, ThemeError> {
    Ok(user_config_dir()?.join("themes"))
}

#[cfg(feature = "fs")]
fn active_path() -> Result<PathBuf, ThemeError> {
    Ok(user_config_dir()?.join("active-theme.json"))
}

#[cfg(feature = "fs")]
fn read_active_selection() -> Result<Option<ActiveTheme>, ThemeError> {
    let path = active_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(ThemeError::Io)?;
    let value: ActiveTheme = serde_json::from_str(&raw).map_err(ThemeError::Json)?;
    Ok(Some(value))
}

#[cfg(feature = "fs")]
fn write_active_selection(id: &str, appearance: Appearance) -> Result<(), ThemeError> {
    let path = active_path()?;
    let parent = path.parent().ok_or(ThemeError::MissingConfigHome)?;
    fs::create_dir_all(parent).map_err(ThemeError::Io)?;
    let value = serde_json::to_vec_pretty(&ActiveTheme {
        id: id.into(),
        appearance,
    })
    .map_err(ThemeError::Json)?;
    atomic_write(&path, &value)
}

#[cfg(feature = "fs")]
#[derive(Serialize, Deserialize)]
struct ActiveTheme {
    id: String,
    #[serde(default)]
    appearance: Appearance,
}

#[cfg(feature = "fs")]
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ThemeError> {
    let parent = path.parent().ok_or(ThemeError::MissingConfigHome)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = parent.join(format!(".theme-{}-{nonce}.tmp", std::process::id()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(ThemeError::Io)?;
        file.write_all(bytes).map_err(ThemeError::Io)?;
        file.sync_all().map_err(ThemeError::Io)?;
        fs::rename(&temporary, path).map_err(ThemeError::Io)?;
        if let Ok(directory) = OpenOptions::new().read(true).open(parent) {
            let _ = directory.sync_all();
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn default_family() -> String {
    BUILTIN_FAMILY.into()
}

fn humanize(slug: &str) -> String {
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().chain(chars).collect::<String>())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug)]
pub enum ThemeError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidDocument(String),
    InvalidColor {
        mode: String,
        role: String,
        value: String,
    },
    UnsupportedVersion(u32),
    UnsupportedPackVersion(u32),
    UnknownTheme(String),
    DuplicateId(String),
    ReservedId(String),
    PackConflicts {
        custom: Vec<String>,
        builtin: Vec<String>,
    },
    MissingConfigHome,
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "theme file error: {error}"),
            Self::Json(error) => write!(f, "theme JSON error: {error}"),
            Self::InvalidDocument(message) => write!(f, "invalid theme: {message}"),
            Self::InvalidColor { mode, role, value } => {
                write!(f, "invalid {mode} palette color for {role}: {value:?}")
            }
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported theme format version {version}")
            }
            Self::UnsupportedPackVersion(version) => {
                write!(f, "unsupported theme pack format version {version}")
            }
            Self::UnknownTheme(id) => write!(f, "unknown theme '{id}'"),
            Self::DuplicateId(id) => write!(f, "theme id '{id}' is already in use"),
            Self::ReservedId(id) => write!(f, "cannot replace or remove built-in theme '{id}'"),
            Self::PackConflicts { custom, builtin } => {
                write!(f, "theme pack has conflicting IDs")?;
                if !custom.is_empty() {
                    write!(f, "; custom themes: {}", custom.join(", "))?;
                }
                if !builtin.is_empty() {
                    write!(f, "; protected built-ins: {}", builtin.join(", "))?;
                }
                Ok(())
            }
            Self::MissingConfigHome => {
                write!(f, "set CYBERCORE_CONFIG_DIR, XDG_CONFIG_HOME, or HOME")
            }
        }
    }
}

impl std::error::Error for ThemeError {}

#[cfg(test)]
mod pack_tests {
    use super::{
        Appearance, PackConflictPolicy, ThemeCatalog, ThemeDocument, ThemeEntry, ThemePackDocument,
        ThemePackMetadata, THEME_PACK_FORMAT_VERSION,
    };
    use crate::schema::Palette;
    use std::collections::BTreeMap;

    fn theme(id: &str, family: &str) -> ThemeDocument {
        let palette = Palette {
            bg: "101010".into(),
            white: "eeeeee".into(),
            acid_green: "aadd44".into(),
            hot_pink: "dd4488".into(),
            purple: "8844aa".into(),
            cyan: "44aaaa".into(),
            orange: "dd8844".into(),
            red: "dd4444".into(),
            panel: "202020".into(),
            line: "444444".into(),
            muted: "aaaaaa".into(),
        };
        let mut document = ThemeDocument::new(id, id, palette);
        document.metadata.family = family.into();
        document
    }

    fn pack(themes: Vec<ThemeDocument>) -> ThemePackDocument {
        ThemePackDocument {
            format_version: THEME_PACK_FORMAT_VERSION,
            metadata: ThemePackMetadata {
                id: "test-pack".into(),
                name: "Test Pack".into(),
                description: String::new(),
                author: String::new(),
                version: "1.0.0".into(),
                license: "MIT".into(),
                homepage: String::new(),
                compatibility: "0.8".into(),
                preview: Vec::new(),
            },
            themes,
        }
    }

    fn catalog() -> ThemeCatalog {
        let builtin = theme("core-default", "core");
        let custom = theme("user-custom", "night-ops");
        ThemeCatalog {
            entries: BTreeMap::from([
                (
                    builtin.metadata.id.clone(),
                    ThemeEntry {
                        document: builtin,
                        builtin: true,
                    },
                ),
                (
                    custom.metadata.id.clone(),
                    ThemeEntry {
                        document: custom,
                        builtin: false,
                    },
                ),
            ]),
            active: "core-default".into(),
            appearance: Appearance::Dark,
        }
    }

    #[test]
    fn old_pack_metadata_loads_with_safe_compatibility_defaults() {
        let encoded = serde_json::to_value(pack(vec![theme("one", "test")])).unwrap();
        let mut value = encoded;
        let metadata = value["metadata"].as_object_mut().unwrap();
        metadata.remove("version");
        metadata.remove("license");
        metadata.remove("homepage");
        metadata.remove("compatibility");
        metadata.remove("preview");
        let decoded: ThemePackDocument = serde_json::from_value(value).unwrap();
        assert_eq!(decoded.metadata.version, "1.0.0");
        assert_eq!(decoded.metadata.compatibility, "0.7");
        assert!(decoded.metadata.preview.is_empty());
        decoded.validate().unwrap();
    }

    #[test]
    fn pack_preview_and_homepage_are_validated_before_install() {
        let mut invalid_homepage = pack(vec![theme("one", "test")]);
        invalid_homepage.metadata.homepage = "javascript:alert(1)".into();
        assert!(invalid_homepage.validate().is_err());

        let mut invalid_preview = pack(vec![theme("one", "test")]);
        invalid_preview.metadata.preview = vec!["##ffffff".into()];
        assert!(invalid_preview.validate().is_err());

        let mut future_pack = pack(vec![theme("one", "test")]);
        future_pack.metadata.compatibility = "9.0+".into();
        assert!(future_pack.validate().is_err());
    }

    #[test]
    fn revision_changes_when_active_theme_or_appearance_changes() {
        let mut catalog = catalog();
        let initial = catalog.revision();
        catalog.active = "user-custom".into();
        assert_ne!(catalog.revision(), initial);
        catalog.active = "core-default".into();
        catalog.appearance = Appearance::Light;
        assert_ne!(catalog.revision(), initial);
    }

    #[test]
    fn pack_validates_and_round_trips_json() {
        let pack = pack(vec![
            theme("night-one", "night-ops"),
            theme("night-two", "night-ops"),
        ]);
        pack.validate().unwrap();
        let encoded = serde_json::to_string(&pack).unwrap();
        let decoded: ThemePackDocument = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, pack);
    }

    #[test]
    fn pack_rejects_duplicate_theme_ids() {
        let pack = pack(vec![
            theme("night-one", "night-ops"),
            theme("night-one", "night-ops"),
        ]);
        assert!(pack.validate().is_err());
    }

    #[test]
    fn pack_version_is_validated_independently() {
        let mut pack = pack(vec![theme("night-one", "night-ops")]);
        pack.format_version += 1;
        assert!(matches!(
            pack.validate(),
            Err(super::ThemeError::UnsupportedPackVersion(_))
        ));
    }

    #[test]
    fn reject_policy_reports_conflicts_without_mutating_catalog() {
        let mut catalog = catalog();
        let pack = pack(vec![
            theme("core-default", "replacement"),
            theme("user-custom", "replacement"),
            theme("new-theme", "replacement"),
        ]);
        let conflicts = catalog.pack_conflicts(&pack);
        assert_eq!(conflicts.builtin, vec!["core-default"]);
        assert_eq!(conflicts.custom, vec!["user-custom"]);
        assert!(catalog
            .import_pack_in_memory(pack, PackConflictPolicy::Reject)
            .is_err());
        assert_eq!(catalog.entries.len(), 2);
        assert!(!catalog.entries.contains_key("new-theme"));
    }

    #[test]
    fn skip_policy_keeps_all_existing_entries_untouched() {
        let mut catalog = catalog();
        let pack = pack(vec![
            theme("core-default", "replacement"),
            theme("user-custom", "replacement"),
        ]);
        let report = catalog
            .import_pack_in_memory(pack, PackConflictPolicy::Skip)
            .unwrap();
        assert!(report.imported.is_empty());
        assert_eq!(report.skipped, vec!["core-default", "user-custom"]);
        assert_eq!(catalog.entries.len(), 2);
    }

    #[test]
    fn export_family_collects_themes_as_a_pack() {
        let catalog = catalog();
        let pack = catalog.export_family("night-ops").unwrap();
        assert_eq!(pack.metadata.id, "night-ops");
        assert_eq!(pack.themes.len(), 1);
        assert_eq!(pack.themes[0].metadata.id, "user-custom");
    }
}

#[cfg(test)]
mod memory_tests {
    use super::*;

    #[test]
    fn memory_catalog_protects_builtins_and_recovers_active_selection() {
        let mut catalog = ThemeCatalog::builtin().unwrap();
        let default = catalog.active_id().to_owned();
        let builtin = catalog.get(&default).unwrap().document.clone();
        assert!(catalog.insert_custom(builtin.clone()).is_err());
        assert!(catalog.remove_custom_entry(&default).is_err());
        assert!(catalog.set_active("does-not-exist").is_err());
        let mut custom = builtin;
        custom.metadata.id = "test-custom".into();
        catalog.insert_custom(custom).unwrap();
        catalog.set_active("test-custom").unwrap();
        catalog.set_active_appearance(Appearance::Light);
        assert!(catalog.remove_custom_entry("test-custom").unwrap());
        assert_eq!(catalog.active_id(), default);
        assert_eq!(catalog.active_appearance(), Appearance::Light);
        assert!(!catalog.remove_custom_entry("test-custom").unwrap());
    }

    #[test]
    fn memory_pack_import_is_atomic_on_validation_and_collision_failure() {
        let mut catalog = ThemeCatalog::builtin().unwrap();
        let mut document = catalog.iter().next().unwrap().1.document.clone();
        let family = document.metadata.family.clone();
        let mut pack = catalog.export_family(&family).unwrap();
        document.metadata.id = "new-custom".into();
        pack.themes.insert(0, document.clone());
        let before = catalog.revision();
        assert!(catalog
            .import_pack_in_memory(pack.clone(), PackConflictPolicy::Reject)
            .is_err());
        assert_eq!(catalog.revision(), before);
        assert!(catalog
            .import_pack_in_memory(pack.clone(), PackConflictPolicy::ReplaceCustom)
            .is_err());
        assert_eq!(catalog.revision(), before);
        let report = catalog
            .import_pack_in_memory(pack.clone(), PackConflictPolicy::Skip)
            .unwrap();
        assert_eq!(report.imported, ["new-custom"]);
        pack.themes = vec![document.clone()];
        pack.themes[0].metadata.name = "Updated".into();
        catalog
            .import_pack_in_memory(pack.clone(), PackConflictPolicy::ReplaceCustom)
            .unwrap();
        assert_eq!(
            catalog.get("new-custom").unwrap().document.metadata.name,
            "Updated"
        );
        document.metadata.id = "second-custom".into();
        pack.themes.insert(0, document);
        pack.themes[1].palette.bg = "invalid".into();
        let before = catalog.revision();
        assert!(catalog
            .import_pack_in_memory(pack, PackConflictPolicy::ReplaceCustom)
            .is_err());
        assert_eq!(catalog.revision(), before);
    }
}
