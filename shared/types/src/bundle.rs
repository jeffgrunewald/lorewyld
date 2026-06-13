use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    AbilityScoreEntry, Alignment, Armor, Background, Class, Condition, ContentModule, Creature,
    CreatureType, DamageType, Document, Environment, Feat, Item, ItemCategory, Language, License,
    Publisher, SchemaVersion, Size, Skill, Species, Spell, SpellSchool, Weapon, WeaponPropertyDef,
};

/// A complete, self-describing import package.
///
/// A bundle pairs a `SchemaVersion` envelope with one or more
/// `ContentModule`s and every record that belongs to those modules. The
/// server verifies `schema.version` against its compiled `SCHEMA_VERSION`
/// before persisting any rows.
///
/// Field order is import-dependency order: importers that insert
/// sequentially never reference a row that hasn't landed yet.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ContentBundle {
    pub schema: SchemaVersion,
    pub modules: Vec<ContentModule>,
    #[serde(default)]
    pub licenses: Vec<License>,
    #[serde(default)]
    pub publishers: Vec<Publisher>,
    #[serde(default)]
    pub documents: Vec<Document>,
    #[serde(default)]
    pub ability_scores: Vec<AbilityScoreEntry>,
    #[serde(default)]
    pub skills: Vec<Skill>,
    #[serde(default)]
    pub alignments: Vec<Alignment>,
    #[serde(default)]
    pub damage_types: Vec<DamageType>,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    #[serde(default)]
    pub languages: Vec<Language>,
    #[serde(default)]
    pub sizes: Vec<Size>,
    #[serde(default)]
    pub environments: Vec<Environment>,
    #[serde(default)]
    pub spell_schools: Vec<SpellSchool>,
    #[serde(default)]
    pub creature_types: Vec<CreatureType>,
    #[serde(default)]
    pub item_categories: Vec<ItemCategory>,
    #[serde(default)]
    pub weapon_properties: Vec<WeaponPropertyDef>,
    #[serde(default)]
    pub spells: Vec<Spell>,
    #[serde(default)]
    pub creatures: Vec<Creature>,
    #[serde(default)]
    pub classes: Vec<Class>,
    #[serde(default)]
    pub species: Vec<Species>,
    #[serde(default)]
    pub feats: Vec<Feat>,
    #[serde(default)]
    pub backgrounds: Vec<Background>,
    #[serde(default)]
    pub weapons: Vec<Weapon>,
    #[serde(default)]
    pub armors: Vec<Armor>,
    #[serde(default)]
    pub items: Vec<Item>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::SCHEMA_VERSION;

    #[test]
    fn empty_bundle_round_trips() {
        let bundle = ContentBundle::default();
        let json = serde_json::to_string(&bundle).unwrap();
        let back: ContentBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(bundle, back);
        assert_eq!(back.schema.version, SCHEMA_VERSION);
    }

    #[test]
    fn partial_bundle_json_fills_defaults() {
        // A spells-only pack omits every other table.
        let json = format!(
            r#"{{"schema":{{"version":{SCHEMA_VERSION},"min_supported":{SCHEMA_VERSION}}},"modules":[]}}"#
        );
        let bundle: ContentBundle = serde_json::from_str(&json).unwrap();
        assert!(bundle.spells.is_empty());
        assert!(bundle.creatures.is_empty());
        assert!(bundle.items.is_empty());
    }
}
