//! New-player guidance quiz: a weighted questionnaire that recommends a
//! class, species, and background from whatever content the client has
//! installed, plus ability-score priorities and an alignment nudge.
//!
//! Every client renders the fixed [`Questionnaire`] returned by
//! [`guidance_questionnaire`], collects [`Answer`]s, and passes them with
//! its candidate records to [`recommend`]. Answer weights are engine
//! internals — they never cross the wire, so tuning them is not a client
//! contract. Known content is scored against the hand-authored profiles
//! in [`crate::guidance_profiles`]; anything unmapped (homebrew, third
//! party) gets a heuristic profile derived from its mechanical fields so
//! it still surfaces. All output is prefill advice: the sheet documents
//! and computes but never enforces.

use lorewyld_types::{AbilityScore, AbilityScores};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::guidance_profiles::{
    ProfileMatch, background_profile, class_profile, species_profile,
};

/// Bumped when questions/weights change enough that stored answers from
/// an older quiz should not be replayed against the new engine.
pub const GUIDANCE_VERSION: u32 = 2;

/// Number of numeric scoring axes; array order matches [`Axis`].
pub const AXIS_COUNT: usize = 12;

/// The fixed scoring axes. Profiles hold a 0.0..=1.0 position per axis;
/// answers accumulate weights into a user vector on the same axes.
///
/// The first nine describe play style and mechanics. The last three are
/// personality: who the character *is*, not how they fight — so the quiz
/// can steer toward the person the player wants to inhabit, whether
/// that's familiar or a deliberate departure from themselves. Opposing
/// poles get their own axis (Solitude vs Camaraderie, like Simplicity)
/// because the weighted sum only rewards presence, never absence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Martial,
    Magic,
    Durability,
    Finesse,
    Support,
    Control,
    Social,
    Stealth,
    Simplicity,
    /// Self-sufficient loner: walks alone, answers to no one.
    Solitude,
    /// Thrives on bonds: companies, guilds, found families.
    Camaraderie,
    /// Serves others as identity — duty of care beyond combat healing.
    Devotion,
}

/// Flavor tags matched by set overlap, separate from the numeric axes:
/// flavor is categorical, not gradable, and keeping it out of the dot
/// product keeps hand-authoring ~60 profiles tractable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Nature,
    Divine,
    Arcane,
    Criminal,
    Noble,
    Wilderness,
    Scholarly,
    Performer,
    Maker,
}

/// One selectable answer. `value` is the stable wire form; `label` is the
/// human-facing text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerOption {
    pub value: String,
    pub label: String,
}

/// One quiz question.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    /// Stable key referenced by [`Answer::question`].
    pub key: String,
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    pub options: Vec<AnswerOption>,
}

/// The fixed question set, versioned for forward compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Questionnaire {
    pub version: u32,
    /// Framing shown before the first question: answers describe the
    /// character the player wants to inhabit, not the player.
    #[serde(default)]
    pub intro: String,
    pub questions: Vec<Question>,
}

/// A player's answer to one question.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
    /// Matches [`Question::key`].
    pub question: String,
    /// Matches [`AnswerOption::value`].
    pub value: String,
}

/// One ranked suggestion within a category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recommendation {
    /// Echoed from the candidate record so clients can fetch/prefill;
    /// empty when the candidate carried no uuid.
    pub uuid: String,
    pub key: String,
    pub name: String,
    /// 0.0..=1.0 fit against the player's answers.
    pub score: f32,
    /// "Why this fits" sentences; always non-empty.
    pub reasons: Vec<String>,
    /// False when the score came from the mechanical heuristic rather
    /// than a hand-authored profile.
    pub mapped: bool,
}

/// The full quiz result: top picks per category plus prefill advice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationSet {
    pub version: u32,
    pub classes: Vec<Recommendation>,
    pub species: Vec<Recommendation>,
    pub backgrounds: Vec<Recommendation>,
    /// Best-first ability order for the top class suggestion.
    pub ability_priorities: Vec<AbilityScore>,
    /// Standard array 15/14/13/12/10/8 dealt out in priority order.
    pub standard_array_suggestion: AbilityScores,
    /// Wire alignment name matching the bundle's alignment records
    /// (e.g. `"chaotic_good"`, `"true_neutral"`).
    pub alignment_suggestion: String,
    pub alignment_reason: String,
}

/// Candidate records per category, exactly as the client already holds
/// them — full records (mobile) or list summaries (web). Only `key` and
/// `name` are required per record; mechanical fields sharpen the
/// heuristic when present. Records that fail to parse are skipped.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct CandidateSets {
    #[serde(default)]
    pub classes: Vec<Value>,
    #[serde(default)]
    pub species: Vec<Value>,
    #[serde(default)]
    pub backgrounds: Vec<Value>,
}

// ─── Questionnaire content ───────────────────────────────────────────────

fn question(key: &str, prompt: &str, options: &[(&str, &str)]) -> Question {
    Question {
        key: key.to_string(),
        prompt: prompt.to_string(),
        help: None,
        options: options
            .iter()
            .map(|(value, label)| AnswerOption {
                value: (*value).to_string(),
                label: (*label).to_string(),
            })
            .collect(),
    }
}

