//! Single-sourced authoring metadata for homebrew content.
//!
//! Every client (Leptos web via WASM, Flutter via FFI) and the server
//! itself drive their authoring forms and pre-submit validation from the
//! one [`FieldSchema`] per content category defined here. A field schema
//! describes only the *authorable* subset of each type: server-managed
//! fields (`uuid`, `content_module_uuid`, `document_uuid`, `key`, `slug`,
//! `is_restricted`, timestamps) are filled in when the server assembles
//! the full record, so they never appear in a form.
//!
//! Validation is schema-driven so the same rules run everywhere; the
//! server additionally deserializes the assembled record into its typed
//! [`lorewyld_types`] struct as the authoritative structural guard.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The content categories a user may author homebrew records for. Matches
/// the server's `DISPLAY_CATEGORIES` and the clients' compendium tiles.
pub const AUTHORABLE_CATEGORIES: &[&str] = &[
    "spell",
    "creature",
    "class",
    "species",
    "background",
    "feat",
    "item",
    "weapon",
    "armor",
    "condition",
    "language",
];

/// One selectable value for an [`FieldKind::SelectEnum`] / `EnumList`
/// field. `value` is the wire form (matches the type's serde rename);
/// `label` is the human-facing display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumOption {
    pub value: String,
    pub label: String,
}

/// The control a field renders as, plus its constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FieldKind {
    /// Single-line free text.
    Text,
    /// Multi-line markdown body.
    Markdown,
    /// Whole number with optional inclusive bounds.
    Int {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<i64>,
    },
    /// Decimal number with optional inclusive bounds.
    Float {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
    },
    /// Checkbox.
    Bool,
    /// Single choice from a closed value set.
    SelectEnum { options: Vec<EnumOption> },
    /// Zero or more choices from a closed value set.
    EnumList { options: Vec<EnumOption> },
    /// Foreign-key reference picked from a lookup table (value is a UUID
    /// string). `table` names the compendium lookup the picker queries.
    SelectLookup { table: String },
    /// Structured-but-variable data edited as raw JSON in the first cut
    /// (creature actions, class features, equipment tables, …). The
    /// `template` seeds the editor with a well-formed default.
    Json {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        template: Option<Value>,
    },
}

/// One authorable field of a content type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDef {
    /// JSON key on the record (matches the type's serde field name).
    pub key: String,
    pub label: String,
    #[serde(flatten)]
    pub kind: FieldKind,
    #[serde(default)]
    pub required: bool,
    /// Short helper text shown under the control.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

/// The full authoring form definition for one content category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldSchema {
    pub category: String,
    pub fields: Vec<FieldDef>,
}

/// A single validation failure, keyed to the field it concerns (empty
/// `field` for record-level errors).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

// ─── Enum option tables (values match the types' serde renames) ─────────

fn ability_options() -> Vec<EnumOption> {
    [
        ("strength", "Strength"),
        ("dexterity", "Dexterity"),
        ("constitution", "Constitution"),
        ("intelligence", "Intelligence"),
        ("wisdom", "Wisdom"),
        ("charisma", "Charisma"),
    ]
    .into_iter()
    .map(|(v, l)| EnumOption {
        value: v.into(),
        label: l.into(),
    })
    .collect()
}

fn damage_type_options() -> Vec<EnumOption> {
    [
        "acid",
        "bludgeoning",
        "cold",
        "fire",
        "force",
        "lightning",
        "necrotic",
        "piercing",
        "poison",
        "psychic",
        "radiant",
        "slashing",
        "thunder",
    ]
    .into_iter()
    .map(title_option)
    .collect()
}

fn condition_options() -> Vec<EnumOption> {
    [
        "blinded",
        "charmed",
        "deafened",
        "exhaustion",
        "frightened",
        "grappled",
        "incapacitated",
        "invisible",
        "paralyzed",
        "petrified",
        "poisoned",
        "prone",
        "restrained",
        "stunned",
        "unconscious",
    ]
    .into_iter()
    .map(title_option)
    .collect()
}

