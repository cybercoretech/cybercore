//! Typed workspace contracts shared by the Cybercore Control Plane and tools.
//!
//! The workspace is intentionally metadata-first: editors may store their
//! native project files beside a resource manifest, while consumers only need
//! this stable contract to discover, validate, and package resources.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CONTROL_PLANE_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ControlPlaneWorkspace {
    pub format_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub resources: Vec<ResourceManifest>,
    #[serde(default)]
    pub sites: Vec<SiteProfile>,
    #[serde(default)]
    pub settings: WorkspaceSettings,
}

impl ControlPlaneWorkspace {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            format_version: CONTROL_PLANE_FORMAT_VERSION,
            id: id.into(),
            name: name.into(),
            ..Self::default()
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, WorkspaceError> {
        let raw = std::fs::read_to_string(path).map_err(WorkspaceError::Read)?;
        let workspace: Self = serde_json::from_str(&raw).map_err(WorkspaceError::Parse)?;
        workspace.validate()?;
        Ok(workspace)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), WorkspaceError> {
        self.validate()?;
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(WorkspaceError::Write)?;
        }
        let raw = serde_json::to_string_pretty(self).map_err(WorkspaceError::Serialize)?;
        std::fs::write(path, format!("{raw}\n")).map_err(WorkspaceError::Write)
    }

    pub fn validate(&self) -> Result<(), WorkspaceError> {
        if self.format_version != CONTROL_PLANE_FORMAT_VERSION {
            return Err(WorkspaceError::UnsupportedVersion(self.format_version));
        }
        validate_id("workspace", &self.id)?;
        if self.name.trim().is_empty() {
            return Err(WorkspaceError::MissingField("name"));
        }
        for resource in &self.resources {
            resource.validate()?;
        }
        for site in &self.sites {
            site.validate()?;
        }
        Ok(())
    }

    pub fn resource(&self, id: &str) -> Option<&ResourceManifest> {
        self.resources.iter().find(|resource| resource.id == id)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResourceManifest {
    pub id: String,
    pub kind: ResourceKind,
    pub version: String,
    pub status: ResourceStatus,
    pub title: String,
    pub path: PathBuf,
    pub license: String,
    pub provenance: Provenance,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub preview: Option<PathBuf>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl ResourceManifest {
    pub fn validate(&self) -> Result<(), WorkspaceError> {
        validate_id("resource", &self.id)?;
        if self.version.trim().is_empty() {
            return Err(WorkspaceError::MissingResourceField {
                id: self.id.clone(),
                field: "version",
            });
        }
        if self.title.trim().is_empty() {
            return Err(WorkspaceError::MissingResourceField {
                id: self.id.clone(),
                field: "title",
            });
        }
        if self.path.as_os_str().is_empty() || self.path.is_absolute() {
            return Err(WorkspaceError::InvalidResourcePath(self.id.clone()));
        }
        if self.license.trim().is_empty() {
            return Err(WorkspaceError::MissingResourceField {
                id: self.id.clone(),
                field: "license",
            });
        }
        self.provenance.validate(&self.id)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceKind {
    Theme,
    Component,
    Effect,
    Snippet,
    Asset,
    Font,
    SiteProfile,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceStatus {
    Draft,
    Review,
    Approved,
    Deprecated,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Provenance {
    pub source: String,
    pub author: String,
    pub license: String,
    #[serde(default)]
    pub reviewed: bool,
}

impl Provenance {
    fn validate(&self, resource_id: &str) -> Result<(), WorkspaceError> {
        for (field, value) in [
            ("source", &self.source),
            ("author", &self.author),
            ("license", &self.license),
        ] {
            if value.trim().is_empty() {
                return Err(WorkspaceError::MissingProvenanceField {
                    id: resource_id.to_string(),
                    field,
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct SiteProfile {
    pub id: String,
    pub repository: String,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub resources: Vec<String>,
    #[serde(default = "default_motion")]
    pub motion: String,
    #[serde(default = "default_fallback")]
    pub fallback: String,
}

impl SiteProfile {
    fn validate(&self) -> Result<(), WorkspaceError> {
        validate_id("site", &self.id)?;
        if self.repository.trim().is_empty() {
            return Err(WorkspaceError::MissingSiteField {
                id: self.id.clone(),
                field: "repository",
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct WorkspaceSettings {
    #[serde(default = "default_control_plane_version")]
    pub control_plane_version: String,
    #[serde(default)]
    pub default_platforms: Vec<String>,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            control_plane_version: default_control_plane_version(),
            default_platforms: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum WorkspaceError {
    Read(std::io::Error),
    Write(std::io::Error),
    Parse(serde_json::Error),
    Serialize(serde_json::Error),
    UnsupportedVersion(u32),
    MissingField(&'static str),
    MissingResourceField { id: String, field: &'static str },
    MissingProvenanceField { id: String, field: &'static str },
    MissingSiteField { id: String, field: &'static str },
    InvalidResourcePath(String),
    InvalidId { kind: &'static str, id: String },
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(error) => write!(f, "could not read workspace: {error}"),
            Self::Write(error) => write!(f, "could not write workspace: {error}"),
            Self::Parse(error) => write!(f, "could not parse workspace: {error}"),
            Self::Serialize(error) => write!(f, "could not serialize workspace: {error}"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported Control Plane workspace version {version}")
            }
            Self::MissingField(field) => write!(f, "workspace is missing {field}"),
            Self::MissingResourceField { id, field } => {
                write!(f, "resource {id} is missing {field}")
            }
            Self::MissingProvenanceField { id, field } => {
                write!(f, "resource {id} is missing provenance.{field}")
            }
            Self::MissingSiteField { id, field } => write!(f, "site {id} is missing {field}"),
            Self::InvalidResourcePath(id) => write!(f, "resource {id} has an invalid path"),
            Self::InvalidId { kind, id } => write!(f, "invalid {kind} id {id:?}"),
        }
    }
}

impl std::error::Error for WorkspaceError {}

fn validate_id(kind: &'static str, id: &str) -> Result<(), WorkspaceError> {
    let valid = !id.is_empty()
        && id.len() <= 128
        && id.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        });
    if valid {
        Ok(())
    } else {
        Err(WorkspaceError::InvalidId {
            kind,
            id: id.to_string(),
        })
    }
}

fn default_control_plane_version() -> String {
    "0.1.0".into()
}

fn default_motion() -> String {
    "full".into()
}

fn default_fallback() -> String {
    "static".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_workspace_and_round_trips_json() {
        let mut workspace = ControlPlaneWorkspace::new("cybercore", "Cybercore");
        workspace.resources.push(ResourceManifest {
            id: "theme.vault-midnight".into(),
            kind: ResourceKind::Theme,
            version: "1.0.0".into(),
            status: ResourceStatus::Approved,
            title: "Vault Midnight".into(),
            path: PathBuf::from("resources/themes/vault-midnight.json"),
            license: "MIT".into(),
            provenance: Provenance {
                source: "Cybercore".into(),
                author: "Cybercore".into(),
                license: "MIT".into(),
                reviewed: true,
            },
            tags: vec!["security".into()],
            dependencies: Vec::new(),
            platforms: vec!["web".into(), "terminal".into()],
            preview: None,
            metadata: serde_json::Value::Null,
        });
        workspace.sites.push(SiteProfile {
            id: "site.cybervault-android".into(),
            repository: "cybervault-android-app".into(),
            theme: Some("theme.vault-midnight".into()),
            resources: vec!["theme.vault-midnight".into()],
            ..SiteProfile::default()
        });
        let raw = serde_json::to_string(&workspace).expect("serialize");
        let decoded: ControlPlaneWorkspace = serde_json::from_str(&raw).expect("parse");
        assert_eq!(decoded, workspace);
        assert!(decoded.validate().is_ok());
    }

    #[test]
    fn rejects_absolute_resource_paths() {
        let mut workspace = ControlPlaneWorkspace::new("cybercore", "Cybercore");
        workspace.resources.push(ResourceManifest {
            id: "asset.logo".into(),
            kind: ResourceKind::Asset,
            version: "1.0.0".into(),
            status: ResourceStatus::Draft,
            title: "Logo".into(),
            path: PathBuf::from("/tmp/logo.svg"),
            license: "MIT".into(),
            provenance: Provenance {
                source: "Cybercore".into(),
                author: "Cybercore".into(),
                license: "MIT".into(),
                reviewed: false,
            },
            tags: Vec::new(),
            dependencies: Vec::new(),
            platforms: Vec::new(),
            preview: None,
            metadata: serde_json::Value::Null,
        });
        assert!(matches!(
            workspace.validate(),
            Err(WorkspaceError::InvalidResourcePath(_))
        ));
    }
}
