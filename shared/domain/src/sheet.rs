//! 5e character-sheet math.
//!
//! The sheet documents and computes (modifiers, proficiency from level,
//! save/skill bonuses) but never enforces — any value the player supplies
//! is accepted, mirroring the clients' permissive philosophy.

use lorewyld_types::character::CharacterSheet;
use lorewyld_types::common::AbilityScores;
use serde::{Deserialize, Serialize};

/// The six ability wire names, in canonical order.
pub const ABILITIES: [&str; 6] = [
    "strength",
    "dexterity",
    "constitution",
    "intelligence",
    "wisdom",
    "charisma",
];

/// The eighteen skills paired with their governing ability, in canonical
/// order. Skill wire names are the camelCase identifiers the clients use
/// (`animalHandling`, `sleightOfHand`), not snake/space forms.
pub const SKILLS: [(&str, &str); 18] = [
    ("acrobatics", "dexterity"),
    ("animalHandling", "wisdom"),
    ("arcana", "intelligence"),
    ("athletics", "strength"),
    ("deception", "charisma"),
    ("history", "intelligence"),
    ("insight", "wisdom"),
    ("intimidation", "charisma"),
    ("investigation", "intelligence"),
    ("medicine", "wisdom"),
    ("nature", "intelligence"),
    ("perception", "wisdom"),
    ("performance", "charisma"),
    ("persuasion", "charisma"),
    ("religion", "intelligence"),
    ("sleightOfHand", "dexterity"),
    ("stealth", "dexterity"),
    ("survival", "wisdom"),
];

/// A named derived value (an ability, save, or skill) and its bonus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedBonus {
    pub name: String,
    pub bonus: i32,
}

/// Everything the sheet UI derives from the raw scores, computed once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedStats {
    pub proficiency_bonus: i32,
    pub ability_modifiers: Vec<NamedBonus>,
    pub saving_throw_bonuses: Vec<NamedBonus>,
    pub skill_bonuses: Vec<NamedBonus>,
    pub initiative: i32,
    pub passive_perception: i32,
}

/// Floor((score - 10) / 2). Uses `div_euclid` so odd scores below 10
/// floor toward negative infinity (score 9 → -1), matching the 5e table;
/// truncating division would wrongly yield 0.
pub fn ability_modifier(score: i32) -> i32 {
    (score - 10).div_euclid(2)
}

/// Proficiency bonus by level: +2 at 1, rising to +6 at 20. Level is
/// clamped to 1..=20 so out-of-range sheets stay on the table.
pub fn proficiency_bonus(level: i32) -> i32 {
    2 + (level.clamp(1, 20) - 1).div_euclid(4)
}

/// The ability governing a skill (defaults to dexterity for unknown
/// names — content is data we read, not data we control).
fn skill_ability(skill: &str) -> &'static str {
    SKILLS
        .iter()
        .find(|(name, _)| *name == skill)
        .map(|(_, ability)| *ability)
        .unwrap_or("dexterity")
}

fn ability_score(scores: &AbilityScores, ability: &str) -> i32 {
    match ability {
        "strength" => scores.strength,
        "dexterity" => scores.dexterity,
        "constitution" => scores.constitution,
        "intelligence" => scores.intelligence,
        "wisdom" => scores.wisdom,
        "charisma" => scores.charisma,
        _ => 10,
    }
}

fn ability_modifier_of(scores: &AbilityScores, ability: &str) -> i32 {
    ability_modifier(ability_score(scores, ability))
}

/// Save bonus = ability modifier, plus proficiency if the ability is in
/// the sheet's `saving_throw_proficiencies`.
pub fn saving_throw_bonus(sheet: &CharacterSheet, ability: &str) -> i32 {
    let modifier = ability_modifier_of(&sheet.abilities, ability);
    let proficient = sheet
        .saving_throw_proficiencies
        .iter()
        .any(|a| a == ability);
    modifier + if proficient { proficiency_bonus(sheet.level) } else { 0 }
}

