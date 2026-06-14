use serde::{Deserialize, Serialize};

use crate::common::{EntityId, Timestamp};

/// A language, structured per the Open5e v2 schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Language {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    #[serde(default)]
    pub desc: String,
    /// Exotic languages need explicit GM permission in some campaigns.
    #[serde(default)]
    pub is_exotic: bool,
    /// Secret languages (Druidic, Thieves' Cant) aren't generally
    /// learnable.
    #[serde(default)]
    pub is_secret: bool,
    /// Key of the language whose script this one uses, when written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
