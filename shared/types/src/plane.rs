use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{EntityId, Timestamp};

/// Top-level grouping of planes.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaneKind {
    Material,
    Inner,
    Outer,
    Astral,
    Ethereal,
    Transitive,
}

/// A cosmological plane of existence in the D&D multiverse. Forms a
/// tree via `parent_uuid`.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plane {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    pub desc: String,
    pub plane_type: PlaneKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alignment_association: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elemental_association: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_uuid: Option<EntityId>,
    #[serde(default)]
    pub inhabitants: Vec<String>,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
