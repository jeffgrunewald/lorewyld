//! Hand-authored archetype profiles for the new-player guidance quiz.
//!
//! Each profile positions one piece of content on the fixed scoring axes
//! defined in [`crate::guidance`] plus a set of flavor [`Theme`] tags.
//! The last three axes are personality — Solitude, Camaraderie, Devotion
//! — so the quiz can match who the character *is*, not just how they
//! fight. Lookups try the stable content `key` first, then fall back to
//! the lowercased display name so reprints of the same class or
//! background from another document still resolve. Content with no
//! profile is scored by the heuristic in `guidance.rs` instead.

use lorewyld_types::AbilityScore;

use crate::guidance::{AXIS_COUNT, Theme};

/// Archetype position of one class/species/background. Axis order matches
/// the `Axis` enum: Martial, Magic, Durability, Finesse, Support, Control,
/// Social, Stealth, Simplicity, Solitude, Camaraderie, Devotion — each
/// 0.0..=1.0.
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub axes: [f32; AXIS_COUNT],
    pub themes: &'static [Theme],
    /// Ability priority, best-first, driving the standard-array prefill.
    /// Populated on class profiles only; empty elsewhere.
    pub ability_priority: &'static [AbilityScore],
}

type Entry = (&'static str, &'static str, Profile);

/// A resolved profile plus how it matched: an exact stable-key hit, or
/// the lowercased-name fallback (reprints of the same content from
/// another document). Same-named duplicates tie on score, so the engine
/// prefers the exact-key copy when collapsing them.
#[derive(Debug, Clone, Copy)]
pub struct ProfileMatch {
    pub profile: &'static Profile,
    pub exact_key: bool,
}

fn find(table: &'static [Entry], key: &str, name: &str) -> Option<ProfileMatch> {
    let name = name.to_lowercase();
    table
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, _, p)| ProfileMatch { profile: p, exact_key: true })
        .or_else(|| {
            table
                .iter()
                .find(|(_, n, _)| *n == name)
                .map(|(_, _, p)| ProfileMatch { profile: p, exact_key: false })
        })
}

/// Profile for a base class, by stable key or lowercased name.
pub fn class_profile(key: &str, name: &str) -> Option<ProfileMatch> {
    find(CLASS_PROFILES, key, name)
}

/// Profile for a base species, by stable key or lowercased name.
pub fn species_profile(key: &str, name: &str) -> Option<ProfileMatch> {
    find(SPECIES_PROFILES, key, name)
}

/// Profile for a background, by stable key or lowercased name.
pub fn background_profile(key: &str, name: &str) -> Option<ProfileMatch> {
    find(BACKGROUND_PROFILES, key, name)
}

use AbilityScore::{Charisma, Constitution, Dexterity, Intelligence, Strength, Wisdom};
use Theme::{
    Arcane, Criminal, Divine, Maker, Nature, Noble, Performer, Scholarly, Wilderness,
};