fn rarity_options() -> Vec<EnumOption> {
    [
        ("common", "Common"),
        ("uncommon", "Uncommon"),
        ("rare", "Rare"),
        ("very_rare", "Very Rare"),
        ("legendary", "Legendary"),
        ("artifact", "Artifact"),
    ]
    .into_iter()
    .map(|(v, l)| EnumOption {
        value: v.into(),
        label: l.into(),
    })
    .collect()
}

fn armor_category_options() -> Vec<EnumOption> {
    [
        ("light", "Light"),
        ("medium", "Medium"),
        ("heavy", "Heavy"),
        ("shield", "Shield"),
    ]
    .into_iter()
    .map(|(v, l)| EnumOption {
        value: v.into(),
        label: l.into(),
    })
    .collect()
}

/// Builds an option whose label is the Title-cased form of its value.
fn title_option(value: &str) -> EnumOption {
    let mut chars = value.chars();
    let label = match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    };
    EnumOption {
        value: value.into(),
        label,
    }
}

// ─── Field-def builder shorthands ───────────────────────────────────────

fn text(key: &str, label: &str, required: bool) -> FieldDef {
    FieldDef {
        key: key.into(),
        label: label.into(),
        kind: FieldKind::Text,
        required,
        help: None,
    }
}

fn markdown(key: &str, label: &str, required: bool) -> FieldDef {
    FieldDef {
        key: key.into(),
        label: label.into(),
        kind: FieldKind::Markdown,
        required,
        help: None,
    }
}

fn int(key: &str, label: &str, required: bool, min: Option<i64>, max: Option<i64>) -> FieldDef {
    FieldDef {
        key: key.into(),
        label: label.into(),
        kind: FieldKind::Int { min, max },
        required,
        help: None,
    }
}

fn float(key: &str, label: &str, required: bool, min: Option<f64>, max: Option<f64>) -> FieldDef {
    FieldDef {
        key: key.into(),
        label: label.into(),
        kind: FieldKind::Float { min, max },
        required,
        help: None,
    }
}

fn boolean(key: &str, label: &str) -> FieldDef {
    FieldDef {
        key: key.into(),
        label: label.into(),
        kind: FieldKind::Bool,
        required: false,
        help: None,
    }
}

fn select_enum(key: &str, label: &str, required: bool, options: Vec<EnumOption>) -> FieldDef {
    FieldDef {
        key: key.into(),
        label: label.into(),
        kind: FieldKind::SelectEnum { options },
        required,
        help: None,
    }
}

fn enum_list(key: &str, label: &str, options: Vec<EnumOption>) -> FieldDef {
    FieldDef {
        key: key.into(),
        label: label.into(),
        kind: FieldKind::EnumList { options },
        required: false,
        help: None,
    }
}

fn lookup(key: &str, label: &str, required: bool, table: &str) -> FieldDef {
    FieldDef {
        key: key.into(),
        label: label.into(),
        kind: FieldKind::SelectLookup {
            table: table.into(),
        },
        required,
        help: None,
    }
}

fn json_field(key: &str, label: &str, template: Value, help: &str) -> FieldDef {
    FieldDef {
        key: key.into(),
        label: label.into(),
        kind: FieldKind::Json {
            template: Some(template),
        },
        required: false,
        help: Some(help.into()),
    }
}

fn with_help(mut field: FieldDef, help: &str) -> FieldDef {
    field.help = Some(help.into());
    field
}

// ─── Per-category schemas ───────────────────────────────────────────────

/// The authoring schema for `category`, or `None` if the category is not
/// user-authorable.
pub fn field_schema(category: &str) -> Option<FieldSchema> {
    let fields = match category {
        "spell" => spell_fields(),
        "creature" => creature_fields(),
        "class" => class_fields(),
        "species" => species_fields(),
        "background" => background_fields(),
        "feat" => feat_fields(),
        "item" => item_fields(),
        "weapon" => weapon_fields(),
        "armor" => armor_fields(),
        "condition" => condition_fields(),
        "language" => language_fields(),
        _ => return None,
    };
    Some(FieldSchema {
        category: category.into(),
        fields,
    })
}

