# DND 5e SRD Data Model

All data follows the System Reference Document (SRD) for D&D 5th edition,
licensed under CC BY 4.0. Content attribution flows through the `ContentModule` schema.

---

## Schema: content_module

Centrally ties together content packs and provides resource attribution. The UUID
is the foreign key used by all other schemas to reference the source content pack.

```json
{
  "name": "content_module",
  "description": "Defines a content pack (source book, supplement, or homebrew set) ",
  "schema": {
    "table": "content_modules",
    "uuid": "STRING (PK, UUID v4)",
    "name": "STRING (required) — Human-readable name, e.g., Player's Handbook",
    "slug": "STRING (unique) — Machine-readable identifier, e.g., players-handbook",
    "license": "STRING (required) — e.g., CC-BY-4.0, OGL-1.0a, proprietary",
    "license_url": "STRING (optional) — URL to the full license text",
    "release_date": "DATE (optional) — Publication date of the content pack",
    "authors": "ARRAY<STRING> — List of authors or contributors",
    "publisher": "STRING (optional) — Publishing entity",
    "description": "TEXT (optional) — Short description of the content pack scope",
    "website_url": "STRING (optional) — Official website or documentation",
    "is_active": "BOOLEAN (default true)",
    "ordering": "INTEGER (default 0) — Sort order in catalogs",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: spell

A magical spell as defined in the SRD, with casting parameters, schools, and effects.

```json
{
  "name": "spell",
  "description": "A magical spell with casting parameters, schools, and effects",
  "schema": {
    "table": "spells",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Spell name, e.g., Fireball",
    "slug": "STRING (unique, index)",
    "level": "INTEGER (0-9) — Spell level (0 = cantrip)",
    "school": "STRING (FK => spell_schools.uuid) — Abjuration, Conjuration, etc.",
    "ritual": "BOOLEAN (default false)",
    "duration": "STRING — e.g., Instantaneous, Concentration up to 1 minute",
    "casting_time": "STRING — e.g., 1 action, 1 bonus action",
    "range": "STRING — e.g., 60 feet, Self, Touch, Special",
    "components": "ARRAY<STRING> — Subset of [V, S, M]",
    "material_components": "STRING (optional) — Material component details",
    "target": "STRING — Target description",
    "description": "TEXT — Full spell description including effects",
    "higher_level": "TEXT (optional) — Effects when cast at higher levels",
    "scaling": "JSON (optional) — Structured scaling data for high-level casting",
    "concentration": "BOOLEAN",
    "requires_verbal": "BOOLEAN",
    "requires_somatic": "BOOLEAN",
    "requires_material": "BOOLEAN",
    "classes": "ARRAY<STRING> — Classes that cast this spell",
    "subclasses": "ARRAY<STRING> — Subclasses that gain access",
    "spell_lists": "ARRAY<STRING> — Spell lists this spell appears on",
    "levels_by_class": "JSON (optional) — Bard: [2], Sorcerer: [2], Wizard: [3]",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: monster

A creature statistic block from the SRD with statistics, abilities, actions, and spellcasting.

```json
{
  "name": "monster",
  "description": "A creature/statistic block with stats, abilities, actions, spellcasting",
  "schema": {
    "table": "monsters",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "type": "STRING (FK => monster_types.name) — Aberration, Beast, Celestial, etc.",
    "subtype": "STRING (optional) — e.g., Shifter, Reptilian",
    "name": "STRING (required)",
    "slug": "STRING (unique, index)",
    "size": "STRING — Tiny, Small, Medium, Large, Huge, Gargantuan",
    "alignment": "STRING (FK => alignments.uuid, optional)",
    "armor_class": "INTEGER",
    "armor_type": "STRING (optional)",
    "hit_points": "INTEGER",
    "hit_dice": "STRING — e.g., 33 (6d8)",
    "speed": "JSON — walk: 40, fly: 0, swim: 30, climb: 20, burrow: 10 (feet)",
    "ability_scores": "JSON — strength, dexterity, constitution, intelligence, wisdom, charisma (base values, not modifiers)",
    "ability_modifiers": "JSON — str: 0, dex: +1, con: +0, etc.",
    "saving_throws": "JSON (optional) — str: +2, dex: +4",
    "skills": "JSON (optional) — Athletics: +6, Perception: +3, Stealth: +4",
    "damage_resistances": "ARRAY<STRING> — FK refs to damage_types",
    "damage_vulnerabilities": "ARRAY<STRING> — FK refs to damage_types",
    "damage_immunities": "ARRAY<STRING> — FK refs to damage_types",
    "condition_immunities": "ARRAY<STRING> — FK refs to conditions.name",
    "senses": "STRING — e.g., darkvision 60 ft., passive Perception 10",
    "senses_detail": "JSON — darkvision: 60, blindsight: 0, tremorsense: 0, truesight: 0",
    "languages": "STRING — comma-separated list",
    "languages_list": "ARRAY<STRING> — FK refs to languages",
    "challenge_rating": "INTEGER",
    "xp": "INTEGER — XP award for defeating this monster",
    "actions": "JSON (array) — Action objects with name, desc, damage, etc.",
    "bonus_actions": "JSON (array) — Bonus action objects",
    "reactions": "JSON (array) — Reaction objects",
    "legendary_actions": "JSON (array) — Legendary action objects",
    "lair_actions": "JSON (array, optional) — Lair-specific actions",
    "legendary_desc": "TEXT (optional)",
    "special_abilities": "JSON (array) — Special/unique abilities",
    "spellcasting": "JSON (optional, false if none) — Spellcasting stat block",
    "spellcasting_level": "INTEGER (optional) — 0 if innate",
    "spellcasting_spells_per_day": "JSON (optional)",
    "spellcasting_spells": "ARRAY — spell entries with level and count",
    "spellcasting_ability": "STRING (optional) — Charisma, Wisdom, Intelligence",
    "spellcasting_bonus": "INTEGER (optional)",
    "spell_save_dc": "INTEGER (optional)",
    "innate_spellcasting": "JSON (optional) — Always-on spells",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: monster_type

A classification of monster type for lookup and cross-referencing.

```json
{
  "name": "monster_type",
  "description": "Classification of monster type (Aberration, Beast, etc.)",
  "schema": {
    "table": "monster_types",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required, enum) — Aberration, Beast, Celestial, Construct, Dragon, Fey, Fiend, Giant, Humanoid, Monstrosity, Ooze, Plant, Undead",
    "slug": "STRING (unique, index)",
    "description": "STRING (optional)",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: monster_action

A single action type that can be shared and reused across monsters (Multiattack, Longsword, etc.).

```json
{
  "name": "monster_action",
  "description": "Reusable action, reaction, or legendary action definitions",
  "schema": {
    "table": "monster_actions",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Multiattack, Longsword, etc.",
    "slug": "STRING (unique, index)",
    "type": "STRING — action, bonus_action, reaction, legendary, lair",
    "desc": "TEXT — Full description",
    "attack_bonus": "INTEGER (optional)",
    "reach": "INTEGER (optional) — Feet",
    "target": "STRING (optional)",
    "damage": "JSON (optional, false) — dice: 1d8, plus_mod: 3, type: slashing",
    "save_dc": "INTEGER (optional)",
    "save_type": "STRING (optional) — Dexterity, Constitution",
    "save_detail": "STRING (optional) — half on success",
    "uses_per_round": "INTEGER (optional) — For legendary actions, e.g., 3",
    "recharge_on_roll": "INTEGER (optional) — On roll of 5-6",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: class

A character class with hit dice, proficiencies, feature table, archetype support, and optional spellcasting.

```json
{
  "name": "class",
  "description": "Character class with hit dice, proficiencies, features, archetypes, and spellcasting",
  "schema": {
    "table": "classes",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Barbarian, Bard, Cleric, etc.",
    "slug": "STRING (unique, index)",
    "hit_dice": "INTEGER — Die size: 6=d6, 8=d8, 10=d10, 12=d12",
    "hp_at_1st_level": "STRING — e.g., 12 hit points",
    "hp_at_higher_levels": "STRING — e.g., 2 (1d6) hit points per level after 1st",
    "prof_armor": "STRING — Armor proficiencies",
    "prof_weapons": "STRING — Weapon proficiencies",
    "prof_tools": "STRING (optional) — Tool proficiencies",
    "prof_saving_throws": "STRING — Saving throw proficiencies",
    "prof_skills": "JSON — choose: 2, from: [Animal Handling, Athletics, Intimidation, Nature, Perception, Survival]",
    "equipment": "JSON — starting equipment options with choices",
    "feature_table": "JSON — Full progression table keyed by character level (1-20):",
    "feature_table.proficiency_bonus": "INTEGER[] — [+2,+2,+2,+2,+2,+2,+2,+2,+3,+3,+3,+3,+3,+3,+3,+3,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+4,+5,+5,+5,+5,+5,+5,+5,+5,+5,+5,+5,+5,+5,+5,+5,+5,+5,+5,+5,+6,+6,+6,+6,+6,+6,+6,+6,+6,+6]",
    "feature_table.level": "INTEGER[] — [1,2,3,...,20]",
    "feature_table.features": "STRING[] — Feature names at each level",
    "feature_table.spells_known": "INTEGER[] — Spells known at each level (for casting classes)",
    "feature_table.cantrips_known": "INTEGER[] — Cantrips known at each level",
    "feature_table.slot_levels": "INTEGER[] — Total slot level available at each level",
    "feature_table.slots_by_level": "JSON — slots per spell level at each character level (slots_1: [], slots_2: [], etc.)",
    "spellcasting_ability": "STRING (optional) — STR, DEX, CON, INT, WIS, or CHA. Null for non-spellcasting classes",
    "spellcasting_table": "JSON (optional, for Pact Magic classes) — Spells known per level",
    "subtypes_name": "STRING (required) — Subclass group name, e.g., Archetype, Divine Domain, Pact Boon",
    "archetype_table": "JSON (optional) — Subclass feature progression keyed by level",
    "archetype_table.feature_names": "STRING[] — Feature names per level",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: subclass

A subclass (Archetype, Domain, Path, Pact, etc.) within a class.

```json
{
  "name": "subclass",
  "description": "Subclass/Archetype/Domain/Path within a class",
  "schema": {
    "table": "subclasses",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Path of the Berserker, Life Domain, etc.",
    "slug": "STRING (unique, index)",
    "parent_class_uuid": "STRING (FK => classes.uuid)",
    "parent_class": "STRING — Display name, e.g., Barbarian",
    "level": "INTEGER — Usually 3",
    "feature_table": "JSON — Subclass features keyed by character level",
    "features": "JSON — { level: { name: Feature Name, desc: Text } }",
    "spell_list": "JSON (optional) — Spells gained at each level for casting subclasses",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: race

A character race/species with ability score increases, speed, traits, vision, and languages.

```json
{
  "name": "race",
  "description": "Character race with ASIs, speed, traits, vision, and languages",
  "schema": {
    "table": "races",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Human, Elf, Dwarf, etc.",
    "slug": "STRING (unique, index)",
    "desc": "TEXT — Race description",
    "asi_values": "JSON — strength: -1, dexterity: 2, constitution: 0, intelligence: 0, wisdom: 0, charisma: 0",
    "asi_desc": "STRING — e.g., +2 DEX",
    "asi_source": "ARRAY<JSON> — For races with variable ASIs: [{ source: Customization, values: { all: 1 } }]",
    "age": "STRING — e.g., Adulthood around 100 years, average 520 years",
    "alignment": "STRING — e.g., Usually chaotic good",
    "size": "STRING — Small or Medium",
    "size_raw": "STRING — Original SRD source text",
    "speed": "INTEGER — Walking speed in feet",
    "speed_desc": "STRING — e.g., 30 feet",
    "languages_base": "ARRAY<STRING> — Required languages, FK refs to languages",
    "languages_additional": "JSON (optional) — { count: 1, from: languages_list }",
    "vision_base": "JSON (optional) — darkvision: 60, blindsight: 0, tremorsense: 0, truesight: 0",
    "trait_names": "ARRAY<STRING> — Darkvision, Fey Ancestry, Trance",
    "trait_definitions": "JSON — Trait descriptions keyed by name",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: racial_trait

A single reusable racial trait definition shared across races (Darkvision, Fey Ancestry, Trance).

```json
{
  "name": "racial_trait",
  "description": "Single reusable racial trait definition",
  "schema": {
    "table": "racial_traits",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Darkvision, Fey Ancestry, Trance, etc.",
    "slug": "STRING (unique, index)",
    "description": "TEXT — Full trait description",
    "category": "STRING — vision, ancestry, mortality, resistance, etc.",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: feat

A character feat providing abilities.

```json
{
  "name": "feat",
  "description": "Character feat providing abilities",
  "schema": {
    "table": "feats",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Alert, War Caster, Resilient, etc.",
    "slug": "STRING (unique, index)",
    "desc": "TEXT — Full feat description",
    "prerequisite": "STRING (optional) — None, STR 13 or higher, etc.",
    "effects": "ARRAY<JSON> — Distinct effects: { name: Feature Name, desc: Text }",
    "ability_score_bump": "JSON (optional) — { strength: 1, dexterity: 1 }",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: background

A character background with skills, proficiencies, languages, equipment, and personality features.

```json
{
  "name": "background",
  "description": "Character background with skills, proficiencies, languages, equipment, features",
  "schema": {
    "table": "backgrounds",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Acolyte, Criminal, Soldier, etc.",
    "slug": "STRING (unique, index)",
    "desc": "TEXT — Full background description",
    "skill_proficiencies": "JSON — e.g., { choose: 1, from: [History, Investigation] }",
    "tool_proficiencies": "JSON — e.g., { choose: 1, from: Two of your choice }",
    "languages": "JSON (optional) — e.g., { choose: 1, from: Two of your choice }",
    "equipment": "ARRAY<STRING> — Starting equipment items",
    "feature_name": "STRING — Feature name, e.g.,神之仆, Criminal's Contact",
    "feature_desc": "STRING — Full feature description",
    "personality_traits": "ARRAY<STRING>",
    "ideals": "ARRAY<STRING>",
    "bonds": "ARRAY<STRING>",
    "flaws": "ARRAY<STRING>",
    "proficiency_names": "ARRAY<STRING> — All proficiencies granted",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: condition

A standard condition that affects creatures mechanically.

```json
{
  "name": "condition",
  "description": "Standard condition with mechanical effects on creatures",
  "schema": {
    "table": "conditions",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required, enum) — Blinded, Charmed, Deafenened, Frightened, Grappled, Incapacitated, Invisible, Paralyzed, Petrified, Poisoned, Prone, Restrained, Stunned, Unconscious",
    "slug": "STRING (unique, index)",
    "desc": "STRING — Full mechanical description of the condition",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: plane

A cosmological plane of existence within the D&D multiverse.

```json
{
  "name": "plane",
  "description": "Cosmological plane of existence",
  "schema": {
    "table": "planes",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Abyss, Astral Plane, Elemental Plane of Fire, etc.",
    "slug": "STRING (unique, index)",
    "desc": "STRING — Description of the plane",
    "plane_type": "STRING — Material Plane, Inner Plane, Outer Plane, Astral Plane, Ethereal Plane",
    "alignment_association": "STRING (optional) — Chaotic Evil, Lawful Good",
    "elemental_association": "STRING (optional) — Fire, Water, Earth, Air",
    "parent_uuid": "STRING (FK => plants.uuid, optional) — Hierarchical parent",
    "inhabitants": "ARRAY<STRING> — Creature types that inhabit this plane",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: magic_item

A magical item with powers, rarity, attunement requirements, and category.

```json
{
  "name": "magic_item",
  "description": "Magical item with powers, rarity, attunement, category",
  "schema": {
    "table": "magic_items",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Longsword +1, Cloak of Protection, etc.",
    "slug": "STRING (unique, index)",
    "type": "STRING — Armor, Potion, Ring, Rod, Scroll, Staff, Wand, Weapon (Any), Wondrous Item, Boots, Cloak, Gloves, Helm, Periapt, Stone, Amulet",
    "rarity": "STRING — Common, Uncommon, Rare, Very Rare, Legendary, Artifact",
    "rarity_roll": "JSON (optional) — { min: 1, max: 21, result: 11 }",
    "requires_attunement": "BOOLEAN",
    "attunement_requirement": "STRING (optional) — by a sorcerer, warlock, or wizard; by an elf; by a creature of good alignment",
    "attunement_by_class": "ARRAY<STRING> (optional)",
    "attunement_by_race": "ARRAY<STRING> (optional)",
    "attunement_by_alignment": "ARRAY<STRING> (optional)",
    "desc": "STRING — Full item description and powers",
    "properties": "JSON (optional) — Structured power entries",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: magic_item_category

A category grouping for magic items (Weapon, Armor, Potion, etc.).

```json
{
  "name": "magic_item_category",
  "description": "Category grouping for magic items",
  "schema": {
    "table": "magic_item_categories",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Weapon, Armor, Potion, Wondrous Item, etc.",
    "slug": "STRING (unique, index)",
    "desc": "STRING (optional)",
    "parent_uuid": "STRING (FK => magic_item_categories.uuid, optional) — Hierarchical parent",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: weapon

A weapon entry with damage dice, properties, cost, and category.

```json
{
  "name": "weapon",
  "description": "Weapon entry with damage, properties, cost, category",
  "schema": {
    "table": "weapons",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Longsword, Longbow, Glaive, etc.",
    "slug": "STRING (unique, index)",
    "category_uuid": "STRING (FK => equipment_categories.uuid)",
    "category_name": "STRING — Simple Melee Weapons, Martial Ranged Weapons, etc.",
    "cost": "DECIMAL — Gold piece cost",
    "damage_dice": "STRING — 1d6, 1d8, 1d10, 2d6, or 1d8/1d10 for versatile",
    "damage_type": "STRING (FK => damage_types.name) — Piercing, Slashing, Bludgeoning",
    "weight": "DECIMAL — Weight in pounds",
    "properties": "ARRAY<STRING> — Heavy, Light, Loading, Thrown (range/normal, range/long), Two-Handed, Versatile, Reach, Special",
    "damage_range": "STRING (optional) — Normal range / long range for thrown/ammunition weapons",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: armor

Armor with AC calculation, weight, cost, and stealth properties.

```json
{
  "name": "armor",
  "description": "Armor with AC calculation, weight, cost, stealth",
  "schema": {
    "table": "armors",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Chain Mail, Scale Mail, Studded Leather, etc.",
    "slug": "STRING (unique, index)",
    "category_uuid": "STRING (FK => equipment_categories.uuid)",
    "category_name": "STRING — Light Armor, Medium Armor, Heavy Armor, Shield",
    "base_ac": "INTEGER — Base AC provided",
    "plus_dex_mod": "BOOLEAN — Whether DEX modifier is added to AC",
    "plus_con_mod": "BOOLEAN — Constitution modifier bonus to AC",
    "plus_wis_mod": "BOOLEAN — Wisdom modifier bonus to AC or DEX max cap",
    "plus_flat_mod": "INTEGER — Flat AC bonus (e.g., +1 for +1 armor)",
    "plus_max": "INTEGER — Maximum DEX modifier that applies",
    "ac_string": "STRING — e.g., 16, 13 + Dex modifier (max 2), 18 + Dex modifier",
    "strength_requirement": "INTEGER (optional)" ,
    "cost": "DECIMAL — Gold piece cost",
    "weight": "DECIMAL — Weight in pounds (optional for some sources)",
    "stealth_disadvantage": "BOOLEAN",
    "is_heavy": "BOOLEAN",
    "is_medium": "BOOLEAN",
    "is_light": "BOOLEAN",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: equipment_category

A category for mundane equipment (Simple Weapons, Martial Weapons, Armor, etc.).

```json
{
  "name": "equipment_category",
  "description": "Category for mundane equipment",
  "schema": {
    "table": "equipment_categories",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Simple Melee Weapons, Martial Ranged Weapons, Light Armor, etc.",
    "slug": "STRING (unique, index)",
    "desc": "STRING (optional)",
    "parent_uuid": "STRING (FK => equipment_categories.uuid, optional) — Hierarchical parent",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: equipment_subcategory

A subcategory for granular equipment categorization.

```json
{
  "name": "equipment_subcategory",
  "description": "Subcategory for granular equipment categorization",
  "schema": {
    "table": "equipment_subcategories",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Simple Melee Weapons, Martial Ranged Weapons",
    "slug": "STRING (unique, index)",
    "category_uuid": "STRING (FK => equipment_categories.uuid)",
    "description": "STRING (optional)",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: alignment

One of the nine alignments on the lawful–neutral–chaotic and good–neutral–evil axes.

```json
{
  "name": "alignment",
  "description": "Nine alignments on the lawful-neutral-chaotic and good-neutral-evil axes",
  "schema": {
    "table": "alignments",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required, enum) — Lawful Good, Neutral Good, Chaotic Good, Lawful Neutral, True Neutral, Chaotic Neutral, Lawful Evil, Neutral Evil, Chaotic Evil",
    "slug": "STRING (unique, index)",
    "is_lawful": "BOOLEAN",
    "is_neutral": "BOOLEAN",
    "is_chaotic": "BOOLEAN",
    "is_good": "BOOLEAN",
    "is_evil": "BOOLEAN",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: ability_score

One of the six core ability scores with a derived modifier.

```json
{
  "name": "ability_score",
  "description": "One of the six core ability scores with derived modifier table",
  "schema": {
    "table": "ability_scores",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required, enum) — strength, dexterity, constitution, intelligence, wisdom, charisma",
    "slug": "STRING (unique, index) — STR, DEX, CON, INT, WIS, CHA",
    "short_name": "STRING — Two-letter abbreviation",
    "description": "STRING — What this ability score governs",
    "modifier_table": "JSON — Mapping of score values (3-30) to modifiers (-5 to +10)",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: damage_type

A damage type with resistances, immunities, and vulnerabilities.

```json
{
  "name": "damage_type",
  "description": "Damage type with resistances, immunities, vulnerabilities",
  "schema": {
    "table": "damage_types",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required, enum) — Acid, Bludgeoning, Cold, Fire, Force, Lightning, Necrotic, Piercing, Poison, Psychic, Radiant, Slashing, Thunder",
    "slug": "STRING (unique, index)",
    "description": "STRING",
    "resistances_against": "ARRAY<STRING> — Damage types that resist this type",
    "immunities_against": "ARRAY<STRING> — Damage types immune to this type",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: language

A language from the SRD with its script, speakers, and classification.

```json
{
  "name": "language",
  "description": "Language from the SRD with script, speakers, classification",
  "schema": {
    "table": "languages",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Common, Elvish, Dwarvish, Primordial, Abyssal, Infernal, Celestial, Draconic, Giant, Gnomish, Goblin, Halfling, Infernal, Infernal, Necrologue, Orcian, Primordial, Sylvan, Undercommon",
    "slug": "STRING (unique, index)",
    "script": "STRING — Dwarvish, Elvish, etc.",
    "region": "STRING (optional) — Core, Underdark, Abyss, etc.",
    "speakers": "ARRAY<STRING> — Humans, Elves, etc.",
    "is_independent": "BOOLEAN",
    "is_tonal": "BOOLEAN",
    "is_alphabetical": "BOOLEAN",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: spell_school

One of the eight schools of magic.

```json
{
  "name": "spell_school",
  "description": "One of the eight schools of magic",
  "schema": {
    "table": "spell_schools",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required, enum) — Abjuration, Conjuration, Divination, Enchantment, Evocation, Illusion, Necromancy, Transmutation",
    "slug": "STRING (unique, index)",
    "description": "STRING",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: spell_list

A named collection of spells tied to one or more classes.

```json
{
  "name": "spell_list",
  "description": "Named collection of spells tied to classes",
  "schema": {
    "table": "spell_lists",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Bard, Cleric, Druid, Ranger, Sorcerer, Warlock, Wizard, etc.",
    "slug": "STRING (unique, index)",
    "class": "STRING — The primary class associated with this spell list",
    "classes": "ARRAY<STRING> — All classes that use this spell list",
    "desc": "STRING (optional)",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: equipment_proficiency

A type of equipment proficiency grantable by classes, races, and backgrounds.

```json
{
  "name": "equipment_proficiency",
  "description": "Equipment proficiency grantable by classes, races, backgrounds",
  "schema": {
    "table": "equipment_proficiencies",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "type": "STRING — armor, weapon, shield, artisan_tool, musical_instrument, vehicle",
    "name": "STRING (required) — Light Armor, Simple Weapons, Disguise Kit, etc.",
    "slug": "STRING (unique, index)",
    "desc": "STRING (optional)",
    "category_uuid": "STRING (FK => equipment_categories.uuid)",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: enchantment

An enchantment suffix applied to a base item to create a magic item (+1, +2, +3, Bane, etc.).

```json
{
  "name": "enchantment",
  "description": "Enchantment suffix for magic item creation (+1, +2, +3, Bane, etc.)",
  "schema": {
    "table": "enchantments",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — +1, +2, +3, Bane vs. Dragons, etc.",
    "slug": "STRING (unique, index)",
    "type": "STRING — weapon, armor, shield",
    "rarity": "STRING — Uncommon, Rare, Very Rare",
    "description": "STRING — Effect description",
    "attunement_required": "BOOLEAN",
    "attunement_requirement": "STRING (optional)",
    "bonus_value": "INTEGER — How much the +N bonus applies to attack/damage/AC",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: equipment_material

A physical consumable component used in spellcasting, alchemy, or crafting.

```json
{
  "name": "equipment_material",
  "description": "Physical consumable for spellcasting, alchemy, crafting",
  "schema": {
    "table": "equipment_materials",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Holy Water, Antitoxin, Powder of Ironstone, etc.",
    "slug": "STRING (unique, index)",
    "cost": "DECIMAL (optional) — Cost per unit in gold pieces",
    "weight": "DECIMAL (optional) — Weight in pounds",
    "description": "STRING — Use and properties",
    "is_consumable": "BOOLEAN",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: character_feature

A general character feature spanning class, feat, background, and race.

```json
{
  "name": "character_feature",
  "description": "General character feature spanning class, feat, background, race",
  "schema": {
    "table": "character_features",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Darkvision, Expertise, Evasion, etc.",
    "slug": "STRING (unique, index)",
    "description": "TEXT — Full feature description",
    "category": "STRING — vision, expertise, resistance, ability boost, etc.",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## Schema: document

A source document or book that contains SRD content, used for attribution.

```json
{
  "name": "document",
  "description": "Source document for SRD attribution",
  "schema": {
    "table": "documents",
    "uuid": "STRING (PK, UUID v4)",
    "content_module_uuid": "STRING (FK => content_modules.uuid)",
    "name": "STRING (required) — Player's Handbook, Dungeon Master's Guide",
    "slug": "STRING (unique, index)",
    "desc": "TEXT (optional)",
    "title": "STRING (required)",
    "url": "STRING (optional)",
    "license": "STRING (required) — CC BY 4.0, OGL 1.0a",
    "author": "STRING (optional)",
    "organization": "STRING (optional)",
    "version": "STRING (optional)",
    "copyright": "STRING (optional)",
    "is_restricted": "BOOLEAN",
    "created_at": "TIMESTAMP (auto)",
    "updated_at": "TIMESTAMP (auto)"
  }
}
```

---

## ER Diagram (Entity Relationships)

```
content_modules (uuid: PK)
    |
    +-- spells (content_module_uuid: FK)
    |     +-- school => spell_schools (FK)
    +-- monsters (content_module_uuid: FK)
    |     +-- type => monster_types (FK)
    |     +-- alignment => alignments (FK)
    |     +-- languages_list => languages (FK refs in array)
    |     +-- damage_resistances/vulnerabilities/immunities => damage_types (FK refs in array)
    |     +-- condition_immunities => conditions (FK refs in array)
    +-- classes (content_module_uuid: FK)
    +-- subclasses (content_module_uuid: FK, parent_class_uuid: FK => classes)
    +-- races (content_module_uuid: FK)
    |     +-- languages_base/additional => languages (FK refs)
    +-- racial_traits (content_module_uuid: FK)
    +-- feats (content_module_uuid: FK)
    +-- backgrounds (content_module_uuid: FK)
    +-- conditions (content_module_uuid: FK)
    +-- planes (content_module_uuid: FK, parent_uuid: FK => planes)
    +-- magic_items (content_module_uuid: FK)
    +-- magic_item_categories (content_module_uuid: FK, parent_uuid: FK => magic_item_categories)
    +-- weapons (content_module_uuid: FK, category_uuid: FK => equipment_categories, damage_type => damage_types)
    +-- armors (content_module_uuid: FK, category_uuid: FK => equipment_categories)
    +-- equipment_categories (content_module_uuid: FK, parent_uuid: FK => equipment_categories)
    +-- equipment_subcategories (content_module_uuid: FK, category_uuid: FK => equipment_categories)
    +-- alignments (content_module_uuid: FK)
    +-- ability_scores (content_module_uuid: FK)
    +-- damage_types (content_module_uuid: FK)
    +-- languages (content_module_uuid: FK)
    +-- spell_schools (content_module_uuid: FK)
    +-- spell_lists (content_module_uuid: FK)
    +-- equipment_proficiencies (content_module_uuid: FK, category_uuid: FK => equipment_categories)
    +-- equipment_materials (content_module_uuid: FK)
    +-- enchantments (content_module_uuid: FK)
    +-- character_features (content_module_uuid: FK)
    +-- documents (content_module_uuid: FK)
    +-- monster_types (content_module_uuid: FK)
    +-- monster_actions (content_module_uuid: FK)
```

---

## Foreign Key Index

| Source Schema | FK Field | References Schema | PK Field |
|---|---|---|---|
| spells | content_module_uuid | content_modules | uuid |
| spells | school | spell_schools | uuid |
| monsters | content_module_uuid | content_modules | uuid |
| monsters | type | monster_types | uuid |
| monsters | alignment | alignments | uuid |
| subclasses | content_module_uuid | content_modules | uuid |
| subclasses | parent_class_uuid | classes | uuid |
| races | content_module_uuid | content_modules | uuid |
| racial_traits | content_module_uuid | content_modules | uuid |
| feats | content_module_uuid | content_modules | uuid |
| backgrounds | content_module_uuid | content_modules | uuid |
| conditions | content_module_uuid | content_modules | uuid |
| planes | content_module_uuid | content_modules | uuid |
| planes | parent_uuid | planes | uuid |
| magic_items | content_module_uuid | content_modules | uuid |
| magic_item_categories | content_module_uuid | content_modules | uuid |
| magic_item_categories | parent_uuid | magic_item_categories | uuid |
| weapons | content_module_uuid | content_modules | uuid |
| weapons | category_uuid | equipment_categories | uuid |
| weapons | damage_type | damage_types | uuid |
| armors | content_module_uuid | content_modules | uuid |
| armors | category_uuid | equipment_categories | uuid |
| equipment_categories | content_module_uuid | content_modules | uuid |
| equipment_categories | parent_uuid | equipment_categories | uuid |
| equipment_subcategories | content_module_uuid | content_modules | uuid |
| equipment_subcategories | category_uuid | equipment_categories | uuid |
| alignments | content_module_uuid | content_modules | uuid |
| ability_scores | content_module_uuid | content_modules | uuid |
| damage_types | content_module_uuid | content_modules | uuid |
| languages | content_module_uuid | content_modules | uuid |
| spell_schools | content_module_uuid | content_modules | uuid |
| spell_lists | content_module_uuid | content_modules | uuid |
| equipment_proficiencies | content_module_uuid | content_modules | uuid |
| equipment_proficiencies | category_uuid | equipment_categories | uuid |
| equipment_materials | content_module_uuid | content_modules | uuid |
| enchantments | content_module_uuid | content_modules | uuid |
| character_features | content_module_uuid | content_modules | uuid |
| documents | content_module_uuid | content_modules | uuid |
| monster_types | content_module_uuid | content_modules | uuid |
| monster_actions | content_module_uuid | content_modules | uuid |

---

## Design Notes

1. **UUID as primary key** for all entities ensures unambiguous cross-referenceability across content packs.
2. **content_module_uuid as the attribution anchor** — every entity references the content pack (book/supplement) that defines it. This supports: (a) versioning (same rule text, different source books), (b) licensing (some content packs are CC-BY, others are OGL, others are proprietary), (c) filtering (only show SRD content vs. non-SRD content).
3. **is_restricted flag** on every schema marks content that is NOT part of the open SRD.
4. **slug fields** provide human-readable, URL-safe identifiers for lookup endpoints.
5. **JSON structured fields** capture inherently unstructured game data (e.g., class progression tables, monster action objects, spell effects).
6. **content_module** is the parent that ties all content together.
7. **Hierarchical parent_uuid patterns** appear in planes, magic_item_categories, and equipment_categories to support tree-like category structures.
8. **Enum constraints** on certain fields (alignment, damage_type, school, ability_score, etc.) should be defined at the schema definition level.
9. **equipment_categories** serves as the primary category hub; **equipment_subcategories** provides deeper granularity.
10. **damage_types** is a small, self-describing look-up table that maps to damage_resistances/vulnerabilities/immunities arrays in the monsters table.
11. **languages** is a standalone look-up table referenced by both monster and race schemas.
12. **spell_schools** and **spell_lists** are shared references — the spell schema references the school directly and spells on the spell list via arrays.
13. **conditions** defines standard SRD conditions which are referenced in the condition_immunities field of monsters.
14. **planes** can form a tree hierarchy via parent_uuid.
15. **character_features** is a catch-all for general character abilities.
16. **document** is a secondary source attribution for source documents.
17. **classes** table supports archetypes via the archetype_table JSON field; **subclasses** is the primary table for subclass definitions.
