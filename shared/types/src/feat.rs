use serde::{Deserialize, Serialize};

use crate::common::{EntityId, Timestamp};

/// One discrete benefit a feat grants (Open5e v2 `benefits`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatBenefit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub desc: String,
}

/// A character feat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Feat {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    /// FK -> `documents.uuid` (source attribution).
    pub document_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    pub desc: String,
    /// Feat category, e.g. `"GENERAL"`, `"ORIGIN"`, `"FIGHTING_STYLE"`.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default)]
    pub has_prerequisite: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prerequisite: Option<String>,
    #[serde(default)]
    pub benefits: Vec<FeatBenefit>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Slim list-projection of a [`Feat`] (see `Feat::summary`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatSummary {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub document_uuid: EntityId,
    pub key: String,
    pub slug: String,
    pub name: String,
    pub has_prerequisite: bool,
    pub prerequisite: Option<String>,
}

impl Feat {
    /// Derives the list-row summary from the full record.
    pub fn summary(&self) -> FeatSummary {
        FeatSummary {
            uuid: self.uuid,
            content_module_uuid: self.content_module_uuid,
            document_uuid: self.document_uuid,
            key: self.key.clone(),
            slug: self.slug.clone(),
            name: self.name.clone(),
            has_prerequisite: self.has_prerequisite,
            prerequisite: self.prerequisite.clone(),
        }
    }
}