fn spell_fields() -> Vec<FieldDef> {
    vec![
        text("name", "Name", true),
        with_help(
            int("level", "Spell level", true, Some(0), Some(9)),
            "0 for a cantrip; 1–9 otherwise.",
        ),
        lookup("school", "School of magic", true, "spell_school"),
        with_help(
            text("casting_time", "Casting time", true),
            "e.g. \"1 action\", \"1 bonus action\".",
        ),
        with_help(
            text("range_text", "Range", false),
            "e.g. \"60 feet\", \"Self\".",
        ),
        text("duration", "Duration", true),
        boolean("concentration", "Concentration"),
        boolean("ritual", "Ritual"),
        boolean("verbal", "Verbal component"),
        boolean("somatic", "Somatic component"),
        boolean("material", "Material component"),
        text("material_specified", "Material detail", false),
        with_help(
            text("damage_roll", "Damage roll", false),
            "Dice expression, e.g. \"8d6\".",
        ),
        enum_list("damage_types", "Damage types", damage_type_options()),
        select_enum(
            "saving_throw_ability",
            "Saving throw",
            false,
            ability_options(),
        ),
        boolean("attack_roll", "Spell attack roll"),
        markdown("description", "Description", true),
        markdown("higher_level", "At higher levels", false),
    ]
}

fn creature_fields() -> Vec<FieldDef> {
    vec![
        text("name", "Name", true),
        lookup("size", "Size", true, "size"),
        lookup("type", "Creature type", true, "creature_type"),
        with_help(
            text("alignment", "Alignment", true),
            "Free text, e.g. \"chaotic evil\".",
        ),
        float(
            "challenge_rating",
            "Challenge rating",
            true,
            Some(0.0),
            Some(30.0),
        ),
        int("armor_class", "Armor class", true, Some(0), None),
        int("hit_points", "Hit points", true, Some(1), None),
        with_help(text("hit_dice", "Hit dice", true), "e.g. \"18d10+72\"."),
        json_field(
            "ability_scores",
            "Ability scores",
            serde_json::json!({
                "strength": 10, "dexterity": 10, "constitution": 10,
                "intelligence": 10, "wisdom": 10, "charisma": 10
            }),
            "Six core ability scores.",
        ),
        json_field(
            "speed",
            "Speed",
            serde_json::json!({ "walk": 30 }),
            "Movement modes in feet (walk, fly, swim, climb, burrow).",
        ),
        json_field(
            "senses",
            "Senses",
            serde_json::json!({ "passive_perception": 10 }),
            "Passive perception plus darkvision/blindsight/etc.",
        ),
        text("languages", "Languages", false),
        enum_list(
            "damage_immunities",
            "Damage immunities",
            damage_type_options(),
        ),
        enum_list(
            "condition_immunities",
            "Condition immunities",
            condition_options(),
        ),
        json_field(
            "traits",
            "Traits",
            serde_json::json!([]),
            "Array of { name, desc } non-action traits.",
        ),
        json_field(
            "actions",
            "Actions",
            serde_json::json!([]),
            "Array of stat-block actions (see docs for shape).",
        ),
        markdown("legendary_desc", "Legendary actions", false),
    ]
}

fn class_fields() -> Vec<FieldDef> {
    vec![
        text("name", "Name", true),
        markdown("desc", "Description", true),
        with_help(
            int("hit_dice", "Hit die", false, Some(4), Some(12)),
            "Die size: 6, 8, 10, or 12.",
        ),
        with_help(
            text("caster_type", "Caster type", false),
            "e.g. \"FULL\", \"HALF\", \"NONE\".",
        ),
        text("prof_armor", "Armor proficiencies", false),
        text("prof_weapons", "Weapon proficiencies", false),
        text("prof_tools", "Tool proficiencies", false),
        enum_list(
            "prof_saving_throws",
            "Saving-throw proficiencies",
            ability_options(),
        ),
        select_enum(
            "spellcasting_ability",
            "Spellcasting ability",
            false,
            ability_options(),
        ),
        lookup("subclass_of", "Subclass of", false, "class"),
        json_field(
            "features",
            "Features",
            serde_json::json!([]),
            "Array of { key, name, desc, gained_at } class features.",
        ),
    ]
}

