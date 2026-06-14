use serde::{Deserialize, Serialize};

use crate::common::{EntityId, Timestamp};

/// A user-owned worldbuilding workspace — the **promotion staging
/// ground** in the platform's three-tier content commitment ramp:
///
///   campaign-scoped *(tonight's game)*
///     → setting-scoped *(my reusable world)*
///     → module-scoped *(published, shareable package)*
///
/// Settings hold lore notes (`LoreNote` with `NoteScope::Setting`) and,
/// in later tiers, structured homebrew records the user is developing
/// for a setting. The Promote-to-Module wizard takes a Setting and
/// produces a publishable `ContentModule` via snapshot publication.
/// After publication, the source Setting persists with
/// `published_as_module_uuid` set, and continues to evolve
/// independently of the frozen module versions it has spawned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Setting {
    pub uuid: EntityId,
    pub name: String,
    /// Optional `SettingScope` lore note serving as the world primer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_note_uuid: Option<EntityId>,
    /// `None` when the owning account has been deleted — content
    /// outlives its owner.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_user_uuid: Option<EntityId>,
    /// Set when the setting has been published as a module via the
    /// Promote-to-Module wizard. Points to the most recent published
    /// version; older versions are reachable via the module's own
    /// `previous_version_uuid` chain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_as_module_uuid: Option<EntityId>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Co-author relationship — a user invited to collaborate on a setting
/// has read/write access to its notes and (in later tiers) its draft
/// structured records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingCollaborator {
    pub setting_uuid: EntityId,
    pub user_uuid: EntityId,
}
