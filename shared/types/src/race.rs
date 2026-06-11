use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{AbilityScores, EntityId, Json, Senses, Timestamp};

/// A character race/species.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Race {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    pub desc: String,
    /// Fixed ability-score increases. For variable ASIs see `asi_source`.
    pub asi_values: AbilityScores,
    /// Human-readable rendering, e.g. `"+2 DEX"`.
    pub asi_desc: String,
    /// Variable ASI options (Customization, Half-Elf, etc.).
    #[serde(default)]
    pub asi_source: Vec<Json>,
    pub age: String,
    pub alignment: String,
    pub size: String,
    /// Original SRD source text for the `size` field, retained for
    /// presentation fidelity.
    pub size_raw: String,
    pub speed: i32,
    pub speed_desc: String,
    /// Required language slugs.
    #[serde(default)]
    pub languages_base: Vec<EntityId>,
    /// Optional extra-language choice, e.g. `{ choose: 1, from: [...] }`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub languages_additional: Option<Json>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vision_base: Option<Senses>,
    /// Trait names granted to every member of the race.
    #[serde(default)]
    pub trait_names: Vec<String>,
    /// `{ trait_name: description }` map of trait text.
    pub trait_definitions: Json,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A reusable racial-trait definition shared across races.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RacialTrait {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    pub description: String,
    /// Coarse grouping (vision, ancestry, mortality, resistance, etc.).
    pub category: String,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
