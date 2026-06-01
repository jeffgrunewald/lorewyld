use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{EntityId, Timestamp};

/// Per-server user identity used for attribution and authoring access.
///
/// V1 ships without passwords — registration is gated only by the
/// server's `join_code`. The `display_name` is unique within a server
/// and acts as the de facto username for login. Future tiers may add
/// credentials, federated identity, or cross-server account linkage;
/// any such evolution is additive on top of this row.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppUser {
    pub uuid: EntityId,
    pub server_uuid: EntityId,
    pub display_name: String,
    pub created_at: Timestamp,
}

/// Opaque session token issued at registration / login.
///
/// Clients send the token via `Authorization: Bearer <token>` on every
/// authenticated request; the server resolves it to a `user_uuid` and
/// uses that for `created_by_user_uuid` attribution + ownership checks.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSession {
    pub token: String,
    pub user_uuid: EntityId,
    pub created_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<Timestamp>,
}
