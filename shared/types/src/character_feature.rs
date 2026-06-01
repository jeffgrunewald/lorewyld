use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{EntityId, Timestamp};

/// Catch-all for general character features spanning class, feat,
/// background, and race grants. Used as a deduplication table so that
/// shared traits (Darkvision, Expertise, Evasion…) are defined once.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterFeature {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    pub description: String,
    /// Coarse grouping (vision, expertise, resistance, ability_boost…).
    pub category: String,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
