use serde::{Deserialize, Serialize};

use crate::common::{AbilityScores, ChoiceFrom, EntityId, Senses, Timestamp};

/// One named species trait (Darkvision, Fey Ancestry, …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpeciesTrait {
    pub name: String,
    pub desc: String,
    /// Coarse grouping when the source provides one.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Position within the species writeup, for faithful rendering.
    #[serde(default)]
    pub order: i32,
}

/// A playable species (formerly "race"). Subspecies are sibling rows
/// linked via `subspecies_of` (the Open5e v2 model).
///
/// The ASI/speed/size/language/vision fields are *retained sheet-math
/// data* that Open5e v2 dropped to prose traits; the bundle generator
/// populates them from the v1 API and curated overrides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Species {
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
    pub is_subspecies: bool,
    /// FK -> `species.uuid` of the parent species.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subspecies_of: Option<EntityId>,
    #[serde(default)]
    pub traits: Vec<SpeciesTrait>,
    /// Fixed ability-score increases for character-sheet math.
    pub asi: AbilityScores,
    /// Human-readable rendering, e.g. `"+2 DEX"`.
    pub asi_desc: String,
    /// Walking speed in feet.
    pub speed: i32,
    /// FK -> `sizes.uuid`.
    pub size: EntityId,
    /// Required language FK refs into `languages.uuid`.
    #[serde(default)]
    pub languages_base: Vec<EntityId>,
    /// Optional extra-language choice, e.g. `{ choose: 1, from: [...] }`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub languages_additional: Option<ChoiceFrom>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vision_base: Option<Senses>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Slim list-projection of a [`Species`] (see `Species::summary`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpeciesSummary {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub document_uuid: EntityId,
    pub key: String,
    pub slug: String,
    pub name: String,
    pub is_subspecies: bool,
    pub size: EntityId,
    pub speed: i32,
    pub subspecies_of: Option<EntityId>,
}

impl Species {
    /// Derives the list-row summary from the full record.
    pub fn summary(&self) -> SpeciesSummary {
        SpeciesSummary {
            uuid: self.uuid,
            content_module_uuid: self.content_module_uuid,
            document_uuid: self.document_uuid,
            key: self.key.clone(),
            slug: self.slug.clone(),
            name: self.name.clone(),
            is_subspecies: self.is_subspecies,
            size: self.size,
            speed: self.speed,
            subspecies_of: self.subspecies_of,
        }
    }
}