/// The fixed quiz shown by every client. Play-style questions address
/// the player ("how much bookkeeping do *you* enjoy"); identity and
/// personality questions address the character, so the player can build
/// someone just like themselves or try on a different person entirely.
pub fn guidance_questionnaire() -> Questionnaire {
    Questionnaire {
        version: GUIDANCE_VERSION,
        intro: "Answer for the character you want to play. They can be \
                just like you — or someone completely different you've \
                always wanted to try being."
            .to_string(),
        questions: vec![
            question(
                "hero_fantasy",
                "When you imagine your character, what's the picture?",
                &[
                    ("warrior", "A mighty warrior standing against impossible odds"),
                    ("trickster", "A cunning trickster nobody sees coming"),
                    ("wielder", "A wielder of strange and wondrous powers"),
                    ("guide", "A wise guide holding the group together"),
                    ("charmer", "A silver-tongued charmer who owns every room"),
                ],
            ),
            question(
                "combat_approach",
                "A fight breaks out. What is your character doing?",
                &[
                    ("charge", "Charging in, weapon swinging"),
                    ("ranged", "Picking targets off from a distance"),
                    ("spells", "Unleashing spells that change the battle"),
                    ("protect", "Keeping my friends standing"),
                    ("avoid", "Honestly? Looking for a way to end it without fighting"),
                ],
            ),
            question(
                "party_role",
                "What job does your character want in the group?",
                &[
                    ("protector", "The protector everyone hides behind"),
                    ("striker", "The damage-dealer who ends fights"),
                    ("healer", "The healer and enabler who keeps everyone going"),
                    ("controller", "The tactician who controls the battlefield"),
                    ("face", "The face who does the talking"),
                ],
            ),
            question(
                "problem_solving",
                "A locked door stands between your character and their goal.",
                &[
                    ("talk", "Talk their way past whoever holds the key"),
                    ("sneak", "Pick the lock — or find a way nobody thought of"),
                    ("magic", "They know a spell for this"),
                    ("force", "Break it down"),
                    ("research", "Find out who built it and what they were hiding"),
                ],
            ),
            question(
                "social_energy",
                "Your character walks into a crowded tavern. Where are they an hour later?",
                &[
                    ("hold_court", "Holding court at the center table, mid-story"),
                    ("one_friend", "Deep in conversation with one trusted friend"),
                    ("observer", "In the corner booth, watching everyone"),
                    ("reluctant", "Outside — they only came because the group insisted"),
                ],
            ),
            question(
                "belonging",
                "What's your character's ideal life?",
                &[
                    ("found_family", "A found family — tight-knit, loyal, always at their back"),
                    ("cause", "A cause, faith, or order worth giving themselves to"),
                    ("open_road", "The open road, answering to no one"),
                    ("quiet_craft", "A quiet workshop or study, mastering their craft"),
                ],
            ),
            question(
                "service",
                "A stranger begs for help at the worst possible moment. Your character…",
                &[
                    ("always_helps", "Drops everything — helping people is who they are"),
                    ("own_people", "Helps, but only once their companions are safe"),
                    ("for_a_price", "Names a price — kindness doesn't pay for itself"),
                    ("self_reliance", "Walks on; people must learn to stand on their own"),
                ],
            ),
            question(
                "complexity",
                "How much bookkeeping do you enjoy?",
                &[
                    ("simple", "Keep it simple — I want to roll dice and have fun"),
                    ("moderate", "A few options to weigh each turn"),
                    ("deep", "Give me all the levers — I love mastering systems"),
                ],
            ),
            question(
                "theme",
                "Which world calls to your character?",
                &[
                    ("nature", "Deep forests and wild beasts"),
                    ("divine", "Temples and higher powers"),
                    ("scholarly", "Dusty libraries and forgotten lore"),
                    ("criminal", "Back alleys and daring heists"),
                    ("noble", "Courts, banners, and high society"),
                    ("maker", "Workshops and ingenious devices"),
                ],
            ),
            question(
                "morality",
                "Which statement sounds most like your character?",
                &[
                    ("principled", "Rules and codes exist for good reasons"),
                    ("kindhearted", "Do what's right, whatever the rules say"),
                    ("free_spirit", "Freedom first — theirs and everyone else's"),
                    ("even_keeled", "Keep your word, keep the balance"),
                    ("self_reliant", "Look out for yourself; nobody else will"),
                ],
            ),
            question(
                "durability_style",
                "How does your character feel about getting hit?",
                &[
                    ("wall", "Fine — they're the wall"),
                    ("dodge", "They'd rather be too quick to hit"),
                    ("distance", "They plan to be far away from the hitting"),
                ],
            ),
            question(
                "magic_affinity",
                "How magical should your character be?",
                &[
                    ("full", "Magic is the whole point"),
                    ("some", "A little magic on the side"),
                    ("none", "None — steel and skill"),
                ],
            ),
        ],
    }
}

// ─── Answer weights (engine-internal, never on the wire) ─────────────────

struct Effect {
    axes: &'static [(Axis, f32)],
    themes: &'static [Theme],
    /// (law↔chaos, good↔evil): −1 lawful / +1 chaotic; +1 good / −1 evil.
    alignment: Option<(i8, i8)>,
}

const fn effect(axes: &'static [(Axis, f32)], themes: &'static [Theme]) -> Effect {
    Effect { axes, themes, alignment: None }
}

