use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{ChoiceFrom, EntityId, Timestamp};

/// A character background with skills, proficiencies, languages,
/// equipment, and personality features.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Background {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    pub desc: String,
    pub skill_proficiencies: ChoiceFrom,
    pub tool_proficiencies: ChoiceFrom,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub languages: Option<ChoiceFrom>,
    #[serde(default)]
    pub equipment: Vec<String>,
    /// Background feature name (e.g. "Criminal Contact").
    pub feature_name: String,
    pub feature_desc: String,
    #[serde(default)]
    pub personality_traits: Vec<String>,
    #[serde(default)]
    pub ideals: Vec<String>,
    #[serde(default)]
    pub bonds: Vec<String>,
    #[serde(default)]
    pub flaws: Vec<String>,
    /// Flat list of every proficiency name the background grants.
    #[serde(default)]
    pub proficiency_names: Vec<String>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
