//! Deserialize models for the Open5e **v1** API — used only to recover
//! structured sheet-math data (class proficiencies, species ASI/speed)
//! that the v2 API dropped to prose.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct V1Class {
    pub name: String,
    #[serde(default)]
    pub prof_armor: String,
    #[serde(default)]
    pub prof_weapons: String,
    #[serde(default)]
    pub prof_tools: String,
    #[serde(default)]
    pub prof_skills: String,
    #[serde(default)]
    pub equipment: String,
    #[serde(default)]
    pub spellcasting_ability: String,
    #[serde(default)]
    pub subtypes_name: String,
}

#[derive(Debug, Deserialize)]
pub struct V1Asi {
    #[serde(default)]
    pub attributes: Vec<String>,
    #[serde(default)]
    pub value: i32,
}

#[derive(Debug, Default, Deserialize)]
pub struct V1Speed {
    #[serde(default)]
    pub walk: i32,
}

#[derive(Debug, Deserialize)]
pub struct V1Race {
    pub name: String,
    #[serde(default)]
    pub asi: Vec<V1Asi>,
    #[serde(default)]
    pub asi_desc: String,
    #[serde(default)]
    pub size_raw: String,
    #[serde(default)]
    pub speed: V1Speed,
    #[serde(default)]
    pub languages: String,
    #[serde(default)]
    pub vision: String,
    #[serde(default)]
    pub subraces: Vec<V1Subrace>,
}

#[derive(Debug, Deserialize)]
pub struct V1Subrace {
    pub name: String,
    #[serde(default)]
    pub asi: Vec<V1Asi>,
    #[serde(default)]
    pub asi_desc: String,
}
