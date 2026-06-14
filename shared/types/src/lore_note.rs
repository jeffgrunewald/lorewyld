use serde::{Deserialize, Serialize};

use crate::common::{EntityId, Timestamp};

/// What sort of container a `LoreNote` belongs to. Each variant identifies
/// the table the `target_uuid` points at.
///
/// The four scopes mirror the platform's content commitment ramp plus
/// the personal/character axis:
///
/// - `Module` — published worldbuilding lore attached to a `ContentModule`.
/// - `Setting` — worldbuilding in a user's personal `Setting` workspace,
///   not yet polished for publication. Promotable to `Module`.
/// - `Campaign` — per-game notes (session recaps, NPCs, plot threads)
///   attached to a `Campaign`. Promotable to `Setting` when a note turns
///   out to be reusable. (`Campaign` table arrives in v1.5.)
/// - `Character` — character backstories and journal entries attached to
///   a `PlayerCharacter`. (`PlayerCharacter` arrives in v1.5.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteScopeKind {
    Module,
    Setting,
    Campaign,
    Character,
}

/// Storage-friendly representation of a note's scope. The discriminator
/// and target_uuid pair maps directly onto the server's `scope_kind` and
/// `scope_target_uuid` columns. On the wire it serializes as a small JSON
/// object the client can deconstruct without parsing a union type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteScope {
    pub kind: NoteScopeKind,
    pub target_uuid: EntityId,
}

/// Visibility flag used purely for **rendering at the current viewer**.
///
/// This is the only access-restriction primitive in the platform.
/// `GamemasterOnly` exists so a GM can hide upcoming-content notes from
/// players to prevent spoilers; it does NOT prevent the note from being
/// copied or exported. Visibility metadata travels with the note; what
/// a recipient sees depends on the recipient's role at render time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NoteVisibility {
    #[default]
    Visible,
    AuthorOnly,
    GamemasterOnly,
}

/// Unstructured markdown content with tag-driven organization.
///
/// A single record type holds every flavor of loose content the
/// platform supports: worldbuilding lore, campaign session recaps,
/// character backstories, NPC sketches, faction codices, etc. The
/// `scope` field discriminates the note's home; the markdown body is
/// rendered identically regardless.
///
/// `derived_from_setting_note_uuid` is set on `Module`-scope notes
/// created via the Promote-to-Module wizard, pointing back to the
/// `Setting`-scope source. It powers the republish-diff algorithm
/// when a setting publishes an updated module version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoreNote {
    pub uuid: EntityId,
    pub title: String,
    #[serde(default)]
    pub body_markdown: String,
    pub scope: NoteScope,
    #[serde(default)]
    pub visibility: NoteVisibility,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived_from_setting_note_uuid: Option<EntityId>,
    /// `None` when the authoring account has been deleted — content
    /// outlives its author.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_user_uuid: Option<EntityId>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
