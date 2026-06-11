use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{EntityId, Timestamp};

/// A language from the SRD with its script, speakers, and classification.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Language {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    pub script: String,
    /// Where the language originates (Core, Underdark, Abyss…).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// Typical speaker groups (Humans, Elves, etc.).
    #[serde(default)]
    pub speakers: Vec<String>,
    pub is_independent: bool,
    pub is_tonal: bool,
    pub is_alphabetical: bool,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
