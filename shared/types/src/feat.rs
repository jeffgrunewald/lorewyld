use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{AbilityScores, EntityId, Json, Timestamp};

/// A character feat.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feat {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    pub desc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prerequisite: Option<String>,
    /// Distinct effects the feat grants — kept Json because the SRD
    /// mixes prose, bullets, and table-style entries.
    #[serde(default)]
    pub effects: Vec<Json>,
    /// Optional ASI granted by the feat itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ability_score_bump: Option<AbilityScores>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
