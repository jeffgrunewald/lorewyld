use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{EntityId, Rarity, Timestamp};

/// Discriminator for which equipment families an enchantment can be
/// applied to.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnchantmentSubject {
    Weapon,
    Armor,
    Shield,
}

/// An enchantment suffix applied to a base item to create a magic
/// item (+1, +2, +3, Bane, etc.).
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enchantment {
    pub uuid: EntityId,
    pub content_module_uuid: EntityId,
    pub name: String,
    pub slug: String,
    #[serde(rename = "type")]
    pub subject: EnchantmentSubject,
    pub rarity: Rarity,
    pub description: String,
    pub attunement_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attunement_requirement: Option<String>,
    /// Numeric +N value applied to attack/damage/AC. 0 for non-plus
    /// enchantments (Bane, Vorpal, etc.).
    #[serde(default)]
    pub bonus_value: i32,
    pub is_restricted: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
