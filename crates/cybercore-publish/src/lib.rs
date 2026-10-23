//! Publish-plan contracts that keep generation explicit and dry-run first.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublishPlan {
    pub version: String,
    pub target: String,
    pub resources: Vec<String>,
    pub dry_run: bool,
}

impl PublishPlan {
    pub fn dry_run(target: impl Into<String>, resources: Vec<String>) -> Self {
        Self {
            version: "1".to_string(),
            target: target.into(),
            resources,
            dry_run: true,
        }
    }
}