//         axes:  Mar   Mag   Dur   Fin   Sup   Con   Soc   Ste   Sim   Sol   Cam   Dev
pub(crate) const CLASS_PROFILES: &[Entry] = &[
    ("srd-2024_barbarian", "barbarian", Profile {
        axes: [0.95, 0.00, 1.00, 0.20, 0.10, 0.10, 0.20, 0.10, 0.95, 0.60, 0.50, 0.30],
        themes: &[Wilderness, Nature],
        ability_priority: &[Strength, Constitution, Dexterity, Wisdom, Charisma, Intelligence],
    }),
    ("srd-2024_bard", "bard", Profile {
        axes: [0.20, 0.80, 0.20, 0.50, 0.80, 0.60, 1.00, 0.30, 0.30, 0.10, 0.80, 0.40],
        themes: &[Performer, Noble],
        ability_priority: &[Charisma, Dexterity, Constitution, Wisdom, Intelligence, Strength],
    }),
    ("srd-2024_cleric", "cleric", Profile {
        axes: [0.40, 0.85, 0.60, 0.10, 1.00, 0.50, 0.40, 0.05, 0.50, 0.20, 0.60, 1.00],
        themes: &[Divine],
        ability_priority: &[Wisdom, Constitution, Strength, Charisma, Intelligence, Dexterity],
    }),
    ("srd-2024_druid", "druid", Profile {
        axes: [0.25, 0.90, 0.40, 0.30, 0.70, 0.70, 0.20, 0.20, 0.35, 0.80, 0.30, 0.60],
        themes: &[Nature, Wilderness],
        ability_priority: &[Wisdom, Constitution, Dexterity, Intelligence, Strength, Charisma],
    }),
    ("srd-2024_fighter", "fighter", Profile {
        axes: [1.00, 0.05, 0.85, 0.60, 0.20, 0.30, 0.20, 0.10, 0.80, 0.30, 0.70, 0.50],
        themes: &[],
        ability_priority: &[Strength, Constitution, Dexterity, Wisdom, Intelligence, Charisma],
    }),
    ("srd-2024_monk", "monk", Profile {
        axes: [0.85, 0.15, 0.50, 0.90, 0.20, 0.30, 0.20, 0.50, 0.50, 0.70, 0.40, 0.50],
        themes: &[Scholarly],
        ability_priority: &[Dexterity, Wisdom, Constitution, Strength, Intelligence, Charisma],
    }),
    ("srd-2024_paladin", "paladin", Profile {
        axes: [0.90, 0.50, 0.90, 0.10, 0.60, 0.20, 0.50, 0.05, 0.55, 0.20, 0.60, 0.95],
        themes: &[Divine, Noble],
        ability_priority: &[Strength, Charisma, Constitution, Wisdom, Intelligence, Dexterity],
    }),
    ("srd-2024_ranger", "ranger", Profile {
        axes: [0.75, 0.40, 0.50, 0.85, 0.30, 0.40, 0.10, 0.60, 0.60, 0.90, 0.30, 0.40],
        themes: &[Nature, Wilderness],
        ability_priority: &[Dexterity, Wisdom, Constitution, Strength, Intelligence, Charisma],
    }),
    ("srd-2024_rogue", "rogue", Profile {
        axes: [0.50, 0.10, 0.25, 0.95, 0.20, 0.30, 0.60, 1.00, 0.65, 0.60, 0.40, 0.20],
        themes: &[Criminal],
        ability_priority: &[Dexterity, Intelligence, Constitution, Charisma, Wisdom, Strength],
    }),
    ("srd-2024_sorcerer", "sorcerer", Profile {
        axes: [0.05, 1.00, 0.15, 0.30, 0.30, 0.70, 0.60, 0.10, 0.45, 0.50, 0.40, 0.30],
        themes: &[Arcane],
        ability_priority: &[Charisma, Constitution, Dexterity, Intelligence, Wisdom, Strength],
    }),
    ("srd-2024_warlock", "warlock", Profile {
        axes: [0.15, 0.95, 0.25, 0.30, 0.20, 0.70, 0.60, 0.30, 0.55, 0.70, 0.30, 0.15],
        themes: &[Arcane],
        ability_priority: &[Charisma, Constitution, Dexterity, Intelligence, Wisdom, Strength],
    }),
    ("srd-2024_wizard", "wizard", Profile {
        axes: [0.05, 1.00, 0.10, 0.30, 0.30, 0.90, 0.20, 0.10, 0.15, 0.70, 0.30, 0.30],
        themes: &[Arcane, Scholarly],
        ability_priority: &[Intelligence, Constitution, Dexterity, Wisdom, Charisma, Strength],
    }),
    ("a5e_marshal", "marshal", Profile {
        axes: [0.85, 0.05, 0.70, 0.40, 0.90, 0.60, 0.60, 0.10, 0.50, 0.10, 1.00, 0.70],
        themes: &[Noble],
        ability_priority: &[Strength, Charisma, Constitution, Dexterity, Wisdom, Intelligence],
    }),
    ("bfrd_mechanist", "mechanist", Profile {
        axes: [0.50, 0.60, 0.50, 0.50, 0.40, 0.60, 0.20, 0.20, 0.20, 0.60, 0.40, 0.40],
        themes: &[Maker, Scholarly],
        ability_priority: &[Intelligence, Constitution, Dexterity, Strength, Wisdom, Charisma],
    }),
];