fn option_effect(question: &str, value: &str) -> Option<Effect> {
    use Axis::{
        Camaraderie, Control, Devotion, Durability, Finesse, Magic, Martial, Simplicity,
        Social, Solitude, Stealth, Support,
    };
    let e = match (question, value) {
        ("hero_fantasy", "warrior") => effect(&[(Martial, 0.9), (Durability, 0.7)], &[]),
        ("hero_fantasy", "trickster") => {
            effect(&[(Stealth, 0.9), (Finesse, 0.6)], &[Theme::Criminal])
        }
        ("hero_fantasy", "wielder") => effect(&[(Magic, 1.0)], &[Theme::Arcane]),
        ("hero_fantasy", "guide") => effect(
            &[(Support, 0.8), (Social, 0.5), (Devotion, 0.6), (Camaraderie, 0.4)],
            &[Theme::Divine],
        ),
        ("hero_fantasy", "charmer") => {
            effect(&[(Social, 0.9)], &[Theme::Performer, Theme::Noble])
        }

        ("combat_approach", "charge") => effect(&[(Martial, 0.9), (Durability, 0.8)], &[]),
        ("combat_approach", "ranged") => effect(&[(Finesse, 0.9), (Martial, 0.5)], &[]),
        ("combat_approach", "spells") => effect(&[(Magic, 0.9), (Control, 0.6)], &[]),
        ("combat_approach", "protect") => {
            effect(&[(Support, 0.9), (Devotion, 0.5), (Camaraderie, 0.4)], &[])
        }
        ("combat_approach", "avoid") => effect(&[(Social, 0.7), (Control, 0.4)], &[]),

        ("party_role", "protector") => {
            effect(&[(Durability, 0.9), (Martial, 0.5), (Devotion, 0.4)], &[])
        }
        ("party_role", "striker") => effect(&[(Martial, 0.6), (Finesse, 0.6)], &[]),
        ("party_role", "healer") => effect(&[(Support, 1.0), (Devotion, 0.5)], &[]),
        ("party_role", "controller") => effect(&[(Control, 1.0), (Magic, 0.6)], &[]),
        ("party_role", "face") => effect(&[(Social, 1.0)], &[]),

        ("social_energy", "hold_court") => {
            effect(&[(Social, 0.8), (Camaraderie, 0.6)], &[Theme::Performer])
        }
        ("social_energy", "one_friend") => effect(&[(Camaraderie, 0.7)], &[]),
        ("social_energy", "observer") => effect(&[(Solitude, 0.6), (Stealth, 0.4)], &[]),
        ("social_energy", "reluctant") => effect(&[(Solitude, 0.9)], &[]),

        ("belonging", "found_family") => effect(&[(Camaraderie, 1.0)], &[]),
        ("belonging", "cause") => {
            effect(&[(Devotion, 0.9), (Camaraderie, 0.3)], &[Theme::Divine])
        }
        ("belonging", "open_road") => {
            effect(&[(Solitude, 0.9)], &[Theme::Wilderness])
        }
        ("belonging", "quiet_craft") => {
            effect(&[(Solitude, 0.6)], &[Theme::Maker, Theme::Scholarly])
        }

        ("service", "always_helps") => effect(&[(Devotion, 1.0), (Support, 0.3)], &[]),
        ("service", "own_people") => effect(&[(Camaraderie, 0.8), (Devotion, 0.4)], &[]),
        ("service", "for_a_price") => {
            effect(&[(Social, 0.4), (Solitude, 0.3)], &[Theme::Criminal])
        }
        ("service", "self_reliance") => effect(&[(Solitude, 0.7)], &[]),

        ("problem_solving", "talk") => effect(&[(Social, 0.8)], &[]),
        ("problem_solving", "sneak") => effect(&[(Stealth, 0.8), (Finesse, 0.5)], &[]),
        ("problem_solving", "magic") => effect(&[(Magic, 0.8)], &[]),
        ("problem_solving", "force") => effect(&[(Martial, 0.7), (Durability, 0.5)], &[]),
        ("problem_solving", "research") => effect(&[(Control, 0.3)], &[Theme::Scholarly]),

        ("complexity", "simple") => effect(&[(Simplicity, 1.0)], &[]),
        ("complexity", "moderate") => effect(&[(Simplicity, 0.5)], &[]),
        ("complexity", "deep") => effect(&[(Magic, 0.3)], &[]),

        ("theme", "nature") => effect(&[], &[Theme::Nature, Theme::Wilderness]),
        ("theme", "divine") => effect(&[], &[Theme::Divine]),
        ("theme", "scholarly") => effect(&[], &[Theme::Arcane, Theme::Scholarly]),
        ("theme", "criminal") => effect(&[(Stealth, 0.4)], &[Theme::Criminal]),
        ("theme", "noble") => effect(&[(Social, 0.4)], &[Theme::Noble]),
        ("theme", "maker") => effect(&[], &[Theme::Maker]),

        ("morality", "principled") => Effect { alignment: Some((-1, 1)), ..effect(&[], &[]) },
        ("morality", "kindhearted") => {
            Effect { alignment: Some((0, 1)), ..effect(&[(Devotion, 0.3)], &[]) }
        }
        ("morality", "free_spirit") => Effect { alignment: Some((1, 1)), ..effect(&[], &[]) },
        ("morality", "even_keeled") => Effect { alignment: Some((-1, 0)), ..effect(&[], &[]) },
        ("morality", "self_reliant") => {
            Effect { alignment: Some((1, -1)), ..effect(&[(Solitude, 0.5)], &[]) }
        }

        ("durability_style", "wall") => effect(&[(Durability, 1.0)], &[]),
        ("durability_style", "dodge") => effect(&[(Finesse, 0.9), (Stealth, 0.4)], &[]),
        ("durability_style", "distance") => {
            effect(&[(Magic, 0.5), (Control, 0.4), (Finesse, 0.3)], &[])
        }

        ("magic_affinity", "full") => effect(&[(Magic, 1.0)], &[]),
        ("magic_affinity", "some") => effect(&[(Magic, 0.5), (Martial, 0.4)], &[]),
        ("magic_affinity", "none") => effect(&[(Martial, 0.8), (Simplicity, 0.4)], &[]),

        _ => return None,
    };
    Some(e)
}

// ─── Candidate parsing ───────────────────────────────────────────────────

/// Permissive view over whatever record JSON a client holds. Only `key`
/// and `name` are required; everything else sharpens the heuristic.
#[derive(Debug, Deserialize)]
struct RawCandidate {
    key: String,
    name: String,
    #[serde(default)]
    uuid: Option<String>,
    #[serde(default)]
    subclass_of: Option<Value>,
    #[serde(default)]
    is_subspecies: bool,
    #[serde(default)]
    subspecies_of: Option<Value>,
    #[serde(default)]
    hit_dice: Option<u8>,
    #[serde(default)]
    caster_type: Option<String>,
    #[serde(default)]
    spellcasting_ability: Option<String>,
    #[serde(default)]
    prof_skills: Option<Value>,
    #[serde(default)]
    asi: Option<Value>,
    #[serde(default)]
    traits: Option<Value>,
    #[serde(default)]
    benefits: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    Class,
    Species,
    Background,
}

impl RawCandidate {
    /// Base records only: the quiz never recommends a subclass or
    /// subspecies directly.
    fn is_base(&self, category: Category) -> bool {
        match category {
            Category::Class => self.subclass_of.is_none(),
            Category::Species => !self.is_subspecies && self.subspecies_of.is_none(),
            Category::Background => true,
        }
    }

    fn profile(&self, category: Category) -> Option<ProfileMatch> {
        match category {
            Category::Class => class_profile(&self.key, &self.name),
            Category::Species => species_profile(&self.key, &self.name),
            Category::Background => background_profile(&self.key, &self.name),
        }
    }
}

// ─── Heuristic profile for unmapped content ──────────────────────────────

struct HeuristicProfile {
    axes: [f32; AXIS_COUNT],
    themes: Vec<Theme>,
}

fn raise(axes: &mut [f32; AXIS_COUNT], axis: Axis, value: f32) {
    let slot = &mut axes[axis as usize];
    *slot = slot.max(value);
}

fn add_theme(themes: &mut Vec<Theme>, theme: Theme) {
    if !themes.contains(&theme) {
        themes.push(theme);
    }
}