fn species_fields() -> Vec<FieldDef> {
    vec![
        text("name", "Name", true),
        markdown("desc", "Description", true),
        int("speed", "Walking speed", true, Some(0), None),
        lookup("size", "Size", true, "size"),
        json_field(
            "asi",
            "Ability score increases",
            serde_json::json!({
                "strength": 0, "dexterity": 0, "constitution": 0,
                "intelligence": 0, "wisdom": 0, "charisma": 0
            }),
            "Fixed ability-score increases.",
        ),
        with_help(
            text("asi_desc", "ASI summary", false),
            "e.g. \"+2 DEX, +1 INT\".",
        ),
        boolean("is_subspecies", "Is a subspecies"),
        lookup("subspecies_of", "Subspecies of", false, "species"),
        json_field(
            "traits",
            "Traits",
            serde_json::json!([]),
            "Array of { name, desc } species traits.",
        ),
    ]
}

fn background_fields() -> Vec<FieldDef> {
    vec![
        text("name", "Name", true),
        markdown("desc", "Description", true),
        json_field(
            "benefits",
            "Benefits",
            serde_json::json!([]),
            "Array of { name, desc, type } benefits.",
        ),
    ]
}

fn feat_fields() -> Vec<FieldDef> {
    vec![
        text("name", "Name", true),
        with_help(
            text("type", "Category", false),
            "e.g. \"GENERAL\", \"ORIGIN\".",
        ),
        markdown("desc", "Description", true),
        boolean("has_prerequisite", "Has prerequisite"),
        text("prerequisite", "Prerequisite", false),
        json_field(
            "benefits",
            "Benefits",
            serde_json::json!([]),
            "Array of { name?, desc } benefits.",
        ),
    ]
}

fn item_fields() -> Vec<FieldDef> {
    vec![
        text("name", "Name", true),
        markdown("desc", "Description", true),
        lookup("category_uuid", "Category", true, "item_category"),
        float("weight", "Weight (lb)", false, Some(0.0), None),
        with_help(text("cost", "Cost", false), "Gold-piece cost, e.g. \"25\"."),
        select_enum("rarity", "Rarity", false, rarity_options()),
        boolean("requires_attunement", "Requires attunement"),
        text("attunement_detail", "Attunement detail", false),
        boolean("is_magic", "Magic item"),
    ]
}

fn weapon_fields() -> Vec<FieldDef> {
    vec![
        text("name", "Name", true),
        with_help(text("damage_dice", "Damage dice", true), "e.g. \"1d8\"."),
        select_enum("damage_type", "Damage type", false, damage_type_options()),
        float("range", "Range (ft)", false, Some(0.0), None),
        float("long_range", "Long range (ft)", false, Some(0.0), None),
        boolean("is_simple", "Simple weapon"),
    ]
}

fn armor_fields() -> Vec<FieldDef> {
    vec![
        text("name", "Name", true),
        select_enum("category", "Category", true, armor_category_options()),
        int("ac_base", "Base AC", true, Some(0), None),
        boolean("ac_add_dexmod", "Adds Dex modifier"),
        int("ac_cap_dexmod", "Max Dex bonus", false, None, None),
        with_help(
            text("ac_display", "AC display", true),
            "e.g. \"14 + Dex modifier (max 2)\".",
        ),
        boolean("grants_stealth_disadvantage", "Stealth disadvantage"),
        int(
            "strength_score_required",
            "Strength requirement",
            false,
            Some(0),
            None,
        ),
    ]
}

fn condition_fields() -> Vec<FieldDef> {
    vec![
        with_help(
            select_enum("name", "Condition", true, condition_options()),
            "Homebrew conditions reuse the SRD condition set.",
        ),
        markdown("desc", "Effect", true),
    ]
}

fn language_fields() -> Vec<FieldDef> {
    vec![
        text("name", "Name", true),
        markdown("desc", "Description", false),
        boolean("is_exotic", "Exotic"),
        boolean("is_secret", "Secret"),
        with_help(
            text("script", "Script", false),
            "Key of the language whose script this uses.",
        ),
    ]
}

// ─── Validation ─────────────────────────────────────────────────────────

