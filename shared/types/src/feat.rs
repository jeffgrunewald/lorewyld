use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{EntityId, Timestamp};

/// One discrete benefit a feat grants (Open5e v2 `benefits`).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatBenefit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub desc: String,
}

/// A character feat.
#[typeshare]
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