/// Skill names → axis/theme signals, shared by class `prof_skills` and
/// background benefit prose.
fn apply_skill_keywords(text: &str, axes: &mut [f32; AXIS_COUNT], themes: &mut Vec<Theme>) {
    let text = text.to_lowercase();
    if text.contains("stealth") || text.contains("sleight") {
        raise(axes, Axis::Stealth, 0.7);
    }
    if text.contains("deception") || text.contains("persuasion") || text.contains("performance") {
        raise(axes, Axis::Social, 0.7);
    }
    if text.contains("intimidation") {
        raise(axes, Axis::Social, 0.5);
    }
    if text.contains("medicine") || text.contains("insight") {
        raise(axes, Axis::Support, 0.6);
    }
    if text.contains("athletics") {
        raise(axes, Axis::Martial, 0.6);
    }
    if text.contains("arcana") {
        raise(axes, Axis::Magic, 0.5);
        add_theme(themes, Theme::Arcane);
    }
    if text.contains("religion") {
        add_theme(themes, Theme::Divine);
    }
    if text.contains("nature") || text.contains("survival") || text.contains("animal") {
        add_theme(themes, Theme::Nature);
    }
    if text.contains("history") || text.contains("investigation") {
        add_theme(themes, Theme::Scholarly);
    }
}

/// Trait/feature names → axis/theme signals.
fn apply_trait_keywords(name: &str, axes: &mut [f32; AXIS_COUNT], themes: &mut Vec<Theme>) {
    let name = name.to_lowercase();
    if name.contains("stealth") || name.contains("sneak") || name.contains("shadow") {
        raise(axes, Axis::Stealth, 0.7);
    }
    if name.contains("darkvision") {
        raise(axes, Axis::Stealth, 0.5);
    }
    if name.contains("brave")
        || name.contains("tough")
        || name.contains("powerful")
        || name.contains("relentless")
        || name.contains("endurance")
    {
        raise(axes, Axis::Durability, 0.7);
    }
    if name.contains("spell") || name.contains("magic") || name.contains("cantrip") {
        raise(axes, Axis::Magic, 0.6);
        add_theme(themes, Theme::Arcane);
    }
}

