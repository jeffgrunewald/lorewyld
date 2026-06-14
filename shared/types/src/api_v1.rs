//! Wire types for the v1 HTTP API.
//!
//! These DTOs live here (not server-side only) so the mobile and web
//! clients deserialize them from the same shared Rust definitions that
//! the catalog types use.

use serde::{Deserialize, Serialize};

use crate::{
    common::EntityId,
    content_module::{ContentModule, LicenseKind},
    lore_note::{LoreNote, NoteScope, NoteScopeKind, NoteVisibility},
    tag::Tag,
    user::User,
};

/// `POST /api/users/register` payload. The join code gates account
/// creation; the password is hashed server-side before storage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub join_code: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

/// `POST /api/users/login` payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Response body for both registration and login: the authenticated
/// user plus the session token to attach to subsequent requests via
/// `Authorization: Bearer <token>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: User,
    pub session_token: String,
}

/// `POST /api/users/password` payload — self-service password change
/// for the logged-in user. The current password re-proves identity;
/// the new password must satisfy the same policy as registration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// `POST /api/admin/users` payload — admin-driven account creation.
/// Same shape as registration minus the join code (admin access
/// supersedes it).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminCreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// `PATCH /api/admin/users/:uuid` payload — toggles the admin flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub admin: bool,
}

/// `GET /api/admin/users` response — one page of registered users.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserListResponse {
    pub users: Vec<User>,
    pub page: u32,
    pub limit: u32,
    pub total: u32,
}

/// `GET /api/admin/server` response — the editable server identity
/// plus the read-only software version for display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerSettings {
    pub name: String,
    pub join_code: String,
    pub version: String,
}

/// `PATCH /api/admin/server` payload. Omitted fields stay. The join
/// code is not directly editable — `POST /api/admin/server/join-code`
/// regenerates it server-side instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UpdateServerSettingsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Server-identity summary exposed by `GET /api/server-info` — omits
/// the `join_code` so the endpoint can be polled without leaking the
/// registration secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameServerSummary {
    pub uuid: EntityId,
    pub name: String,
    pub version: String,
}

/// `GET /api/server-info` response. Includes the installed-module
/// manifest so clients can render attribution labels and search filters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerInfo {
    pub server: GameServerSummary,
    pub modules: Vec<ContentModule>,
}

/// A `LoreNote` plus its attached tags. Used as the response shape for
/// every endpoint that returns notes — the raw `LoreNote` storage row
/// doesn't carry tag attachments, which are stored separately via the
/// `tag_attachment_lore_note` join table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoreNoteWithTags {
    pub note: LoreNote,
    pub tags: Vec<Tag>,
}

/// `POST /api/lore-notes` request body. The server fills in `uuid`,
/// `created_by_user_uuid`, and timestamps from the caller's session.
/// Tag slugs are resolved (or auto-created as user tags) before
/// attachment.
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTagRequest {
    pub slug: String,
    pub display_name: String,
}

/// `POST /api/settings` request body. The server fills in `uuid`,
/// `owner_user_uuid`, and timestamps from the caller's session.
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UpdateSettingRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_note_uuid: Option<EntityId>,
}

/// `POST /api/settings/:uuid/collaborators` request body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddCollaboratorRequest {
    pub user_uuid: EntityId,
}

/// `POST /api/search` request body. Composes free-text FTS5 matching
/// with tag filtering and scope filtering. All fields optional —
/// omitted filters expand the result set.
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResponse {
    pub notes: Vec<LoreNoteWithTags>,
}

/// `POST /api/modules` request body — the Promote-to-Module wizard's
/// commit payload. The server snapshot-copies every selected
/// `LoreNote` from the source `Setting` scope into the new
/// `ContentModule`'s scope and links the source `Setting` to the
/// published module via `Setting.published_as_module_uuid`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishModuleRequest {
    pub source_setting_uuid: EntityId,
    pub name: String,
    pub slug: String,
    /// Homebrew may publish as `Unlicensed`; bundled content may not.
    pub license: LicenseKind,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishModuleResponse {
    pub module: ContentModule,
    pub note_count: u32,
}

/// How a `ContentModule` landed on this server. Server-side metadata —
/// deliberately not part of `ContentModule` so it never leaks into
/// exported/imported `ContentBundle`s.
///
/// `Bundled` modules can only be disabled (the boot seeder would
/// re-add a deleted bundled module); the rest are fully uninstallable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModuleOrigin {
    Bundled,
    Uploaded,
    Published,
}

/// Record count for one content category (e.g. `spells: 319`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: u32,
}

/// `GET /api/admin/modules` row — every module on the server (active
/// or not) with its provenance and per-category record counts for the
/// management UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminModuleSummary {
    pub module: ContentModule,
    pub origin: ModuleOrigin,
    pub record_counts: Vec<CategoryCount>,
    pub lore_note_count: u32,
}

/// `PATCH /api/admin/modules/:uuid` payload — disable (`false`) or
/// reinstall/activate (`true`) a module. Disabled module content stays
/// in the database but is excluded from every content read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateModuleStatusRequest {
    pub is_active: bool,
}

/// `POST /api/admin/modules/install` response. The request body is a
/// complete `ContentBundle` package file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallModuleResponse {
    pub installed: Vec<ContentModule>,
    pub record_count: u32,
}

/// `GET /api/content/counts` response — entry counts per compendium
/// category across active modules, for the landing-grid tiles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentCountsResponse {
    pub counts: Vec<CategoryCount>,
}

/// One row of the home page's recently-added list: enough to render a
/// link to `/compendium/{category}/{uuid}` with attribution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentContentItem {
    pub category: String,
    pub uuid: EntityId,
    pub name: String,
    pub module_name: String,
    /// Record's own `created_at` (RFC 3339), the feed's recency basis;
    /// `None` for records whose stored blob omits it.
    pub created_at: Option<String>,
}

/// `GET /api/content/recent` response — newest content entries across
/// active modules, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentContentResponse {
    pub items: Vec<RecentContentItem>,
}
