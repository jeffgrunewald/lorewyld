use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{ConditionName, EntityId, Timestamp};

/// A standard SRD condition with mechanical effects on creatures.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Condition {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: ConditionName,
    pub slug: String,
    /// Stable external identifier (Open5e key for imported content).
    pub key: String,
    /// Description matching the module's game edition (Open5e v2 ships
    /// multi-edition descriptions; the generator selects one).
    pub desc: String,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
