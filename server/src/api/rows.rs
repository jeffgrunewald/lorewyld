//! Shared sqlx row types and enum string codecs for entities read by
//! multiple endpoint modules. One row struct per table and one codec per
//! closed enum, so adding a scope kind or visibility variant is a
//! single-site change instead of a hunt across handlers.

use chrono::{DateTime, NaiveDate, Utc};
use lorewyld_types::{
    content_module::ContentModule,
    lore_note::{LoreNote, NoteScope, NoteScopeKind, NoteVisibility},
};
use uuid::Uuid;

use crate::api::error::ApiError;

/// Per-note visibility WHERE clause. Expects the `lore_note` table to be
/// aliased `n` and binds one value: the caller's user uuid. Visible notes
/// are readable by anyone authenticated; AuthorOnly/GamemasterOnly only
/// by their creator. v1.5 broadens GamemasterOnly to campaign DMs — this
/// constant is the single place that check changes.
///
/// A NULL author (deleted account) never matches the bound uuid, so
/// orphaned restricted notes are invisible to everyone.
pub const VISIBILITY_PREDICATE: &str = "(n.visibility = 'visible' OR n.created_by_user_uuid = ?)";

/// Unaliased lore-note select (single-table queries).
pub const LORE_NOTE_SELECT: &str = "SELECT uuid, title, body_markdown, scope_kind, \
                                    scope_target_uuid, visibility, \
                                    derived_from_setting_note_uuid, \
                                    created_by_user_uuid, created_at, updated_at \
                                    FROM lore_note";

/// `n`-aliased lore-note select, for queries that append JOINs and use
/// [`VISIBILITY_PREDICATE`].
pub const LORE_NOTE_SELECT_N: &str = "SELECT n.uuid, n.title, n.body_markdown, n.scope_kind, \
                                      n.scope_target_uuid, n.visibility, \
                                      n.derived_from_setting_note_uuid, \
                                      n.created_by_user_uuid, n.created_at, n.updated_at \
                                      FROM lore_note n";

#[derive(sqlx::FromRow)]
pub struct LoreNoteRow {
    pub uuid: String,
    pub title: String,
    pub body_markdown: String,
    pub scope_kind: String,
    pub scope_target_uuid: String,
    pub visibility: String,
    pub derived_from_setting_note_uuid: Option<String>,
    pub created_by_user_uuid: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl LoreNoteRow {
    pub fn into_dto(self) -> Result<LoreNote, ApiError> {
        Ok(LoreNote {
            uuid: parse_uuid(&self.uuid)?,
            title: self.title,
            body_markdown: self.body_markdown,
            scope: NoteScope {
                kind: scope_kind_from_str(&self.scope_kind)?,
                target_uuid: parse_uuid(&self.scope_target_uuid)?,
            },
            visibility: visibility_from_str(&self.visibility)?,
            derived_from_setting_note_uuid: self
                .derived_from_setting_note_uuid
                .as_deref()
                .map(parse_uuid)
                .transpose()?,
            created_by_user_uuid: self
                .created_by_user_uuid
                .as_deref()
                .map(parse_uuid)
                .transpose()?,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// Active-module listing, shared by `GET /api/modules` and the
/// server-info manifest.
pub const MODULE_SELECT_ACTIVE: &str = "SELECT uuid, name, slug, license, license_url, \
                                        schema_version, release_date, authors, publisher, \
                                        description, website_url, is_active, ordering, \
                                        version_string, previous_version_uuid, published_at, \
                                        created_at, updated_at \
                                        FROM content_module WHERE is_active = 1 \
                                        ORDER BY ordering, name";

pub const MODULE_SELECT_ONE: &str = "SELECT uuid, name, slug, license, license_url, \
                                     schema_version, release_date, authors, publisher, \
                                     description, website_url, is_active, ordering, \
                                     version_string, previous_version_uuid, published_at, \
                                     created_at, updated_at \
                                     FROM content_module WHERE uuid = ?";

#[derive(sqlx::FromRow)]
pub struct ContentModuleRow {
    pub uuid: String,
    pub name: String,
    pub slug: String,
    pub license: String,
    pub license_url: Option<String>,
    pub schema_version: i64,
    pub release_date: Option<NaiveDate>,
    pub authors: String,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub website_url: Option<String>,
    pub is_active: i64,
    pub ordering: i64,
    pub version_string: String,
    pub previous_version_uuid: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ContentModuleRow {
    pub fn into_dto(self) -> Result<ContentModule, ApiError> {
        Ok(ContentModule {
            uuid: parse_uuid(&self.uuid)?,
            name: self.name,
            slug: self.slug,
            // Rows predating the license enumeration (or hand-edited
            // values) degrade to Unlicensed rather than erroring.
            license: lorewyld_types::LicenseKind::from_wire(&self.license)
                .unwrap_or(lorewyld_types::LicenseKind::Unlicensed),
            license_url: self.license_url,
            schema_version: self.schema_version as u32,
            release_date: self.release_date,
            authors: serde_json::from_str(&self.authors)
                .map_err(|e| ApiError::Internal(e.into()))?,
            publisher: self.publisher,
            description: self.description,
            website_url: self.website_url,
            is_active: self.is_active != 0,
            ordering: self.ordering as i32,
            version_string: self.version_string,
            previous_version_uuid: self
                .previous_version_uuid
                .as_deref()
                .map(parse_uuid)
                .transpose()?,
            published_at: self.published_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

pub fn scope_kind_to_str(kind: NoteScopeKind) -> &'static str {
    match kind {
        NoteScopeKind::Module => "module",
        NoteScopeKind::Setting => "setting",
        NoteScopeKind::Campaign => "campaign",
        NoteScopeKind::Character => "character",
    }
}

pub fn scope_kind_from_str(s: &str) -> Result<NoteScopeKind, ApiError> {
    match s {
        "module" => Ok(NoteScopeKind::Module),
        "setting" => Ok(NoteScopeKind::Setting),
        "campaign" => Ok(NoteScopeKind::Campaign),
        "character" => Ok(NoteScopeKind::Character),
        other => Err(ApiError::Internal(anyhow::anyhow!(
            "unknown scope_kind in database: {other}"
        ))),
    }
}

pub fn visibility_to_str(v: NoteVisibility) -> &'static str {
    match v {
        NoteVisibility::Visible => "visible",
        NoteVisibility::AuthorOnly => "author_only",
        NoteVisibility::GamemasterOnly => "gamemaster_only",
    }
}

pub fn visibility_from_str(s: &str) -> Result<NoteVisibility, ApiError> {
    match s {
        "visible" => Ok(NoteVisibility::Visible),
        "author_only" => Ok(NoteVisibility::AuthorOnly),
        "gamemaster_only" => Ok(NoteVisibility::GamemasterOnly),
        other => Err(ApiError::Internal(anyhow::anyhow!(
            "unknown visibility in database: {other}"
        ))),
    }
}

fn parse_uuid(s: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(s).map_err(|e| ApiError::Internal(e.into()))
}
