//! 5e multiclass prerequisite checks (the "primary ability" 13+ rule).
//! Prerequisites are OR-of-AND ability groups read from the record's
//! `primary_abilities` field — either a class content record or a sheet
//! class entry carrying its pick-time snapshot; both share the field.

use lorewyld_types::common::AbilityScores;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::sheet::ability_score;

const MINIMUM_SCORE: i32 = 13;

/// One class's prerequisite verdict from [`check_multiclass`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrereqResult {
    pub name: String,
    pub ok: bool,
    /// Human requirement text, e.g. "Strength 13 or Dexterity 13";
    /// "" when the class has no prerequisite.
    pub requirement: String,
}

/// Prerequisite groups from a record's `primary_abilities`; empty (=
/// no prerequisite) when the field is absent or malformed.
pub fn prereq_groups(class_record: &Value) -> Vec<Vec<String>> {
    class_record
        .get("primary_abilities")
        .and_then(Value::as_array)
        .map(|groups| {
            groups
                .iter()
                .filter_map(Value::as_array)
                .map(|group| {
                    group
                        .iter()
                        .filter_map(Value::as_str)
                        .map(|a| a.to_lowercase())
                        .collect::<Vec<String>>()
                })
                .filter(|group| !group.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// RAW multiclassing: the character must meet the prerequisite of the
/// class being added AND of every class already held, so callers pass
/// every relevant class record. No prerequisite = always ok.
pub fn check_multiclass(abilities: &AbilityScores, class_records: &[Value]) -> Vec<PrereqResult> {
    class_records
        .iter()
        .map(|record| {
            let groups = prereq_groups(record);
            let ok = groups.is_empty()
                || groups.iter().any(|group| {
                    group
                        .iter()
                        .all(|a| ability_score(abilities, a) >= MINIMUM_SCORE)
                });
            PrereqResult {
                name: record
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                ok,
                requirement: requirement_text(&groups),
            }
        })
        .collect()
}

/// Formats OR-of-AND groups as prose: `[[dex, wis]]` -> "Dexterity 13
/// and Wisdom 13"; `[[str], [dex]]` -> "Strength 13 or Dexterity 13".
fn requirement_text(groups: &[Vec<String>]) -> String {
    groups
        .iter()
        .map(|group| {
            group
                .iter()
                .map(|a| format!("{} {MINIMUM_SCORE}", capitalize(a)))
                .collect::<Vec<String>>()
                .join(" and ")
        })
        .collect::<Vec<String>>()
        .join(" or ")
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn scores(pairs: &[(&str, i32)]) -> AbilityScores {
        let mut s = AbilityScores {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        };
        for (name, value) in pairs {
            match *name {
                "strength" => s.strength = *value,
                "dexterity" => s.dexterity = *value,
                "constitution" => s.constitution = *value,
                "intelligence" => s.intelligence = *value,
                "wisdom" => s.wisdom = *value,
                "charisma" => s.charisma = *value,
                other => panic!("unknown ability {other}"),
            }
        }
        s
    }

    #[test]
    fn any_of_group_passes_when_either_ability_qualifies() {
        // Fighter (STR or DEX): STR 11 fails alone, DEX 14 carries it.
        let fighter = json!({
            "name": "Fighter",
            "primary_abilities": [["strength"], ["dexterity"]],
        });
        let results = check_multiclass(&scores(&[("strength", 11), ("dexterity", 14)]), &[fighter]);
        assert!(results[0].ok);
        assert_eq!(results[0].requirement, "Strength 13 or Dexterity 13");
    }

    #[test]
    fn all_of_group_requires_every_ability() {
        // Monk (DEX and WIS): WIS 12 fails the whole group.
        let monk = json!({
            "name": "Monk",
            "primary_abilities": [["dexterity", "wisdom"]],
        });
        let failing =
            check_multiclass(&scores(&[("dexterity", 14), ("wisdom", 12)]), &[monk.clone()]);
        assert!(!failing[0].ok);
        assert_eq!(failing[0].requirement, "Dexterity 13 and Wisdom 13");

        let passing = check_multiclass(&scores(&[("dexterity", 14), ("wisdom", 13)]), &[monk]);
        assert!(passing[0].ok);
    }

    #[test]
    fn record_without_the_field_has_no_prerequisite() {
        // Content the author gave no prerequisite — or a sheet entry
        // whose snapshot is empty — is always allowed.
        let homebrew = json!({"name": "Bloodhunter"});
        let results = check_multiclass(&scores(&[]), &[homebrew]);
        assert!(results[0].ok);
        assert_eq!(results[0].requirement, "");
    }

    #[test]
    fn sheet_entry_snapshot_shape_is_accepted() {
        // A CharacterClassEntry serialized to JSON carries the same
        // primary_abilities field a content record does.
        let entry = json!({
            "name": "Paladin",
            "level": 5,
            "subclass": "",
            "starting": true,
            "primary_abilities": [["strength", "charisma"]],
        });
        let results = check_multiclass(&scores(&[("strength", 14)]), &[entry]);
        assert!(!results[0].ok);
        assert_eq!(results[0].requirement, "Strength 13 and Charisma 13");
    }

    #[test]
    fn every_class_in_the_set_is_checked() {
        // Current Wizard (INT 15 ok) + new Paladin (STR and CHA, fails).
        let records = [
            json!({"name": "Wizard", "primary_abilities": [["intelligence"]]}),
            json!({"name": "Paladin", "primary_abilities": [["strength", "charisma"]]}),
        ];
        let results =
            check_multiclass(&scores(&[("intelligence", 15), ("strength", 14)]), &records);
        assert!(results[0].ok);
        assert!(!results[1].ok);
        assert_eq!(results[1].requirement, "Strength 13 and Charisma 13");
    }
}
