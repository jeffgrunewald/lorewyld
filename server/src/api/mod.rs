pub mod admin;
pub mod admin_modules;
pub mod auth;
pub mod characters;
pub mod compendium;
pub mod error;
pub mod lore_notes;
pub mod modules;
pub mod rows;
pub mod search;
pub mod server_info;
pub mod settings;
pub mod tags;

use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use leptos::config::Env;
use leptos::prelude::LeptosOptions;
use leptos_axum::{LeptosRoutes, generate_route_list};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::info;

use crate::web::InstanceName;

pub struct ApiServer {
    listen_addr: SocketAddr,
    db: SqlitePool,
    leptos: LeptosOptions,
    instance_name: InstanceName,
}

#[derive(Clone)]
pub struct ApiState {
    pub db: SqlitePool,
}

impl ApiServer {
    pub fn new(db: SqlitePool, listen_addr: SocketAddr, instance_name: InstanceName) -> Self {
        let leptos = LeptosOptions::builder()
            .output_name("lorewyld")
            .site_root(".")
            .env(Env::DEV)
            .site_addr(listen_addr)
            .build();
        Self {
            listen_addr,
            leptos,
            db,
            instance_name,
        }
    }

    pub async fn run(self, shutdown: triggered::Listener) -> Result<()> {
        let leptos = self.leptos.clone();
        let instance_name = self.instance_name.clone();
        let routes = generate_route_list(crate::web::app::App);

        let api_state = ApiState::new(self.db);

        // Two subrouters, each owning its own state, then state-erased
        // to `Router<()>` via `with_state(...)`. Composing the
        // two via `merge` on a root `Router<()>` keeps the state types
        // separate without forcing a single composite state struct.
        let api_router: Router<()> = Router::new()
            .route("/api/server-info", get(server_info::server_info))
            .route("/api/users/register", post(auth::register))
            .route("/api/users/login", post(auth::login))
            .route("/api/users/logout", post(auth::logout))
            .route("/api/users/me", get(auth::me))
            .route("/api/users/password", post(auth::change_password))
            .route(
                "/api/admin/users",
                get(admin::list_users).post(admin::create_user),
            )
            .route(
                "/api/admin/users/{uuid}",
                axum::routing::delete(admin::delete_user).patch(admin::set_admin),
            )
            .route(
                "/api/admin/server",
                get(admin::get_server_settings).patch(admin::update_server_settings),
            )
            .route(
                "/api/admin/server/join-code",
                post(admin::regenerate_join_code),
            )
            .route("/api/admin/modules", get(admin_modules::list_modules))
            .route(
                "/api/admin/modules/install",
                post(admin_modules::install_module)
                    // The body is a whole ContentBundle package; the
                    // embedded SRD bundle alone is ~24MB.
                    .layer(axum::extract::DefaultBodyLimit::max(64 * 1024 * 1024)),
            )
            .route(
                "/api/admin/modules/{uuid}",
                axum::routing::patch(admin_modules::update_module_status)
                    .delete(admin_modules::uninstall_module),
            )
            .route("/api/content/counts", get(compendium::content_counts))
            .route("/api/content/recent", get(compendium::recent_content))
            .route("/api/content/{category}", get(compendium::list_category))
            .route("/api/content/{category}/{uuid}", get(compendium::get_entry))
            .route(
                "/api/characters",
                get(characters::list_characters).post(characters::create_character),
            )
            .route(
                "/api/characters/{uuid}",
                get(characters::get_character)
                    .put(characters::replace_character)
                    .delete(characters::delete_character),
            )
            .route(
                "/api/lore-notes",
                get(lore_notes::list_lore_notes).post(lore_notes::create_lore_note),
            )
            .route(
                "/api/lore-notes/{uuid}",
                get(lore_notes::get_lore_note)
                    .patch(lore_notes::update_lore_note)
                    .delete(lore_notes::delete_lore_note),
            )
            .route("/api/tags", get(tags::list_tags).post(tags::create_tag))
            .route(
                "/api/settings",
                get(settings::list_settings).post(settings::create_setting),
            )
            .route(
                "/api/settings/{uuid}",
                get(settings::get_setting)
                    .patch(settings::update_setting)
                    .delete(settings::delete_setting),
            )
            .route(
                "/api/settings/{uuid}/collaborators",
                post(settings::add_collaborator),
            )
            .route(
                "/api/settings/{uuid}/collaborators/{collab_uuid}",
                axum::routing::delete(settings::remove_collaborator),
            )
            .route("/api/search", post(search::search))
            .route(
                "/api/modules",
                get(modules::list_modules).post(modules::publish_module),
            )
            .route("/api/modules/{uuid}", get(modules::get_module))
            .with_state(api_state);

        let style_version = crate::web::StyleVersion::from_asset_mtime("assets/style.css");
        let script_version = crate::web::StyleVersion::from_asset_mtime("assets/lw-content.js");
        let leptos_router: Router<()> = Router::new()
            .nest_service("/assets", ServeDir::new("assets"))
            .leptos_routes(&leptos, routes, {
                let leptos = leptos.clone();
                let instance_name = instance_name.clone();
                move || {
                    crate::web::app::shell(
                        leptos.clone(),
                        instance_name.clone(),
                        style_version.clone(),
                        script_version.clone(),
                    )
                }
            })
            .with_state(leptos);

        let router = Router::new()
            .route("/health", get(health))
            .merge(api_router)
            .merge(leptos_router);

        let listener = TcpListener::bind(self.listen_addr)
            .await
            .expect("Failed to bind tcp address");

        info!(addr = %self.listen_addr, "API server & web listening");

        axum::serve(listener, router.into_make_service())
            .with_graceful_shutdown(shutdown)
            .await
            .map_err(anyhow::Error::from)
    }
}

impl ApiState {
    fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

async fn health() -> &'static str {
    "OK"
}
