//! Wire types for the v1 HTTP API.
//!
//! These DTOs live here (not server-side only) so the mobile and web
//! clients can deserialize them via the same typeshare-generated bindings
//! that the catalog types use.

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    app_user::AppUser,
    common::EntityId,
    content_module::ContentModule,
    lore_note::{LoreNote, NoteScope, NoteScopeKind, NoteVisibility},
    tag::Tag,
};

/// `POST /api/users/register` payload.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub join_code: String,
    pub display_name: String,
}

/// `POST /api/users/login` payload. v1 ships without passwords —
/// knowing the display name within a server is sufficient to obtain a
/// session token. Future tiers will add credentials.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginRequest {
    pub display_name: String,
}

/// Response body for both registration and login: the authenticated
/// user plus the session token to attach to subsequent requests via
/// `Authorization: Bearer <token>`.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: AppUser,
    pub session_token: String,
}

/// Server-identity summary exposed by `GET /api/server-info` — omits
/// the `join_code` so the endpoint can be polled without leaking the
/// registration secret.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameServerSummary {
    pub uuid: EntityId,
    pub name: String,
    pub version: String,
}

/// `GET /api/server-info` response. Includes the installed-module
/// manifest so clients can render attribution labels and search filters.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerInfo {
    pub server: GameServerSummary,
    pub modules: Vec<ContentModule>,
}

/// A `LoreNote` plus its attached tags. Used as the response shape for
/// every endpoint that returns notes — the raw `LoreNote` storage row
/// doesn't carry tag attachments, which are stored separately via the
/// `tag_attachment_lore_note` join table.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoreNoteWithTags {
    pub note: LoreNote,
    pub tags: Vec<Tag>,
}

/// `POST /api/lore-notes` request body. The server fills in `uuid`,
/// `created_by_user_uuid`, and timestamps from the caller's session.
/// Tag slugs are resolved (or auto-created as user tags) before
/// attachment.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateLoreNoteRequest {
    pub title: String,
    #[serde(default)]
    pub body_markdown: String,
    pub scope: NoteScope,
    #[serde(default)]
    pub visibility: NoteVisibility,
    #[serde(default)]
    pub tag_slugs: Vec<String>,
}

/// `PATCH /api/lore-notes/:uuid` request body. Any field omitted is
/// left unchanged. `scope` is intentionally not modifiable here —
/// scope transitions (Campaign → Setting promotion, etc.) get their
/// own dedicated endpoints to make the action explicit.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UpdateLoreNoteRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_markdown: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<NoteVisibility>,
    /// When set, replaces the note's full tag set (use `Some(vec![])`
    /// to clear all tags). When `None`, tags are left untouched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_slugs: Option<Vec<String>>,
}

/// `POST /api/tags` request body. Creates a new user-introduced tag.
/// The slug must not collide with an existing tag slug; the server
/// returns 409 if it does.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTagRequest {
    pub slug: String,
    pub display_name: String,
}

/// `POST /api/settings` request body. The server fills in `uuid`,
/// `owner_user_uuid`, and timestamps from the caller's session.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateSettingRequest {
    pub name: String,
    /// Optional initial setting-scope LoreNote uuid to install as the
    /// world primer. Usually clients create the Setting first and then
    /// add a description note in a follow-up call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_note_uuid: Option<EntityId>,
}

/// `PATCH /api/settings/:uuid` request body. Omitted fields stay.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UpdateSettingRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_note_uuid: Option<EntityId>,
}

/// `POST /api/settings/:uuid/collaborators` request body.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddCollaboratorRequest {
    pub user_uuid: EntityId,
}

/// `POST /api/search` request body. Composes free-text FTS5 matching
/// with tag filtering and scope filtering. All fields optional —
/// omitted filters expand the result set.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SearchRequest {
    /// Free-text query against `title + body_markdown`. FTS5 syntax.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    /// Restrict by note scope kind (`module` / `setting` / `campaign` /
    /// `character`). Combine with `scope_target_uuid` for a single
    /// container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_kind: Option<NoteScopeKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_target_uuid: Option<EntityId>,
    /// All listed tag slugs must be attached for a note to match (AND
    /// semantics).
    #[serde(default)]
    pub tag_slugs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// `POST /api/search` response. v1 returns only lore-note matches;
/// structured-record search lands in v1.5 alongside structured
/// authoring.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResponse {
    pub notes: Vec<LoreNoteWithTags>,
}

/// `POST /api/modules` request body — the Promote-to-Module wizard's
/// commit payload. The server snapshot-copies every selected
/// `LoreNote` from the source `Setting` scope into the new
/// `ContentModule`'s scope and links the source `Setting` to the
/// published module via `Setting.published_as_module_uuid`.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishModuleRequest {
    pub source_setting_uuid: EntityId,
    pub name: String,
    pub slug: String,
    pub license: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    /// Semver string for the published module version. Must increase
    /// monotonically when republishing an existing module.
    pub version_string: String,
    /// UUIDs of `Setting`-scope `LoreNote`s to include in the snapshot.
    /// Notes not listed here are excluded from the published module.
    pub selected_note_uuids: Vec<EntityId>,
}

/// `POST /api/modules` response: the newly created module's full row.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishModuleResponse {
    pub module: ContentModule,
    pub note_count: u32,
}
