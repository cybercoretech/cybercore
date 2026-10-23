//! Typed manifests for independently versioned Cybercore resource packs.

use serde::{Deserialize, Serialize};
use std::path::Path;

pub const PACK_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackManifest {
    pub format_version: u32,
    pub id: String,
    pub version: String,
    pub title: String,
    pub kind: PackKind,
    pub description: String,
    pub cybercore: Compatibility,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub license: String,
    pub provenance: PackProvenance,
    pub platforms: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub resources: Vec<PackResource>,
    pub checksums: std::collections::BTreeMap<String, String>,
    pub accessibility: AccessibilityContract,
    pub performance: PerformanceBudget,
    #[serde(default)]
    pub preview: Option<String>,
}

impl PackManifest {
    pub fn validate(&self) -> Result<(), PackError> {
        if self.format_version != PACK_FORMAT_VERSION {
            return Err(PackError::UnsupportedVersion(self.format_version));
        }
        validate_id(&self.id)?;
        for (field, value) in [
            ("version", &self.version),
            ("title", &self.title),
            ("description", &self.description),
            ("license", &self.license),
        ] {
            if value.trim().is_empty() {
                return Err(PackError::MissingField(field));
            }
        }
        self.cybercore.validate()?;
        self.provenance.validate()?;
        if self.categories.is_empty() || self.tags.is_empty() || self.platforms.is_empty() {
            return Err(PackError::MissingClassification);
        }
        if self.resources.is_empty() {
            return Err(PackError::EmptyPack);
        }
        for resource in &self.resources {
            resource.validate()?;
            if !self.checksums.contains_key(&resource.path) {
                return Err(PackError::MissingChecksum(resource.path.clone()));
            }
        }
        self.accessibility.validate()?;
        self.performance.validate()?;
        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, PackError> {
        let raw = std::fs::read_to_string(path).map_err(PackError::Read)?;
        let manifest: Self = serde_json::from_str(&raw).map_err(PackError::Parse)?;
        manifest.validate()?;
        Ok(manifest)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PackKind {
    Effects,
    Images,
    Fonts,
    Svgs,
    Snippets,
    Components,
    Themes,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Compatibility {
    pub minimum: String,
    #[serde(default)]
    pub maximum: Option<String>,
}

impl Compatibility {
    fn validate(&self) -> Result<(), PackError> {
        if self.minimum.trim().is_empty() {
            return Err(PackError::MissingField("cybercore.minimum"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackProvenance {
    pub source: String,
    pub author: String,
    pub license: String,
    pub reviewed: bool,
}

impl PackProvenance {
    fn validate(&self) -> Result<(), PackError> {
        for (field, value) in [
            ("provenance.source", &self.source),
            ("provenance.author", &self.author),
            ("provenance.license", &self.license),
        ] {
            if value.trim().is_empty() {
                return Err(PackError::MissingField(field));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PackResource {
    pub id: String,
    pub path: String,
    pub title: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub license: String,
    pub fallback: Option<String>,
    pub reduced_motion: Option<String>,
}

impl PackResource {
    fn validate(&self) -> Result<(), PackError> {
        validate_id(&self.id)?;
        if self.path.trim().is_empty()
            || Path::new(&self.path).is_absolute()
            || self.path.split('/').any(|part| part == "..")
        {
            return Err(PackError::InvalidPath(self.path.clone()));
        }
        if self.title.trim().is_empty() || self.categories.is_empty() || self.tags.is_empty() {
            return Err(PackError::InvalidResource(self.id.clone()));
        }
        if self.license.trim().is_empty() {
            return Err(PackError::MissingField("resource.license"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AccessibilityContract {
    pub semantic_fallback: String,
    pub reduced_motion: bool,
    pub keyboard_safe: bool,
}

impl AccessibilityContract {
    fn validate(&self) -> Result<(), PackError> {
        if self.semantic_fallback.trim().is_empty() {
            return Err(PackError::MissingField("accessibility.semantic_fallback"));
        }
        if !self.reduced_motion || !self.keyboard_safe {
            return Err(PackError::AccessibilityRequired);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PerformanceBudget {
    pub risk: String,
    pub max_cpu_percent: u8,
    pub max_gpu_layers: u8,
    pub mobile_fallback: String,
}

impl PerformanceBudget {
    fn validate(&self) -> Result<(), PackError> {
        if !matches!(self.risk.as_str(), "low" | "medium" | "high") {
            return Err(PackError::InvalidPerformanceRisk(self.risk.clone()));
        }
        if self.mobile_fallback.trim().is_empty() {
            return Err(PackError::MissingField("performance.mobile_fallback"));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum PackError {
    Read(std::io::Error),
    Parse(serde_json::Error),
    UnsupportedVersion(u32),
    MissingField(&'static str),
    MissingClassification,
    EmptyPack,
    MissingChecksum(String),
    InvalidPath(String),
    InvalidResource(String),
    AccessibilityRequired,
    InvalidPerformanceRisk(String),
    InvalidId(String),
}

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(error) => write!(f, "could not read pack manifest: {error}"),
            Self::Parse(error) => write!(f, "could not parse pack manifest: {error}"),
            Self::UnsupportedVersion(version) => write!(f, "unsupported pack version {version}"),
            Self::MissingField(field) => write!(f, "pack is missing {field}"),
            Self::MissingClassification => write!(f, "pack needs categories, tags, and platforms"),
            Self::EmptyPack => write!(f, "pack must contain at least one resource"),
            Self::MissingChecksum(path) => write!(f, "resource {path} has no checksum"),
            Self::InvalidPath(path) => write!(f, "invalid pack resource path {path:?}"),
            Self::InvalidResource(id) => write!(f, "invalid pack resource {id}"),
            Self::AccessibilityRequired => {
                write!(
                    f,
                    "packs must declare reduced-motion and keyboard-safe behavior"
                )
            }
            Self::InvalidPerformanceRisk(risk) => write!(f, "invalid performance risk {risk:?}"),
            Self::InvalidId(id) => write!(f, "invalid pack id {id:?}"),
        }
    }
}

impl std::error::Error for PackError {}

fn validate_id(id: &str) -> Result<(), PackError> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".-_".contains(character))
    {
        return Err(PackError::InvalidId(id.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> PackManifest {
        PackManifest {
            format_version: PACK_FORMAT_VERSION,
            id: "cybercore-effects.hud".into(),
            version: "1.0.0".into(),
            title: "HUD Effects".into(),
            kind: PackKind::Effects,
            description: "Governed HUD recipes.".into(),
            cybercore: Compatibility {
                minimum: "0.8.1".into(),
                maximum: None,
            },
            categories: vec!["hud".into()],
            tags: vec!["neon".into()],
            license: "MIT".into(),
            provenance: PackProvenance {
                source: "Cybercore".into(),
                author: "Cybercore Team".into(),
                license: "MIT".into(),
                reviewed: true,
            },
            platforms: vec!["web".into()],
            dependencies: Vec::new(),
            resources: vec![PackResource {
                id: "effect.scanline".into(),
                path: "effects/scanline/effect.css".into(),
                title: "Scanline".into(),
                categories: vec!["scanlines".into()],
                tags: vec!["crt".into()],
                license: "MIT".into(),
                fallback: Some("effects/scanline/fallback.css".into()),
                reduced_motion: Some("effects/scanline/reduced-motion.css".into()),
            }],
            checksums: [("effects/scanline/effect.css".into(), "sha256:test".into())]
                .into_iter()
                .collect(),
            accessibility: AccessibilityContract {
                semantic_fallback: "plain content".into(),
                reduced_motion: true,
                keyboard_safe: true,
            },
            performance: PerformanceBudget {
                risk: "low".into(),
                max_cpu_percent: 5,
                max_gpu_layers: 2,
                mobile_fallback: "static line".into(),
            },
            preview: None,
        }
    }

    #[test]
    fn validates_pack_contract() {
        assert!(manifest().validate().is_ok());
    }

    #[test]
    fn requires_checksums_for_every_resource() {
        let mut pack = manifest();
        pack.checksums.clear();
        assert!(matches!(
            pack.validate(),
            Err(PackError::MissingChecksum(_))
        ));
    }
}
