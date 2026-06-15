//! OpenAPI document for the Lorewyld HTTP API.
//!
//! Schemas are derived (via the `openapi` feature) from the shared
//! `lorewyld-types` crate, so the published contract tracks the same Rust
//! types the server, mobile, and web clients use. Served at
//! `/api/openapi.json` with a Swagger UI at `/swagger-ui`.

use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

use lorewyld_types::{AuthResponse, LoginRequest, RegisterRequest, User};

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
        crate::api::auth::register,
        crate::api::auth::login,
        crate::api::auth::logout,
        crate::api::auth::me,
    ),
    components(schemas(RegisterRequest, LoginRequest, AuthResponse, User)),
    modifiers(&BearerSecurity),
    tags((name = "auth", description = "Registration, login, and session lifecycle.")),
)]
pub struct ApiDoc;
