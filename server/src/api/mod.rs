use anyhow::Result;
use axum::{routing::get, Router};
use leptos::config::Env;
use leptos::prelude::LeptosOptions;
use leptos_axum::{generate_route_list, LeptosRoutes};
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
        let _state = ApiState::new(self.db);
        let leptos = self.leptos.clone();
        let instance_name = self.instance_name.clone();
        let routes = generate_route_list(crate::web::app::App);

        let router = Router::new()
            .route("/health", get(health))
            .nest_service("/assets", ServeDir::new("assets"))
            .leptos_routes(&leptos, routes, {
                let leptos = leptos.clone();
                let instance_name = instance_name.clone();
                move || crate::web::app::shell(leptos.clone(), instance_name.clone())
            })
            .with_state(leptos);

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