/// Values of an `{"items": [...]}`-or-array JSON field's `name`/`desc`
/// string members, flattened for keyword scanning.
fn json_names(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .flat_map(|item| {
                    ["name", "desc"]
                        .iter()
                        .filter_map(|k| item.get(k).and_then(Value::as_str))
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Builds an approximate profile from whatever mechanical fields the
/// candidate carries. With nothing to go on, a flat 0.4 baseline lands
/// summary-only homebrew mid-pack rather than at zero.
fn heuristic_profile(c: &RawCandidate) -> HeuristicProfile {
    let mut axes = [0.4f32; AXIS_COUNT];
    let mut themes = Vec::new();

    let is_caster = c.spellcasting_ability.is_some()
        || c.caster_type.as_deref().is_some_and(|t| t.eq_ignore_ascii_case("full"));
    if is_caster {
        raise(&mut axes, Axis::Magic, 0.8);
        match c.spellcasting_ability.as_deref() {
            Some("intelligence") => {
                raise(&mut axes, Axis::Control, 0.6);
                add_theme(&mut themes, Theme::Arcane);
                add_theme(&mut themes, Theme::Scholarly);
            }
            Some("wisdom") => {
                raise(&mut axes, Axis::Support, 0.6);
                add_theme(&mut themes, Theme::Nature);
                add_theme(&mut themes, Theme::Divine);
            }
            Some("charisma") => {
                raise(&mut axes, Axis::Social, 0.6);
                add_theme(&mut themes, Theme::Arcane);
            }
            _ => {}
        }
    }

    if let Some(hd) = c.hit_dice {
        if hd >= 10 {
            raise(&mut axes, Axis::Martial, 0.7);
            raise(&mut axes, Axis::Durability, 0.7);
        }
        if hd >= 12 {
            raise(&mut axes, Axis::Durability, 0.9);
            raise(&mut axes, Axis::Simplicity, 0.7);
        }
        if hd <= 6 {
            raise(&mut axes, Axis::Magic, 0.6);
            axes[Axis::Durability as usize] = 0.15;
        }
    }

    if let Some(skills) = c.prof_skills.as_ref().and_then(|v| v.get("from")).and_then(Value::as_array)
    {
        for skill in skills.iter().filter_map(Value::as_str) {
            apply_skill_keywords(skill, &mut axes, &mut themes);
        }
    }

    if let Some(asi) = c.asi.as_ref().and_then(Value::as_object) {
        let boosted = |ability: &str| asi.get(ability).and_then(Value::as_i64).unwrap_or(0) > 0;
        if boosted("strength") || boosted("constitution") {
            raise(&mut axes, Axis::Martial, 0.6);
            raise(&mut axes, Axis::Durability, 0.6);
        }
        if boosted("dexterity") {
            raise(&mut axes, Axis::Finesse, 0.7);
            raise(&mut axes, Axis::Stealth, 0.5);
        }
        if boosted("charisma") {
            raise(&mut axes, Axis::Social, 0.7);
        }
        if boosted("intelligence") {
            raise(&mut axes, Axis::Control, 0.5);
            add_theme(&mut themes, Theme::Scholarly);
        }
        if boosted("wisdom") {
            raise(&mut axes, Axis::Support, 0.5);
        }
    }

    for field in [&c.traits, &c.benefits].into_iter().flatten() {
        for name in json_names(field) {
            apply_trait_keywords(&name, &mut axes, &mut themes);
        }
    }

    HeuristicProfile { axes, themes }
}

// ─── Scoring ─────────────────────────────────────────────────────────────

const THEME_BONUS_WEIGHT: f32 = 0.15;
/// Profile axis values below this are too weak to claim as a reason.
const REASON_AXIS_FLOOR: f32 = 0.55;

struct UserVector {
    axes: [f32; AXIS_COUNT],
    themes: Vec<Theme>,
    law_chaos: i32,
    good_evil: i32,
}

fn user_vector(answers: &[Answer]) -> UserVector {
    let mut u = UserVector {
        axes: [0.0; AXIS_COUNT],
        themes: Vec::new(),
        law_chaos: 0,
        good_evil: 0,
    };
    // Unknown question keys/values are ignored so stored answers from a
    // newer questionnaire never break an older engine.
    for effect in answers
        .iter()
        .filter_map(|a| option_effect(&a.question, &a.value))
    {
        for &(axis, weight) in effect.axes {
            u.axes[axis as usize] += weight;
        }
        for &theme in effect.themes {
            add_theme(&mut u.themes, theme);
        }
        if let Some((lc, ge)) = effect.alignment {
            u.law_chaos += i32::from(lc);
            u.good_evil += i32::from(ge);
        }
    }
    u
}

fn axis_phrase(axis: Axis) -> &'static str {
    match axis {
        Axis::Martial => "shines in physical combat, which matched your answers",
        Axis::Magic => "is steeped in magic, which you leaned toward",
        Axis::Durability => "holds the front line, right where you put your character",
        Axis::Finesse => "favors speed and precision over brute force, matching their style",
        Axis::Support => "keeps the party standing — the role you gravitated to",
        Axis::Control => "bends the battlefield to its will, fitting how they solve problems",
        Axis::Social => "talks its way through trouble, just like the character you described",
        Axis::Stealth => "works from the shadows, matching their sneaky streak",
        Axis::Simplicity => "is easy to pick up and play, fitting the pace you wanted",
        Axis::Solitude => "walks its own path — a natural fit for the loner you described",
        Axis::Camaraderie => "is at its best among companions, the way your character lives",
        Axis::Devotion => "lives to serve and protect others, true to the character you described",
    }
}

const ALL_AXES: [Axis; AXIS_COUNT] = [
    Axis::Martial,
    Axis::Magic,
    Axis::Durability,
    Axis::Finesse,
    Axis::Support,
    Axis::Control,
    Axis::Social,
    Axis::Stealth,
    Axis::Simplicity,
    Axis::Solitude,
    Axis::Camaraderie,
    Axis::Devotion,
];

fn theme_phrase(theme: Theme) -> &'static str {
    match theme {
        Theme::Nature => "the natural world",
        Theme::Divine => "gods and higher powers",
        Theme::Arcane => "arcane mysteries",
        Theme::Criminal => "the criminal underside",
        Theme::Noble => "courts and high society",
        Theme::Wilderness => "the untamed wilds",
        Theme::Scholarly => "lore and learning",
        Theme::Performer => "the performer's life",
        Theme::Maker => "craft and invention",
    }
}

struct Scored {
    uuid: String,
    key: String,
    name: String,
    score: f32,
    axes: [f32; AXIS_COUNT],
    themes: Vec<Theme>,
    mapped: bool,
    /// Profile resolved by exact stable key (vs name fallback/heuristic);
    /// preferred when same-named duplicates tie.
    exact_key: bool,
    ability_priority: Vec<AbilityScore>,
    reasons: Vec<String>,
}

fn score_candidate(raw: &RawCandidate, category: Category, user: &UserVector) -> Scored {
    let (axes, themes, mapped, exact_key, authored_priority): (
        [f32; AXIS_COUNT],
        Vec<Theme>,
        bool,
        bool,
        Vec<AbilityScore>,
    ) = match raw.profile(category) {
        Some(m) => (
            m.profile.axes,
            m.profile.themes.to_vec(),
            true,
            m.exact_key,
            m.profile.ability_priority.to_vec(),
        ),
        None => {
            let h = heuristic_profile(raw);
            (h.axes, h.themes, false, false, Vec::new())
        }
    };

    let weight_total: f32 = user.axes.iter().sum();
    let base = if weight_total > 0.0 {
        ALL_AXES
            .iter()
            .map(|&a| user.axes[a as usize] * axes[a as usize])
            .sum::<f32>()
            / weight_total
    } else {
        0.5
    };
    let overlap = themes.iter().filter(|t| user.themes.contains(t)).count() as f32;
    let theme_bonus =
        THEME_BONUS_WEIGHT * overlap / (user.themes.len().max(1) as f32);
    let score = (base + theme_bonus).min(1.0);

    let ability_priority = if authored_priority.is_empty() && category == Category::Class {
        heuristic_ability_priority(raw)
    } else {
        authored_priority
    };

    Scored {
        uuid: raw.uuid.clone().unwrap_or_default(),
        key: raw.key.clone(),
        name: raw.name.clone(),
        score,
        axes,
        themes,
        mapped,
        exact_key,
        ability_priority,
        reasons: Vec::new(),
    }
}

/// Fills in "why this fits" text for the final ranked cards. Runs across
/// the whole top-3 so the cards read differently: once a higher-ranked
/// card has claimed an axis or theme sentence, lower-ranked cards prefer
/// their strongest *unclaimed* axis instead of repeating the same lines
/// with only the name swapped.
fn build_reasons(scored: &mut [Scored], user: &UserVector) {
    let mut used_axes: Vec<Axis> = Vec::new();
    let mut used_themes: Vec<Theme> = Vec::new();

    for s in scored.iter_mut() {
        let mut contributions: Vec<(Axis, f32)> = ALL_AXES
            .iter()
            .map(|&axis| (axis, user.axes[axis as usize] * s.axes[axis as usize]))
            .filter(|&(axis, c)| c > 0.0 && s.axes[axis as usize] >= REASON_AXIS_FLOOR)
            .collect();
        contributions.sort_by(|a, b| b.1.total_cmp(&a.1));

        let fresh: Vec<Axis> = contributions
            .iter()
            .map(|&(axis, _)| axis)
            .filter(|axis| !used_axes.contains(axis))
            .take(2)
            .collect();
        // Nothing distinctive left: repeat this card's single strongest
        // axis rather than saying nothing.
        let chosen: Vec<Axis> = if fresh.is_empty() {
            contributions.first().map(|&(axis, _)| axis).into_iter().collect()
        } else {
            fresh
        };
        used_axes.extend(&chosen);

        let mut reasons: Vec<String> = chosen
            .iter()
            .map(|&axis| format!("{} {}.", s.name, axis_phrase(axis)))
            .collect();

        if let Some(theme) = s
            .themes
            .iter()
            .find(|t| user.themes.contains(t) && !used_themes.contains(t))
        {
            used_themes.push(*theme);
            reasons.push(format!(
                "Its connection to {} matches the world you're drawn to.",
                theme_phrase(*theme)
            ));
        }
        if !s.mapped {
            reasons.push("Estimated from its game mechanics — this one isn't in our hand-tuned guide yet.".to_string());
        }
        if reasons.is_empty() {
            reasons.push(format!("{} is a solid, beginner-friendly pick.", s.name));
        }
        s.reasons = reasons;
    }
}

fn score_category(candidates: &[Value], category: Category, user: &UserVector) -> Vec<Scored> {
    let mut scored: Vec<Scored> = candidates
        .iter()
        .filter_map(|v| serde_json::from_value::<RawCandidate>(v.clone()).ok())
        .filter(|raw| raw.is_base(category))
        .map(|raw| score_candidate(&raw, category, user))
        .collect();
    scored.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| b.exact_key.cmp(&a.exact_key))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.key.cmp(&b.key))
    });
    // The bundle carries same-named reprints across source documents
    // (two Acolytes, two Criminals, …) that tie on score and would fill
    // the list with twins — keep only the best-ranked copy of a name.
    let mut seen_names: Vec<String> = Vec::new();
    scored.retain(|s| {
        let name = s.name.to_lowercase();
        if seen_names.contains(&name) {
            false
        } else {
            seen_names.push(name);
            true
        }
    });
    scored.truncate(3);
    build_reasons(&mut scored, user);
    scored
}

