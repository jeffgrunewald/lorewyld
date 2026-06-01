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
    pub desc: String,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
