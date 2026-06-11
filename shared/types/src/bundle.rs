use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    AbilityScoreEntry, Alignment, Armor, Background, CharacterFeature, Class, Condition,
    ContentModule, DamageType, Document, Enchantment, EquipmentCategory, EquipmentMaterial,
    EquipmentProficiency, EquipmentSubcategory, Feat, Language, MagicItem, MagicItemCategory,
    Monster, MonsterAction, MonsterType, Plane, Race, RacialTrait, SchemaVersion, Spell, SpellList,
    SpellSchool, Subclass, Weapon,
};

/// A complete, self-describing import package.
///
/// A bundle pairs a `SchemaVersion` envelope with one or more
/// `ContentModule`s and every record that belongs to those modules. The
/// server verifies `schema.version` against its compiled `SCHEMA_VERSION`
/// before persisting any rows.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ContentBundle {
    pub schema: SchemaVersion,
    pub modules: Vec<ContentModule>,
    #[serde(default)]
    pub documents: Vec<Document>,
    #[serde(default)]
    pub ability_scores: Vec<AbilityScoreEntry>,
    #[serde(default)]
    pub alignments: Vec<Alignment>,
    #[serde(default)]
    pub damage_types: Vec<DamageType>,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    #[serde(default)]
    pub languages: Vec<Language>,
    #[serde(default)]
    pub planes: Vec<Plane>,
    #[serde(default)]
    pub spell_schools: Vec<SpellSchool>,
    #[serde(default)]
    pub spell_lists: Vec<SpellList>,
    #[serde(default)]
    pub spells: Vec<Spell>,
    #[serde(default)]
    pub monster_types: Vec<MonsterType>,
    #[serde(default)]
    pub monster_actions: Vec<MonsterAction>,
    #[serde(default)]
    pub monsters: Vec<Monster>,
    #[serde(default)]
    pub classes: Vec<Class>,
    #[serde(default)]
    pub subclasses: Vec<Subclass>,
    #[serde(default)]
    pub races: Vec<Race>,
    #[serde(default)]
    pub racial_traits: Vec<RacialTrait>,
    #[serde(default)]
    pub feats: Vec<Feat>,
    #[serde(default)]
    pub backgrounds: Vec<Background>,
    #[serde(default)]
    pub equipment_categories: Vec<EquipmentCategory>,
    #[serde(default)]
    pub equipment_subcategories: Vec<EquipmentSubcategory>,
    #[serde(default)]
    pub equipment_proficiencies: Vec<EquipmentProficiency>,
    #[serde(default)]
    pub equipment_materials: Vec<EquipmentMaterial>,
    #[serde(default)]
    pub weapons: Vec<Weapon>,
    #[serde(default)]
    pub armors: Vec<Armor>,
    #[serde(default)]
    pub magic_item_categories: Vec<MagicItemCategory>,
    #[serde(default)]
    pub magic_items: Vec<MagicItem>,
    #[serde(default)]
    pub enchantments: Vec<Enchantment>,
    #[serde(default)]
    pub character_features: Vec<CharacterFeature>,
}