// ─── Ability priorities & alignment ──────────────────────────────────────

const DEFAULT_PRIORITY: [AbilityScore; 6] = [
    AbilityScore::Strength,
    AbilityScore::Dexterity,
    AbilityScore::Constitution,
    AbilityScore::Wisdom,
    AbilityScore::Intelligence,
    AbilityScore::Charisma,
];

fn parse_ability(name: &str) -> Option<AbilityScore> {
    match name {
        "strength" => Some(AbilityScore::Strength),
        "dexterity" => Some(AbilityScore::Dexterity),
        "constitution" => Some(AbilityScore::Constitution),
        "intelligence" => Some(AbilityScore::Intelligence),
        "wisdom" => Some(AbilityScore::Wisdom),
        "charisma" => Some(AbilityScore::Charisma),
        _ => None,
    }
}

/// Derives a priority order for a class with no authored profile:
/// spellcasting ability first when present, then Con for staying power,
/// then the remaining abilities in a sensible fixed order.
fn heuristic_ability_priority(raw: &RawCandidate) -> Vec<AbilityScore> {
    let leads: Vec<AbilityScore> = raw
        .spellcasting_ability
        .as_deref()
        .and_then(parse_ability)
        .map(|casting| vec![casting, AbilityScore::Constitution])
        .unwrap_or_else(|| {
            if raw.hit_dice.unwrap_or(8) >= 10 {
                vec![AbilityScore::Strength, AbilityScore::Constitution]
            } else {
                vec![AbilityScore::Dexterity, AbilityScore::Constitution]
            }
        });
    let mut priority = leads.clone();
    priority.extend(DEFAULT_PRIORITY.iter().filter(|a| !leads.contains(a)));
    priority
}

/// Standard array 15/14/13/12/10/8 dealt out best-first by priority.
fn standard_array(priority: &[AbilityScore]) -> AbilityScores {
    const VALUES: [i32; 6] = [15, 14, 13, 12, 10, 8];
    let mut scores = AbilityScores::default();
    for (ability, value) in priority.iter().zip(VALUES) {
        match ability {
            AbilityScore::Strength => scores.strength = value,
            AbilityScore::Dexterity => scores.dexterity = value,
            AbilityScore::Constitution => scores.constitution = value,
            AbilityScore::Intelligence => scores.intelligence = value,
            AbilityScore::Wisdom => scores.wisdom = value,
            AbilityScore::Charisma => scores.charisma = value,
        }
    }
    scores
}

/// Maps summed morality answers to one of the six non-evil alignments.
/// Evil leanings are folded to neutral: the quiz never steers a brand-new
/// player toward an evil character.
fn resolve_alignment(law_chaos: i32, good_evil: i32) -> (&'static str, &'static str) {
    let lc = law_chaos.signum();
    let ge = good_evil.signum().max(0);
    match (lc, ge) {
        (-1, 1) => ("lawful_good", "You value both doing right and doing it by the book."),
        (0, 1) => ("neutral_good", "You put doing right above rules or rebellion."),
        (1, 1) => ("chaotic_good", "You want to do right — on your own terms."),
        (-1, 0) => ("lawful_neutral", "You keep your word and keep the balance."),
        (1, 0) => ("chaotic_neutral", "You answer to yourself first."),
        _ => ("true_neutral", "You take the world as it comes, without an agenda."),
    }
}

// ─── Entry point ─────────────────────────────────────────────────────────

