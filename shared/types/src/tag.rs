use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{EntityId, Timestamp};

/// A label that can be attached to lore notes (and, in later tiers,
/// structured records) for search and organization.
///
/// Tags use a **global slug namespace with collide-by-merge** semantics:
/// the common-case search (a user typing `npc` to find every NPC) works
/// frictionlessly. Module-private namespacing is opt-in via prefix slugs
/// (e.g. `vr/fey-realm` for a Verdant Realms-specific tag); the system
/// does not enforce or auto-disambiguate.
///
/// `is_system` flags well-known tags shipped with the app (reserved
/// slugs that user-introduced tags must not collide with).
/// `introduced_by_module_uuid` records attribution when a tag entered
/// the namespace via a published content module.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub uuid: EntityId,
    pub slug: String,
    pub display_name: String,
    #[serde(default)]
    pub is_system: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introduced_by_module_uuid: Option<EntityId>,
    pub created_at: Timestamp,
}

/// Alternate slug that resolves to the same tag — supports synonyms like
/// `npc` ↔ `non-player-character`. Aliases do not introduce new tags.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagAlias {
    pub tag_uuid: EntityId,
    pub alias_slug: String,
}