/// Skill bonus = governing-ability modifier, plus proficiency if the
/// skill is in the sheet's `skill_proficiencies`.
pub fn skill_bonus(sheet: &CharacterSheet, skill: &str) -> i32 {
    let modifier = ability_modifier_of(&sheet.abilities, skill_ability(skill));
    let proficient = sheet.skill_proficiencies.iter().any(|s| s == skill);
    modifier + if proficient { proficiency_bonus(sheet.level) } else { 0 }
}

/// Initiative = dexterity modifier.
pub fn initiative(sheet: &CharacterSheet) -> i32 {
    ability_modifier_of(&sheet.abilities, "dexterity")
}

/// Passive perception = 10 + Perception skill bonus.
pub fn passive_perception(sheet: &CharacterSheet) -> i32 {
    10 + skill_bonus(sheet, "perception")
}

/// Computes the full derived-stat block in one pass — the call the sheet
/// UI makes after every edit.
pub fn derive_stats(sheet: &CharacterSheet) -> DerivedStats {
    DerivedStats {
        proficiency_bonus: proficiency_bonus(sheet.level),
        ability_modifiers: ABILITIES
            .iter()
            .map(|a| NamedBonus {
                name: (*a).to_string(),
                bonus: ability_modifier_of(&sheet.abilities, a),
            })
            .collect(),
        saving_throw_bonuses: ABILITIES
            .iter()
            .map(|a| NamedBonus {
                name: (*a).to_string(),
                bonus: saving_throw_bonus(sheet, a),
            })
            .collect(),
        skill_bonuses: SKILLS
            .iter()
            .map(|(s, _)| NamedBonus {
                name: (*s).to_string(),
                bonus: skill_bonus(sheet, s),
            })
            .collect(),
        initiative: initiative(sheet),
        passive_perception: passive_perception(sheet),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a sheet with the given level/abilities/proficiencies.
    /// Abilities default to 10 (the 5e baseline and the mobile UI default).
    fn sheet(
        level: i32,
        abilities: &[(&str, i32)],
        saves: &[&str],
        skills: &[&str],
    ) -> CharacterSheet {
        let mut scores = AbilityScores {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        };
        for (name, value) in abilities {
            match *name {
                "strength" => scores.strength = *value,
                "dexterity" => scores.dexterity = *value,
                "constitution" => scores.constitution = *value,
                "intelligence" => scores.intelligence = *value,
                "wisdom" => scores.wisdom = *value,
                "charisma" => scores.charisma = *value,
                other => panic!("unknown ability {other}"),
            }
        }
        let epoch = chrono::DateTime::from_timestamp(0, 0).unwrap();
        CharacterSheet {
            uuid: uuid::Uuid::nil(),
            name: "Thistle".to_string(),
            owner_user_uuid: None,
            owner_username: None,
            race: String::new(),
            class_name: String::new(),
            level,
            background: String::new(),
            alignment: String::new(),
            abilities: scores,
            saving_throw_proficiencies: saves.iter().map(|x| x.to_string()).collect(),
            skill_proficiencies: skills.iter().map(|x| x.to_string()).collect(),
            armor_class: 10,
            speed: 30,
            max_hp: 1,
            current_hp: 1,
            hit_dice: String::new(),
            equipment: Vec::new(),
            spells: Vec::new(),
            created_at: epoch,
            updated_at: epoch,
        }
    }

    #[test]
    fn ability_modifiers_follow_the_5e_table_including_odd_scores_below_10() {
        let cases = [
            (1, -5),
            (3, -4),
            (8, -1),
            (9, -1),
            (10, 0),
            (11, 0),
            (15, 2),
            (20, 5),
            (30, 10),
        ];
        for (score, expected) in cases {
            assert_eq!(ability_modifier(score), expected, "score {score}");
        }
    }

    #[test]
    fn proficiency_bonus_scales_and_clamps() {
        let cases = [
            (1, 2),
            (4, 2),
            (5, 3),
            (8, 3),
            (9, 4),
            (12, 4),
            (13, 5),
            (16, 5),
            (17, 6),
            (20, 6),
            // Out-of-range levels clamp onto the table (web JS did not
            // clamp; unifying fixes that latent divergence).
            (0, 2),
            (21, 6),
        ];
        for (level, expected) in cases {
            assert_eq!(proficiency_bonus(level), expected, "level {level}");
        }
    }

    #[test]
    fn proficiency_adds_the_bonus_exactly_once() {
        let s = sheet(5, &[("dexterity", 16)], &["dexterity"], &["stealth"]);
        assert_eq!(saving_throw_bonus(&s, "dexterity"), 6);
        assert_eq!(skill_bonus(&s, "stealth"), 6);
        // Unproficient skill on the same ability gets the bare modifier.
        assert_eq!(skill_bonus(&s, "acrobatics"), 3);
        // Unproficient save on a default-10 ability is flat 0.
        assert_eq!(saving_throw_bonus(&s, "wisdom"), 0);
    }

    #[test]
    fn initiative_and_passive_perception_derive_correctly() {
        let s = sheet(
            1,
            &[("dexterity", 14), ("wisdom", 12)],
            &[],
            &["perception"],
        );
        assert_eq!(initiative(&s), 2);
        assert_eq!(passive_perception(&s), 10 + 1 + 2); // 10 + WIS mod + prof
    }

    #[test]
    fn derive_stats_emits_all_axes_in_canonical_order() {
        let s = sheet(5, &[("dexterity", 16)], &["dexterity"], &["stealth"]);
        let d = derive_stats(&s);
        assert_eq!(d.proficiency_bonus, 3);
        assert_eq!(d.ability_modifiers.len(), 6);
        assert_eq!(d.ability_modifiers[1].name, "dexterity");
        assert_eq!(d.ability_modifiers[1].bonus, 3);
        assert_eq!(d.skill_bonuses.len(), 18);
        let stealth = d.skill_bonuses.iter().find(|b| b.name == "stealth").unwrap();
        assert_eq!(stealth.bonus, 6);
        assert_eq!(d.initiative, 3);
    }

    #[test]
    fn parses_the_mobile_wire_shape_and_computes() {
        // Exactly the JSON shape `CharacterSheet.toJson()` produces in the
        // Flutter app. Guards the Rust serde field names against silent
        // drift from the Dart wire contract — the whole point of the FFI
        // unification.
        let json = r#"{
            "uuid": "00000000-0000-0000-0000-000000000000",
            "name": "Thistle",
            "race": "",
            "class_name": "Rogue",
            "level": 5,
            "background": "",
            "alignment": "",
            "abilities": {"strength":10,"dexterity":16,"constitution":10,"intelligence":10,"wisdom":12,"charisma":10},
            "saving_throw_proficiencies": ["dexterity"],
            "skill_proficiencies": ["stealth","perception"],
            "armor_class": 12,
            "speed": 30,
            "max_hp": 27,
            "current_hp": 27,
            "hit_dice": "5d8",
            "equipment": [{"name":"Dagger","quantity":2,"notes":""}],
            "spells": [],
            "created_at": "2026-01-01T00:00:00.000Z",
            "updated_at": "2026-01-01T00:00:00.000Z"
        }"#;
        let sheet: CharacterSheet =
            serde_json::from_str(json).expect("Dart wire shape must deserialize");
        let d = derive_stats(&sheet);
        assert_eq!(d.proficiency_bonus, 3);
        // DEX 16 -> +3 modifier, save proficient -> +6.
        let dex_save = d
            .saving_throw_bonuses
            .iter()
            .find(|b| b.name == "dexterity")
            .unwrap();
        assert_eq!(dex_save.bonus, 6);
        // Stealth (DEX), proficient -> +6.
        let stealth = d.skill_bonuses.iter().find(|b| b.name == "stealth").unwrap();
        assert_eq!(stealth.bonus, 6);
        // Perception (WIS 12 -> +1), proficient -> +4; passive = 10 + 4.
        assert_eq!(d.passive_perception, 14);
    }
}