//         axes:  Mar   Mag   Dur   Fin   Sup   Con   Soc   Ste   Sim   Sol   Cam   Dev
pub(crate) const SPECIES_PROFILES: &[Entry] = &[
    ("srd-2024_dragonborn", "dragonborn", Profile {
        axes: [0.70, 0.40, 0.70, 0.20, 0.20, 0.20, 0.50, 0.10, 0.70, 0.40, 0.60, 0.60],
        themes: &[Noble],
        ability_priority: &[],
    }),
    ("srd-2024_dwarf", "dwarf", Profile {
        axes: [0.70, 0.20, 0.90, 0.20, 0.40, 0.20, 0.20, 0.20, 0.80, 0.30, 0.80, 0.60],
        themes: &[Maker],
        ability_priority: &[],
    }),
    ("srd-2024_elf", "elf", Profile {
        axes: [0.40, 0.70, 0.30, 0.80, 0.40, 0.50, 0.50, 0.60, 0.50, 0.60, 0.40, 0.40],
        themes: &[Nature, Arcane],
        ability_priority: &[],
    }),
    ("srd-2024_gnome", "gnome", Profile {
        axes: [0.20, 0.80, 0.30, 0.60, 0.40, 0.50, 0.50, 0.50, 0.40, 0.40, 0.60, 0.40],
        themes: &[Maker, Arcane],
        ability_priority: &[],
    }),
    ("srd-2024_goliath", "goliath", Profile {
        axes: [0.80, 0.00, 0.90, 0.10, 0.20, 0.10, 0.10, 0.10, 0.80, 0.60, 0.60, 0.40],
        themes: &[Wilderness],
        ability_priority: &[],
    }),
    ("srd-2024_halfling", "halfling", Profile {
        axes: [0.30, 0.20, 0.30, 0.90, 0.50, 0.20, 0.70, 0.80, 0.80, 0.20, 0.90, 0.60],
        themes: &[],
        ability_priority: &[],
    }),
    ("srd-2024_human", "human", Profile {
        axes: [0.50, 0.50, 0.50, 0.50, 0.50, 0.50, 0.60, 0.40, 0.80, 0.40, 0.60, 0.50],
        themes: &[Noble],
        ability_priority: &[],
    }),
    ("srd-2024_orc", "orc", Profile {
        axes: [0.90, 0.10, 0.85, 0.30, 0.20, 0.10, 0.20, 0.20, 0.85, 0.50, 0.60, 0.40],
        themes: &[Wilderness],
        ability_priority: &[],
    }),
    ("srd-2024_tiefling", "tiefling", Profile {
        axes: [0.30, 0.80, 0.30, 0.40, 0.30, 0.50, 0.70, 0.50, 0.50, 0.80, 0.30, 0.30],
        themes: &[Arcane, Criminal],
        ability_priority: &[],
    }),
    ("srd_half-elf", "half-elf", Profile {
        axes: [0.40, 0.50, 0.40, 0.50, 0.50, 0.40, 0.90, 0.40, 0.60, 0.60, 0.50, 0.50],
        themes: &[Noble, Performer],
        ability_priority: &[],
    }),
    ("srd_half-orc", "half-orc", Profile {
        axes: [0.85, 0.10, 0.80, 0.30, 0.20, 0.10, 0.30, 0.30, 0.85, 0.60, 0.40, 0.40],
        themes: &[Wilderness],
        ability_priority: &[],
    }),
    ("toh_alseid", "alseid", Profile {
        axes: [0.50, 0.30, 0.40, 0.80, 0.30, 0.20, 0.20, 0.60, 0.60, 0.70, 0.40, 0.40],
        themes: &[Nature, Wilderness],
        ability_priority: &[],
    }),
    ("toh_catfolk", "catfolk", Profile {
        axes: [0.50, 0.20, 0.30, 0.90, 0.20, 0.20, 0.50, 0.90, 0.70, 0.70, 0.40, 0.30],
        themes: &[Criminal, Wilderness],
        ability_priority: &[],
    }),
    ("toh_darakhul", "darakhul", Profile {
        axes: [0.60, 0.40, 0.70, 0.40, 0.10, 0.30, 0.20, 0.60, 0.50, 0.80, 0.30, 0.20],
        themes: &[Scholarly],
        ability_priority: &[],
    }),
    ("toh_derro", "derro", Profile {
        axes: [0.40, 0.60, 0.40, 0.50, 0.10, 0.50, 0.20, 0.60, 0.40, 0.90, 0.20, 0.10],
        themes: &[Arcane],
        ability_priority: &[],
    }),
    ("toh_drow", "drow", Profile {
        axes: [0.40, 0.60, 0.30, 0.80, 0.20, 0.50, 0.50, 0.80, 0.50, 0.70, 0.30, 0.20],
        themes: &[Criminal, Arcane],
        ability_priority: &[],
    }),
    ("toh_erina", "erina", Profile {
        axes: [0.30, 0.20, 0.50, 0.70, 0.50, 0.20, 0.50, 0.60, 0.80, 0.30, 0.80, 0.60],
        themes: &[Nature],
        ability_priority: &[],
    }),
    ("toh_gearforged", "gearforged", Profile {
        axes: [0.70, 0.30, 0.90, 0.20, 0.30, 0.30, 0.20, 0.10, 0.70, 0.60, 0.40, 0.50],
        themes: &[Maker],
        ability_priority: &[],
    }),
    ("toh_minotaur", "minotaur", Profile {
        axes: [0.90, 0.05, 0.85, 0.20, 0.20, 0.20, 0.20, 0.10, 0.80, 0.50, 0.50, 0.40],
        themes: &[Wilderness],
        ability_priority: &[],
    }),
    ("toh_mushroomfolk", "mushroomfolk", Profile {
        axes: [0.40, 0.50, 0.60, 0.30, 0.60, 0.40, 0.30, 0.30, 0.70, 0.20, 0.90, 0.50],
        themes: &[Nature],
        ability_priority: &[],
    }),
    ("toh_satarre", "satarre", Profile {
        axes: [0.40, 0.70, 0.40, 0.40, 0.20, 0.60, 0.30, 0.40, 0.40, 0.80, 0.30, 0.20],
        themes: &[Arcane],
        ability_priority: &[],
    }),
    ("toh_shade", "shade", Profile {
        axes: [0.30, 0.50, 0.20, 0.70, 0.20, 0.40, 0.40, 1.00, 0.50, 0.90, 0.20, 0.20],
        themes: &[Criminal],
        ability_priority: &[],
    }),
];