/// Ranks the client's candidate records against the player's answers.
/// Pure and deterministic: identical inputs always produce identical
/// output. Candidates that fail to parse are skipped, and unknown answer
/// keys/values are ignored.
pub fn recommend(answers: &[Answer], candidates: &CandidateSets) -> RecommendationSet {
    let user = user_vector(answers);

    let classes = score_category(&candidates.classes, Category::Class, &user);
    let species = score_category(&candidates.species, Category::Species, &user);
    let backgrounds = score_category(&candidates.backgrounds, Category::Background, &user);

    let ability_priorities = classes
        .first()
        .filter(|s| !s.ability_priority.is_empty())
        .map_or_else(|| DEFAULT_PRIORITY.to_vec(), |s| s.ability_priority.clone());
    let standard_array_suggestion = standard_array(&ability_priorities);
    let (alignment_suggestion, alignment_reason) =
        resolve_alignment(user.law_chaos, user.good_evil);

    let take = |scored: Vec<Scored>| -> Vec<Recommendation> {
        scored
            .into_iter()
            .map(|s| Recommendation {
                uuid: s.uuid,
                key: s.key,
                name: s.name,
                score: s.score,
                reasons: s.reasons,
                mapped: s.mapped,
            })
            .collect()
    };

    RecommendationSet {
        version: GUIDANCE_VERSION,
        classes: take(classes),
        species: take(species),
        backgrounds: take(backgrounds),
        ability_priorities,
        standard_array_suggestion,
        alignment_suggestion: alignment_suggestion.to_string(),
        alignment_reason: alignment_reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn all_answers() -> Vec<Answer> {
        vec![
            Answer { question: "hero_fantasy".into(), value: "warrior".into() },
            Answer { question: "combat_approach".into(), value: "charge".into() },
            Answer { question: "party_role".into(), value: "protector".into() },
            Answer { question: "problem_solving".into(), value: "force".into() },
            Answer { question: "social_energy".into(), value: "hold_court".into() },
            Answer { question: "belonging".into(), value: "found_family".into() },
            Answer { question: "service".into(), value: "always_helps".into() },
            Answer { question: "complexity".into(), value: "simple".into() },
            Answer { question: "theme".into(), value: "nature".into() },
            Answer { question: "morality".into(), value: "free_spirit".into() },
            Answer { question: "durability_style".into(), value: "wall".into() },
            Answer { question: "magic_affinity".into(), value: "none".into() },
        ]
    }

    fn bundle() -> Value {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../content/srd-bundle.json");
        let raw = std::fs::read_to_string(path).expect("srd bundle readable");
        serde_json::from_str(&raw).expect("srd bundle parses")
    }

    fn bundle_candidates() -> CandidateSets {
        let bundle = bundle();
        let arr = |key: &str| bundle[key].as_array().expect("bundle array").clone();
        CandidateSets {
            classes: arr("classes"),
            species: arr("species"),
            backgrounds: arr("backgrounds"),
        }
    }

    #[test]
    fn questionnaire_is_well_formed_and_round_trips() {
        let q = guidance_questionnaire();
        assert!((6..=12).contains(&q.questions.len()));
        assert!(!q.intro.is_empty());
        for question in &q.questions {
            assert!(question.options.len() >= 2, "{} too few options", question.key);
        }
        let json = serde_json::to_string(&q).unwrap();
        let back: Questionnaire = serde_json::from_str(&json).unwrap();
        assert_eq!(q, back);
    }

    #[test]
    fn every_option_value_has_weights() {
        for question in guidance_questionnaire().questions {
            for option in &question.options {
                assert!(
                    option_effect(&question.key, &option.value).is_some(),
                    "no weights for {}:{}",
                    question.key,
                    option.value
                );
            }
        }
    }

    #[test]
    fn recommend_is_deterministic() {
        let candidates = bundle_candidates();
        let answers = all_answers();
        let a = serde_json::to_string(&recommend(&answers, &candidates)).unwrap();
        let b = serde_json::to_string(&recommend(&answers, &candidates)).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn warrior_answers_rank_martial_classes_first() {
        let result = recommend(&all_answers(), &bundle_candidates());
        assert_eq!(result.classes.len(), 3);
        let top = &result.classes[0];
        assert!(
            ["Barbarian", "Fighter"].contains(&top.name.as_str()),
            "expected a martial class on top, got {}",
            top.name
        );
        assert_eq!(result.alignment_suggestion, "chaotic_good");
        assert_eq!(result.ability_priorities[0], AbilityScore::Strength);
        assert_eq!(result.standard_array_suggestion.strength, 15);
    }

    /// Two-way drift guard between the profile tables and the bundle:
    /// every base class/species in the bundle must resolve a profile, and
    /// every profile key must still exist in the bundle.
    #[test]
    fn srd_bundle_base_records_are_all_mapped() {
        let bundle = bundle();
        let records = |key: &str| {
            bundle[key]
                .as_array()
                .expect("bundle array")
                .iter()
                .map(|r| {
                    (
                        r["key"].as_str().expect("key").to_string(),
                        r["name"].as_str().expect("name").to_string(),
                    )
                })
                .collect::<Vec<_>>()
        };

        for (key, name) in records("classes")
            .iter()
            .filter(|(k, _)| bundle["classes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["key"] == k.as_str() && r["subclass_of"].is_null()))
        {
            assert!(
                class_profile(key, name).is_some(),
                "base class {key} ({name}) has no profile"
            );
        }
        for (key, name) in records("species").iter().filter(|(k, _)| {
            bundle["species"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["key"] == k.as_str() && r["is_subspecies"] == false)
        }) {
            assert!(
                species_profile(key, name).is_some(),
                "base species {key} ({name}) has no profile"
            );
        }

        let mapped_backgrounds = records("backgrounds")
            .iter()
            .filter(|(key, name)| background_profile(key, name).is_some())
            .count();
        assert!(
            mapped_backgrounds >= 20,
            "only {mapped_backgrounds} backgrounds mapped"
        );

        let bundle_keys = |key: &str| {
            bundle[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r["key"].as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        };
        let (classes, species, backgrounds) =
            (bundle_keys("classes"), bundle_keys("species"), bundle_keys("backgrounds"));
        for (key, _, _) in crate::guidance_profiles::CLASS_PROFILES {
            assert!(classes.contains(&(*key).to_string()), "stale class profile key {key}");
        }
        for (key, _, _) in crate::guidance_profiles::SPECIES_PROFILES {
            assert!(species.contains(&(*key).to_string()), "stale species profile key {key}");
        }
        for (key, _, _) in crate::guidance_profiles::BACKGROUND_PROFILES {
            assert!(
                backgrounds.contains(&(*key).to_string()),
                "stale background profile key {key}"
            );
        }
    }

    /// Personality answers must steer results on their own: a devoted,
    /// group-oriented character lands on service classes; a loner lands
    /// on solitary classes and backgrounds — independent of combat
    /// answers.
    #[test]
    fn personality_answers_steer_recommendations() {
        let devoted = vec![
            Answer { question: "hero_fantasy".into(), value: "guide".into() },
            Answer { question: "social_energy".into(), value: "one_friend".into() },
            Answer { question: "belonging".into(), value: "cause".into() },
            Answer { question: "service".into(), value: "always_helps".into() },
            Answer { question: "morality".into(), value: "kindhearted".into() },
        ];
        let result = recommend(&devoted, &bundle_candidates());
        assert_eq!(
            result.classes[0].name, "Cleric",
            "devoted answers should surface the Cleric first, got {:?}",
            result.classes.iter().map(|r| &r.name).collect::<Vec<_>>()
        );

        let loner = vec![
            Answer { question: "social_energy".into(), value: "reluctant".into() },
            Answer { question: "belonging".into(), value: "open_road".into() },
            Answer { question: "service".into(), value: "self_reliance".into() },
        ];
        let result = recommend(&loner, &bundle_candidates());
        assert_eq!(
            result.classes[0].name, "Ranger",
            "loner answers should surface the Ranger first, got {:?}",
            result.classes.iter().map(|r| &r.name).collect::<Vec<_>>()
        );
        assert!(
            ["Exile", "Hermit", "Outlander"]
                .contains(&result.backgrounds[0].name.as_str()),
            "loner answers should surface a solitary background, got {:?}",
            result.backgrounds.iter().map(|r| &r.name).collect::<Vec<_>>()
        );
    }

    /// The bundle ships same-named background reprints (srd-2024 and
    /// a5e-ag Acolyte/Criminal/Sage/Soldier, two Scoundrels) whose
    /// profiles tie exactly — only one copy of a name may surface, and
    /// on a tie it must be the exact-key (SRD) copy, not the reprint.
    #[test]
    fn duplicate_named_candidates_collapse_preferring_exact_key() {
        let result = recommend(&all_answers(), &bundle_candidates());
        for recs in [&result.classes, &result.species, &result.backgrounds] {
            let names: Vec<String> =
                recs.iter().map(|r| r.name.to_lowercase()).collect();
            let mut unique = names.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(names.len(), unique.len(), "duplicate names in top-3");
        }

        let twins = CandidateSets {
            classes: vec![],
            species: vec![],
            backgrounds: vec![
                json!({"uuid": "a", "key": "a5e-ag_acolyte", "name": "Acolyte"}),
                json!({"uuid": "b", "key": "srd-2024_acolyte", "name": "Acolyte"}),
            ],
        };
        let result = recommend(&all_answers(), &twins);
        assert_eq!(result.backgrounds.len(), 1);
        assert_eq!(result.backgrounds[0].key, "srd-2024_acolyte");
    }

    /// Ranked cards in one category must not repeat the same reason
    /// lines with only the name swapped: lower cards prefer their
    /// strongest axis not already claimed by a higher card, and each
    /// theme sentence appears at most once.
    #[test]
    fn top3_reasons_differ_across_cards() {
        let result = recommend(&all_answers(), &bundle_candidates());
        for recs in [&result.classes, &result.species, &result.backgrounds] {
            let stripped: Vec<Vec<String>> = recs
                .iter()
                .map(|r| {
                    r.reasons
                        .iter()
                        .map(|line| line.replace(&r.name, "<name>"))
                        .collect()
                })
                .collect();
            for (i, a) in stripped.iter().enumerate() {
                for b in stripped.iter().skip(i + 1) {
                    assert_ne!(a, b, "identical reason lists in a category");
                }
            }
            let theme_lines: Vec<&String> = recs
                .iter()
                .flat_map(|r| &r.reasons)
                .filter(|line| line.contains("matches the world"))
                .collect();
            let mut unique = theme_lines.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(theme_lines.len(), unique.len(), "repeated theme sentence");
        }
    }

    #[test]
    fn unmapped_candidate_gets_heuristic_score_and_reason() {
        let candidates = CandidateSets {
            classes: vec![json!({"key": "hb_gladiator", "name": "Gladiator", "hit_dice": 12})],
            species: vec![],
            backgrounds: vec![],
        };
        let result = recommend(&all_answers(), &candidates);
        assert_eq!(result.classes.len(), 1);
        let rec = &result.classes[0];
        assert!(!rec.mapped);
        assert!(rec.score > 0.0);
        assert!(!rec.reasons.is_empty());
        assert!(rec.reasons.iter().any(|r| r.contains("Estimated")));
        // d12 martial heuristic should lead with Strength.
        assert_eq!(result.ability_priorities[0], AbilityScore::Strength);
    }

    #[test]
    fn all_recommendations_have_reasons() {
        let result = recommend(&all_answers(), &bundle_candidates());
        for rec in result
            .classes
            .iter()
            .chain(&result.species)
            .chain(&result.backgrounds)
        {
            assert!(!rec.reasons.is_empty(), "{} has no reasons", rec.name);
        }
        // Empty answers must still produce reasons (the generic fallback).
        let empty = recommend(&[], &bundle_candidates());
        for rec in empty.classes.iter().chain(&empty.species).chain(&empty.backgrounds) {
            assert!(!rec.reasons.is_empty(), "{} has no reasons on empty answers", rec.name);
        }
    }

    #[test]
    fn standard_array_uses_each_value_once() {
        let result = recommend(&all_answers(), &bundle_candidates());
        let s = result.standard_array_suggestion;
        let mut values = [
            s.strength,
            s.dexterity,
            s.constitution,
            s.intelligence,
            s.wisdom,
            s.charisma,
        ];
        values.sort_unstable();
        assert_eq!(values, [8, 10, 12, 13, 14, 15]);
    }

    #[test]
    fn every_morality_option_maps_to_a_valid_nonevil_alignment() {
        let q = guidance_questionnaire();
        let morality = q
            .questions
            .iter()
            .find(|question| question.key == "morality")
            .expect("morality question");
        for option in &morality.options {
            let answers = vec![Answer {
                question: "morality".into(),
                value: option.value.clone(),
            }];
            let result = recommend(&answers, &CandidateSets::default());
            assert!(
                [
                    "lawful_good",
                    "neutral_good",
                    "chaotic_good",
                    "lawful_neutral",
                    "true_neutral",
                    "chaotic_neutral",
                ]
                .contains(&result.alignment_suggestion.as_str()),
                "{} maps to {}",
                option.value,
                result.alignment_suggestion
            );
            assert!(!result.alignment_reason.is_empty());
        }
    }

    /// The web client passes list summaries, not full records: summary
    /// shapes must stay sufficient for parsing and ranking.
    #[test]
    fn summaries_alone_are_sufficient() {
        let bundle = bundle();
        let summarize = |record: &Value, keep: &[&str]| -> Value {
            let mut out = serde_json::Map::new();
            for k in keep {
                if let Some(v) = record.get(*k) {
                    out.insert((*k).to_string(), v.clone());
                }
            }
            Value::Object(out)
        };
        // Field lists mirror ClassSummary / SpeciesSummary / BackgroundSummary.
        let candidates = CandidateSets {
            classes: bundle["classes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| summarize(r, &["uuid", "key", "slug", "name", "subclass_of", "hit_dice", "caster_type"]))
                .collect(),
            species: bundle["species"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| summarize(r, &["uuid", "key", "slug", "name", "is_subspecies", "size", "speed", "subspecies_of"]))
                .collect(),
            backgrounds: bundle["backgrounds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| summarize(r, &["uuid", "key", "slug", "name"]))
                .collect(),
        };
        let result = recommend(&all_answers(), &candidates);
        assert_eq!(result.classes.len(), 3);
        assert_eq!(result.species.len(), 3);
        assert_eq!(result.backgrounds.len(), 3);
        assert!(result.classes.iter().all(|r| !r.uuid.is_empty()));
    }
}
