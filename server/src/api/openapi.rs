//! OpenAPI document for the Lorewyld HTTP API.
//!
//! Schemas are derived (via the `openapi` feature) from the shared
//! `lorewyld-types` crate, so the published contract tracks the same Rust
//! types the server, mobile, and web clients use. Served at
//! `/api/openapi.json` with a Swagger UI at `/swagger-ui`.
//!
//! Two endpoints are intentionally documented without a body schema:
//! `GET /api/content/{category}` (per-category typed summaries — the row
//! shape is the category's `XSummary`) and `POST /api/admin/modules/install`
//! (a full `ContentBundle`, which would pull the entire content type graph
//! into the spec). Both are described in prose on their operations.

use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

/// Registers the `bearer` security scheme (`Authorization: Bearer <token>`).
struct BearerSecurity;

impl Modify for BearerSecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("opaque session token")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Lorewyld API",
        description = "HTTP API for the Lorewyld server. Request/response types are generated from the shared Rust schema crate (lorewyld-types).",
    ),
    paths(
        crate::api::server_info::server_info,
        crate::api::auth::register,
        crate::api::auth::login,
        crate::api::auth::logout,
        crate::api::auth::me,
        crate::api::auth::change_password,
        crate::api::admin::list_users,
        crate::api::admin::create_user,
        crate::api::admin::delete_user,
        crate::api::admin::set_admin,
        crate::api::admin::get_server_settings,
        crate::api::admin::update_server_settings,
        crate::api::admin::regenerate_join_code,
        crate::api::admin_modules::list_modules,
        crate::api::admin_modules::install_module,
        crate::api::admin_modules::update_module_status,
        crate::api::admin_modules::uninstall_module,
        crate::api::compendium::content_counts,
        crate::api::compendium::recent_content,
        crate::api::compendium::list_category,
        crate::api::compendium::get_entry,
        crate::api::characters::list_characters,
        crate::api::characters::create_character,
        crate::api::characters::get_character,
        crate::api::characters::replace_character,
        crate::api::characters::delete_character,
        crate::api::lore_notes::list_lore_notes,
        crate::api::lore_notes::create_lore_note,
        crate::api::lore_notes::get_lore_note,
        crate::api::lore_notes::update_lore_note,
        crate::api::lore_notes::delete_lore_note,
        crate::api::tags::list_tags,
        crate::api::tags::create_tag,
        crate::api::settings::list_settings,
        crate::api::settings::create_setting,
        crate::api::settings::get_setting,
        crate::api::settings::update_setting,
        crate::api::settings::delete_setting,
        crate::api::settings::add_collaborator,
        crate::api::settings::remove_collaborator,
        crate::api::search::search,
        crate::api::modules::list_modules,
        crate::api::modules::get_module,
        crate::api::modules::publish_module,
    ),
    components(schemas(
        lorewyld_types::User,
        lorewyld_types::RegisterRequest,
        lorewyld_types::LoginRequest,
        lorewyld_types::AuthResponse,
        lorewyld_types::ChangePasswordRequest,
        lorewyld_types::AdminCreateUserRequest,
        lorewyld_types::AdminUpdateUserRequest,
        lorewyld_types::UserListResponse,
        lorewyld_types::ServerSettings,
        lorewyld_types::UpdateServerSettingsRequest,
        lorewyld_types::GameServerSummary,
        lorewyld_types::ServerInfo,
        lorewyld_types::ContentModule,
        lorewyld_types::LicenseKind,
        lorewyld_types::LoreNote,
        lorewyld_types::LoreNoteWithTags,
        lorewyld_types::NoteScope,
        lorewyld_types::NoteScopeKind,
        lorewyld_types::NoteVisibility,
        lorewyld_types::CreateLoreNoteRequest,
        lorewyld_types::UpdateLoreNoteRequest,
        lorewyld_types::Tag,
        lorewyld_types::CreateTagRequest,
        lorewyld_types::Setting,
        lorewyld_types::SettingCollaborator,
        lorewyld_types::CreateSettingRequest,
        lorewyld_types::UpdateSettingRequest,
        lorewyld_types::AddCollaboratorRequest,
        lorewyld_types::SearchRequest,
        lorewyld_types::SearchResponse,
        lorewyld_types::PublishModuleRequest,
        lorewyld_types::PublishModuleResponse,
        lorewyld_types::ModuleOrigin,
        lorewyld_types::CategoryCount,
        lorewyld_types::AdminModuleSummary,
        lorewyld_types::UpdateModuleStatusRequest,
        lorewyld_types::InstallModuleResponse,
        lorewyld_types::ContentCountsResponse,
        lorewyld_types::RecentContentItem,
        lorewyld_types::RecentContentResponse,
        lorewyld_types::CharacterSheet,
        lorewyld_types::CharacterEquipmentItem,
        lorewyld_types::CharacterSpellEntry,
        lorewyld_types::AbilityScores,
        crate::api::modules::ModuleWithNotes,
    )),
    modifiers(&BearerSecurity),
    tags(
        (name = "server", description = "Server discovery."),
        (name = "auth", description = "Registration, login, and session lifecycle."),
        (name = "content", description = "Read-only compendium of installed SRD content."),
        (name = "characters", description = "Character-sheet CRUD."),
        (name = "lore-notes", description = "Worldbuilding/campaign lore notes."),
        (name = "tags", description = "Tag vocabulary."),
        (name = "settings", description = "Setting workspaces and collaborators."),
        (name = "search", description = "Full-text + tag/scope search."),
        (name = "modules", description = "Published content modules."),
        (name = "admin", description = "Operator-only: users, server settings, module lifecycle."),
    ),
)]
pub struct ApiDoc;