//         axes:  Mar   Mag   Dur   Fin   Sup   Con   Soc   Ste   Sim   Sol   Cam   Dev
pub(crate) const BACKGROUND_PROFILES: &[Entry] = &[
    ("srd-2024_acolyte", "acolyte", Profile {
        axes: [0.20, 0.50, 0.20, 0.10, 0.80, 0.20, 0.50, 0.10, 0.70, 0.20, 0.60, 0.90],
        themes: &[Divine],
        ability_priority: &[],
    }),
    ("srd-2024_criminal", "criminal", Profile {
        axes: [0.30, 0.10, 0.20, 0.60, 0.10, 0.20, 0.40, 0.90, 0.70, 0.60, 0.40, 0.10],
        themes: &[Criminal],
        ability_priority: &[],
    }),
    ("srd-2024_sage", "sage", Profile {
        axes: [0.05, 0.70, 0.10, 0.20, 0.40, 0.50, 0.40, 0.10, 0.50, 0.70, 0.20, 0.30],
        themes: &[Scholarly, Arcane],
        ability_priority: &[],
    }),
    ("srd-2024_soldier", "soldier", Profile {
        axes: [0.90, 0.00, 0.70, 0.30, 0.40, 0.30, 0.30, 0.10, 0.80, 0.20, 0.90, 0.60],
        themes: &[],
        ability_priority: &[],
    }),
    ("a5e-ag_artisan", "artisan", Profile {
        axes: [0.20, 0.20, 0.30, 0.50, 0.40, 0.20, 0.50, 0.10, 0.70, 0.50, 0.50, 0.40],
        themes: &[Maker],
        ability_priority: &[],
    }),
    ("a5e-ag_charlatan", "charlatan", Profile {
        axes: [0.10, 0.20, 0.10, 0.50, 0.20, 0.30, 0.90, 0.70, 0.60, 0.60, 0.30, 0.05],
        themes: &[Criminal, Performer],
        ability_priority: &[],
    }),
    ("a5e-ag_cultist", "cultist", Profile {
        axes: [0.20, 0.70, 0.20, 0.20, 0.20, 0.40, 0.40, 0.50, 0.50, 0.40, 0.60, 0.40],
        themes: &[Arcane, Divine],
        ability_priority: &[],
    }),
    ("a5e-ag_entertainer", "entertainer", Profile {
        axes: [0.20, 0.20, 0.20, 0.60, 0.50, 0.20, 0.90, 0.30, 0.70, 0.10, 0.70, 0.30],
        themes: &[Performer],
        ability_priority: &[],
    }),
    ("a5e-ag_exile", "exile", Profile {
        axes: [0.40, 0.30, 0.50, 0.50, 0.20, 0.20, 0.30, 0.50, 0.60, 0.95, 0.20, 0.30],
        themes: &[Wilderness],
        ability_priority: &[],
    }),
    ("a5e-ag_farmer", "farmer", Profile {
        axes: [0.50, 0.10, 0.70, 0.30, 0.50, 0.10, 0.30, 0.10, 0.90, 0.50, 0.70, 0.60],
        themes: &[Nature],
        ability_priority: &[],
    }),
    ("a5e-ag_folk-hero", "folk hero", Profile {
        axes: [0.60, 0.10, 0.60, 0.40, 0.60, 0.20, 0.60, 0.20, 0.80, 0.30, 0.70, 0.90],
        themes: &[Nature],
        ability_priority: &[],
    }),
    ("a5e-ag_gambler", "gambler", Profile {
        axes: [0.20, 0.10, 0.20, 0.60, 0.20, 0.30, 0.80, 0.50, 0.70, 0.50, 0.40, 0.10],
        themes: &[Criminal, Performer],
        ability_priority: &[],
    }),
    ("a5e-ag_guard", "guard", Profile {
        axes: [0.70, 0.00, 0.70, 0.30, 0.40, 0.30, 0.30, 0.20, 0.85, 0.20, 0.80, 0.70],
        themes: &[],
        ability_priority: &[],
    }),
    ("a5e-ag_guildmember", "guildmember", Profile {
        axes: [0.30, 0.20, 0.30, 0.40, 0.50, 0.30, 0.70, 0.20, 0.70, 0.20, 0.80, 0.40],
        themes: &[Maker, Noble],
        ability_priority: &[],
    }),
    ("a5e-ag_hermit", "hermit", Profile {
        axes: [0.20, 0.60, 0.40, 0.30, 0.50, 0.30, 0.10, 0.30, 0.60, 1.00, 0.10, 0.40],
        themes: &[Nature, Divine],
        ability_priority: &[],
    }),
    ("a5e-ag_marauder", "marauder", Profile {
        axes: [0.80, 0.00, 0.70, 0.50, 0.10, 0.20, 0.20, 0.40, 0.80, 0.50, 0.60, 0.10],
        themes: &[Wilderness, Criminal],
        ability_priority: &[],
    }),
    ("a5e-ag_noble", "noble", Profile {
        axes: [0.30, 0.20, 0.20, 0.20, 0.40, 0.40, 0.90, 0.10, 0.70, 0.20, 0.60, 0.30],
        themes: &[Noble],
        ability_priority: &[],
    }),
    ("a5e-ag_outlander", "outlander", Profile {
        axes: [0.70, 0.10, 0.70, 0.60, 0.20, 0.10, 0.10, 0.40, 0.80, 0.90, 0.30, 0.30],
        themes: &[Wilderness, Nature],
        ability_priority: &[],
    }),
    ("a5e-ag_sailor", "sailor", Profile {
        axes: [0.60, 0.00, 0.60, 0.60, 0.30, 0.20, 0.40, 0.30, 0.80, 0.30, 0.80, 0.40],
        themes: &[Wilderness],
        ability_priority: &[],
    }),
    ("a5e-ag_trader", "trader", Profile {
        axes: [0.20, 0.10, 0.20, 0.30, 0.40, 0.20, 0.90, 0.20, 0.80, 0.30, 0.60, 0.30],
        themes: &[Noble, Maker],
        ability_priority: &[],
    }),
    ("a5e-ag_urchin", "urchin", Profile {
        axes: [0.20, 0.10, 0.30, 0.70, 0.30, 0.10, 0.40, 0.90, 0.80, 0.80, 0.30, 0.30],
        themes: &[Criminal],
        ability_priority: &[],
    }),
];
