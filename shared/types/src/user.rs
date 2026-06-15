use serde::{Deserialize, Serialize};

use crate::common::{EntityId, Timestamp};

/// Server-instance user account: credentials, attribution, and admin
/// access rolled into one identity. Registration is gated by the
/// server's `join_code`; passwords are stored as argon2 PHC hashes.
/// `admin` unlocks server settings and user management.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct User {
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub uuid: EntityId,
    pub username: String,
    pub email: String,
    pub admin: bool,
    #[cfg_attr(feature = "openapi", schema(value_type = String, format = DateTime))]
    pub created_at: Timestamp,
}

/// Opaque session token issued at registration / login.
///
/// Clients send the token via `Authorization: Bearer <token>` on every
/// authenticated request; the server resolves it to a `user_uuid` and
/// uses that for `created_by_user_uuid` attribution + ownership checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSession {
    pub token: String,
    pub user_uuid: EntityId,
    pub created_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<Timestamp>,
}