/// Validates a JSON-encoded authoring input against the category's schema.
/// Returns the list of field errors, or `Ok(())` when the input is valid.
/// Unknown categories and unparseable JSON yield a single record-level
/// error.
pub fn validate_record(category: &str, input_json: &str) -> Result<(), Vec<FieldError>> {
    let Some(schema) = field_schema(category) else {
        return Err(vec![FieldError {
            field: String::new(),
            message: format!("Unknown content category \"{category}\"."),
        }]);
    };
    let value: Value = serde_json::from_str(input_json).map_err(|e| {
        vec![FieldError {
            field: String::new(),
            message: format!("Invalid JSON: {e}"),
        }]
    })?;
    validate_value(&schema, &value)
}

/// Validates an already-parsed authoring input against a schema.
pub fn validate_value(schema: &FieldSchema, value: &Value) -> Result<(), Vec<FieldError>> {
    let mut errors = Vec::new();
    for field in &schema.fields {
        let present = value.get(&field.key);
        let is_empty = match present {
            None | Some(Value::Null) => true,
            Some(Value::String(s)) => s.trim().is_empty(),
            _ => false,
        };
        if field.required && is_empty {
            errors.push(FieldError {
                field: field.key.clone(),
                message: format!("{} is required.", field.label),
            });
            continue;
        }
        let Some(v) = present else { continue };
        if matches!(v, Value::Null) {
            continue;
        }
        validate_field(field, v, &mut errors);
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_field(field: &FieldDef, v: &Value, errors: &mut Vec<FieldError>) {
    let mut err = |message: String| {
        errors.push(FieldError {
            field: field.key.clone(),
            message,
        });
    };
    match &field.kind {
        FieldKind::Text | FieldKind::Markdown | FieldKind::SelectLookup { .. } => {
            if !v.is_string() {
                err(format!("{} must be text.", field.label));
            }
        }
        FieldKind::Bool => {
            if !v.is_boolean() {
                err(format!("{} must be true or false.", field.label));
            }
        }
        FieldKind::Int { min, max } => match v.as_i64() {
            Some(n) => {
                if min.is_some_and(|m| n < m) || max.is_some_and(|m| n > m) {
                    err(range_message(
                        &field.label,
                        min.map(|m| m as f64),
                        max.map(|m| m as f64),
                    ));
                }
            }
            None => err(format!("{} must be a whole number.", field.label)),
        },
        FieldKind::Float { min, max } => match v.as_f64() {
            Some(n) => {
                if min.is_some_and(|m| n < m) || max.is_some_and(|m| n > m) {
                    err(range_message(&field.label, *min, *max));
                }
            }
            None => err(format!("{} must be a number.", field.label)),
        },
        FieldKind::SelectEnum { options } => match v.as_str() {
            Some(s) if options.iter().any(|o| o.value == s) => {}
            _ => err(format!(
                "{} must be one of the listed options.",
                field.label
            )),
        },
        FieldKind::EnumList { options } => match v.as_array() {
            Some(items) => {
                for item in items {
                    let ok = item
                        .as_str()
                        .is_some_and(|s| options.iter().any(|o| o.value == s));
                    if !ok {
                        err(format!("{} contains an invalid option.", field.label));
                        break;
                    }
                }
            }
            None => err(format!("{} must be a list.", field.label)),
        },
        FieldKind::Json { .. } => {
            // Shape is validated authoritatively when the server
            // deserializes the assembled record into its typed struct.
        }
    }
}

fn range_message(label: &str, min: Option<f64>, max: Option<f64>) -> String {
    match (min, max) {
        (Some(lo), Some(hi)) => format!("{label} must be between {lo} and {hi}."),
        (Some(lo), None) => format!("{label} must be at least {lo}."),
        (None, Some(hi)) => format!("{label} must be at most {hi}."),
        (None, None) => format!("{label} is out of range."),
    }
}

// ─── Record skeletons ───────────────────────────────────────────────────

/// A nil-UUID / epoch-timestamp skeleton of a full content record for
/// `category`: every field the typed [`lorewyld_types`] struct requires,
/// defaulted. The server overlays the authorable form input on top, then
/// stamps real identity/timestamps before persisting — so this is the
/// merge base that guarantees a structurally-complete record even when
/// the form only carries the authorable subset. `None` for unknown
/// categories.
///
/// Single-sourced here (rather than per-platform) and pinned by
/// `skeleton_deserializes_into_its_type`: every skeleton must round-trip
/// into its Rust type, so a required-field addition that breaks the
/// skeleton fails the domain test rather than 500ing at runtime.
pub fn default_record(category: &str) -> Option<Value> {
    const NIL: &str = "00000000-0000-0000-0000-000000000000";
    const EPOCH: &str = "1970-01-01T00:00:00Z";
    let abilities = serde_json::json!({
        "strength": 10, "dexterity": 10, "constitution": 10,
        "intelligence": 10, "wisdom": 10, "charisma": 10
    });
    let zero_abilities = serde_json::json!({
        "strength": 0, "dexterity": 0, "constitution": 0,
        "intelligence": 0, "wisdom": 0, "charisma": 0
    });
    let record = match category {
        "spell" => serde_json::json!({
            "uuid": NIL, "content_module_uuid": NIL, "document_uuid": NIL,
            "name": "", "slug": "", "key": "", "level": 0, "school": NIL,
            "concentration": false, "casting_time": "", "range": 0.0,
            "duration": "", "verbal": false, "somatic": false, "material": false,
            "description": "", "is_restricted": false,
            "created_at": EPOCH, "updated_at": EPOCH
        }),
        "creature" => serde_json::json!({
            "uuid": NIL, "content_module_uuid": NIL, "document_uuid": NIL,
            "name": "", "slug": "", "key": "", "type": NIL, "size": NIL,
            "alignment": "", "challenge_rating": 0.0, "proficiency_bonus": 2,
            "experience_points": 0, "armor_class": 10, "hit_points": 1,
            "hit_dice": "", "speed": { "walk": 30 },
            "ability_scores": abilities, "modifiers": zero_abilities,
            "senses": { "passive_perception": 10 }, "languages": "",
            "is_restricted": false, "created_at": EPOCH, "updated_at": EPOCH
        }),
        "class" => serde_json::json!({
            "uuid": NIL, "content_module_uuid": NIL, "document_uuid": NIL,
            "name": "", "slug": "", "key": "", "desc": "",
            "is_restricted": false, "created_at": EPOCH, "updated_at": EPOCH
        }),
        "species" => serde_json::json!({
            "uuid": NIL, "content_module_uuid": NIL, "document_uuid": NIL,
            "name": "", "slug": "", "key": "", "desc": "", "asi": zero_abilities,
            "asi_desc": "", "speed": 30, "size": NIL,
            "is_restricted": false, "created_at": EPOCH, "updated_at": EPOCH
        }),
        "background" => serde_json::json!({
            "uuid": NIL, "content_module_uuid": NIL, "document_uuid": NIL,
            "name": "", "slug": "", "key": "", "desc": "",
            "is_restricted": false, "created_at": EPOCH, "updated_at": EPOCH
        }),
        "feat" => serde_json::json!({
            "uuid": NIL, "content_module_uuid": NIL, "document_uuid": NIL,
            "name": "", "slug": "", "key": "", "desc": "",
            "is_restricted": false, "created_at": EPOCH, "updated_at": EPOCH
        }),
        "item" => serde_json::json!({
            "uuid": NIL, "content_module_uuid": NIL, "document_uuid": NIL,
            "name": "", "slug": "", "key": "", "desc": "", "category_uuid": NIL,
            "is_restricted": false, "created_at": EPOCH, "updated_at": EPOCH
        }),
        "weapon" => serde_json::json!({
            "uuid": NIL, "content_module_uuid": NIL, "document_uuid": NIL,
            "name": "", "slug": "", "key": "", "damage_dice": "", "is_simple": false,
            "is_restricted": false, "created_at": EPOCH, "updated_at": EPOCH
        }),
        "armor" => serde_json::json!({
            "uuid": NIL, "content_module_uuid": NIL, "document_uuid": NIL,
            "name": "", "slug": "", "key": "", "category": "light", "ac_base": 10,
            "ac_add_dexmod": false, "ac_display": "",
            "grants_stealth_disadvantage": false,
            "is_restricted": false, "created_at": EPOCH, "updated_at": EPOCH
        }),
        "condition" => serde_json::json!({
            "uuid": NIL, "content_module_uuid": NIL, "name": "blinded", "slug": "",
            "key": "", "desc": "", "is_restricted": false,
            "created_at": EPOCH, "updated_at": EPOCH
        }),
        "language" => serde_json::json!({
            "uuid": NIL, "content_module_uuid": NIL, "name": "", "slug": "",
            "key": "", "desc": "", "is_exotic": false, "is_secret": false,
            "is_restricted": false, "created_at": EPOCH, "updated_at": EPOCH
        }),
        _ => return None,
    };
    Some(record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_authorable_category_has_a_schema_with_a_name_field() {
        for cat in AUTHORABLE_CATEGORIES {
            let schema = field_schema(cat).unwrap_or_else(|| panic!("no schema for {cat}"));
            assert_eq!(&schema.category, cat);
            assert!(
                schema.fields.iter().any(|f| f.key == "name"),
                "{cat} schema missing a name field"
            );
        }
    }

    #[test]
    fn unknown_category_has_no_schema() {
        assert!(field_schema("dragon").is_none());
        assert!(validate_record("dragon", "{}").is_err());
    }

    #[test]
    fn missing_required_name_is_rejected() {
        let errs = validate_record("spell", r#"{"level":3}"#).unwrap_err();
        assert!(errs.iter().any(|e| e.field == "name"));
    }

    #[test]
    fn out_of_range_level_is_rejected() {
        let errs = validate_record("spell", r#"{"name":"Test","level":12,"school":"x","casting_time":"1 action","duration":"Instant","description":"d"}"#).unwrap_err();
        assert!(errs.iter().any(|e| e.field == "level"));
    }

    #[test]
    fn valid_spell_input_passes() {
        let input = r#"{
            "name": "Spark",
            "level": 0,
            "school": "0c8c1b2e-0000-0000-0000-000000000000",
            "casting_time": "1 action",
            "duration": "Instantaneous",
            "description": "A tiny spark.",
            "damage_types": ["fire"],
            "saving_throw_ability": "dexterity"
        }"#;
        assert!(validate_record("spell", input).is_ok());
    }

    #[test]
    fn invalid_enum_member_is_rejected() {
        let errs = validate_record(
            "spell",
            r#"{"name":"X","level":1,"school":"s","casting_time":"1 action","duration":"d","description":"d","damage_types":["plasma"]}"#,
        )
        .unwrap_err();
        assert!(errs.iter().any(|e| e.field == "damage_types"));
    }

    #[test]
    fn schema_round_trips_through_json() {
        let schema = field_schema("creature").unwrap();
        let json = serde_json::to_string(&schema).unwrap();
        let back: FieldSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(schema, back);
    }

    /// Each skeleton must deserialize into its `lorewyld_types` struct, so
    /// a required-field change to a content type that the skeleton misses
    /// fails here (in the domain crate) rather than 500ing on the server's
    /// record-assembly path.
    #[test]
    fn skeleton_deserializes_into_its_type() {
        use lorewyld_types::{
            Armor, Background, Class, Condition, Creature, Feat, Item, Language, Species, Spell,
            Weapon,
        };
        macro_rules! check {
            ($cat:literal, $ty:ty) => {{
                let v = default_record($cat).expect(concat!("no skeleton for ", $cat));
                serde_json::from_value::<$ty>(v)
                    .expect(concat!($cat, " skeleton does not deserialize into its type"));
            }};
        }
        check!("spell", Spell);
        check!("creature", Creature);
        check!("class", Class);
        check!("species", Species);
        check!("background", Background);
        check!("feat", Feat);
        check!("item", Item);
        check!("weapon", Weapon);
        check!("armor", Armor);
        check!("condition", Condition);
        check!("language", Language);
    }

    #[test]
    fn unknown_category_has_no_skeleton() {
        assert!(default_record("dragon").is_none());
    }
}
