# D&D 5e SRD Data Model (v2)

All shipped base content follows the System Reference Document (SRD) for D&D 5th
edition, licensed under CC BY 4.0. The model is aligned with the **Open5e v2 API**
(`https://api.open5e.com/v2/`) structure; the original v1-shaped model this
document previously described was retired at `SCHEMA_VERSION = 2` before any data
was persisted.

**Source of truth:** the Rust crate at `shared/types/src/`. Every persisted type
is a Serde-derived Rust struct there, consumed natively by the server/web, via
WASM, and over FFI by mobile (see `docs/ARCHITECTURE.md`); this document explains
the model's shape and the decisions behind it rather than duplicating field lists.

---

## Identity and provenance

Every content record carries:

| Field | Meaning |
|---|---|
| `uuid` | Primary key. **Deterministic**: `UUIDv5(LOREWYLD_CONTENT_NAMESPACE, "{type}:{key}")` via `common::content_uuid`. Regenerating a bundle never churns identities, and FKs are computable without lookups. |
| `key` | Stable external identifier — the Open5e key for imported content (e.g. `srd-2024_fireball`, `srd_wizard`). Treat as **opaque**: upstream prefixes are inconsistent and do not reliably encode the source document. |
| `slug` | URL/display-friendly identifier. |
| `content_module_uuid` | FK → `content_module`. The module is the unit of licensing, activation, and attribution UI. |
| `document_uuid` | (major content types only) FK → `document`. Per-record source attribution — required because one module can mix sources (the shipped SRD module is 5.2-base, gap-filled from 5.1). |
| `is_restricted` | Per-record restriction flag. |
| `created_at` / `updated_at` | Audit timestamps. Seeded content uses the bundle's pinned generation timestamp so output is byte-stable. |

Provenance graph (Open5e v2's document/publisher/license split):

```
license ──< document >── publisher
              │  (gamesystem_key: bare string — one shipped game system)
              ▼
   every major content record (document_uuid)
```

## Content types

### Major content (carry `document_uuid`)

| Type | File | Notes |
|---|---|---|
| `Spell` | `spell.rs` | Fully structured per v2: numeric `range` + unit/text, target/shape fields, V/S/M booleans + material detail, `saving_throw_ability`, `attack_roll`, `damage_roll`/`damage_types`, and a typed `casting_options[]` upcast table (`"default"`, `"slot_level_N"`, `"player_level_N"` rows with nullable overrides). `classes` holds caster-class keys. |
| `Creature` | `creature.rs` | Replaces v1 `Monster`. One ordered `actions[]` array with a `type` discriminator (`ACTION`/`BONUS_ACTION`/`REACTION`/`LEGENDARY_ACTION`) replaces five parallel arrays. Attacks are fully structured (`CreatureAttack`: to-hit, reach/range, damage dice + extra damage). `traits[]` holds prose traits — spellcasting stays prose, matching the SRD. Numeric CR (`0.125` = 1/8), `proficiency_bonus`, typed resistance/immunity arrays with `*_display` companions for conditional prose. |
| `Class` | `class.rs` | Subclasses are **sibling rows** linked via `subclass_of` (v2 model) — no separate Subclass type. `features[]` is typed (`ClassFeature` with `gained_at` level list). **Retained sheet-math fields** (see below): `prof_*`, `equipment`, `spellcasting_ability`, `spell_slot_table`, `hp_at_*`. |
| `Species` | `species.rs` | Replaces v1 `Race`. Subspecies are sibling rows via `subspecies_of`. `traits[]` is typed prose. **Retained sheet-math fields**: `asi`, `speed`, `size`, `languages_*`, `vision_base`. |
| `Background` | `background.rs` | Mechanics live in the typed `benefits[]` array (`ability_score`, `skill_proficiency`, `equipment`, `feature`, …) per v2. |
| `Feat` | `feat.rs` | `type` category, `has_prerequisite`/`prerequisite`, typed `benefits[]`. |
| `Item` | `item.rs` | **Unified mundane + magic gear** (v2 model — dissolves the v1 equipment/magic-item split). Carries category, cost (decimal string — exact, sortable after CAST), weight, rarity/attunement for magic items, and optional `weapon_uuid`/`armor_uuid` FKs. Magic items share the base weapon/armor record they enchant. |
| `Weapon` / `Armor` | `item.rs` | Pure mechanics records wrapped by `Item`. Weapon: damage dice/type, range, `is_simple`, typed `properties[]` refs with per-weapon detail (Versatile `"1d10"`). Armor: structured AC (`ac_base`, `ac_add_dexmod`, `ac_cap_dexmod`), stealth disadvantage, strength gate. |

### Lookup tables (module-scoped, no `document_uuid`)

`AbilityScoreEntry`, `Skill`, `Alignment`, `DamageType`, `Condition`, `Language`,
`Size`, `Environment`, `SpellSchool`, `CreatureType`, `ItemCategory`,
`WeaponPropertyDef` — small rows backing closed-set enums (where one exists) or
open vocabularies (sizes, environments, weapon properties — these replaced v1
closed enums so content packs can extend them).

### Closed-set enums (`common.rs` unless noted)

`AbilityScore`, `AlignmentName`, `DamageTypeName`, `ConditionName`,
`SpellSchoolName`, `Rarity` (variant order = Open5e rarity rank),
`CreatureTypeName` (`creature.rs`), `CreatureActionKind` (`creature.rs`).

## Retained sheet-math fields

Open5e v2 dropped to prose several structures the character-creation flow
computes against. We keep them as first-class fields; the **bundle generator**
populates them by joining the Open5e **v1 API** (classes: proficiencies,
equipment, spell-slot tables, spellcasting ability; races: ASI, speed, vision,
languages) and curated overrides in `tools/bundle-gen/data/overrides.json`
(2024-edition deltas, the 2024 species-ASI policy, dedup name aliases).

Never derive mechanics from v2's sparsely populated fields (`caster_type`,
`data_for_class_table`, `primary_abilities`) — they are informational only.

## Bundle format

`bundle.rs` defines `ContentBundle`: a `SchemaVersion` envelope + one `Vec` per
type, ordered by import dependency (lookups before the records that reference
them). Every list is `#[serde(default)]` so partial packs work. Versioning lives
in `version.rs` (`SCHEMA_VERSION = 2`).

The shipped SRD bundle is produced by `tools/bundle-gen` (see its README):
SRD 5.2 (`srd-2024`) as the base, gap-filled with SRD 5.1 (`srd-2014`) records
whose normalized names are absent from 5.2, deduplicated by name. Output is
deterministic: records sorted by `key`, pinned timestamps, stable UUIDs.

## Dropped at v2 (no Open5e v2 data source, no consumer)

`Plane`, `SpellList` (superseded by `Spell.classes`), `Enchantment`,
`EquipmentCategory`/`Subcategory`/`Proficiency`/`Material`, `MagicItem` +
`MagicItemCategory` (merged into `Item`), `RacialTrait`, standalone
`MonsterAction`/`Spellcasting`, `SpellComponent` and `CreatureSize`/
`WeaponProperty` enums (now booleans / lookup rows).
