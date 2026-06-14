use serde::{Deserialize, Serialize};

use crate::common::{EntityId, Timestamp};

/// One typed benefit a background grants (Open5e v2 `benefits`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundBenefit {
    pub name: String,
    pub desc: String,
    /// `"ability_score"`, `"skill_proficiency"`, `"tool_proficiency"`,
    /// `"language"`, `"equipment"`, `"feature"`,
    /// `"suggested_characteristics"`, …
    #[serde(rename = "type")]
    pub kind: String,
}

/// A character background. Mechanics live in the typed `benefits` array.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Background {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    /// FK -> `documents.uuid` (source attribution).
    pub document_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    pub desc: String,
    #[serde(default)]
    pub benefits: Vec<BackgroundBenefit>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
