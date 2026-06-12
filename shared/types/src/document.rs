use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{EntityId, Timestamp};

/// A content license (CC-BY-4.0, CC0, OGL-1.0a). Carries the full
/// license text so attribution can render offline.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct License {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Full license text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// The organization that published a source document.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Publisher {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A source document for content attribution (SRD 5.1, SRD 5.2, …).
/// Every major content record carries a `document_uuid` so per-record
/// provenance survives modules that mix sources (e.g. an SRD 5.2 base
/// gap-filled from SRD 5.1).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    /// Attribution text required by the document's license.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    pub license_uuid: EntityId,
    pub publisher_uuid: EntityId,
    /// Game-system key (e.g. `"5e-2014"`, `"5e-2024"`). A bare string —
    /// one shipped game system doesn't justify a lookup table.
    pub gamesystem_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permalink: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_on: Option<NaiveDate>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
