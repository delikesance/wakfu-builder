use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Equipment {
    #[serde(rename = "id_equipment")]
    pub id: i32,
    #[serde(rename = "name_equipment")]
    pub name: String,
    pub level: i32,
    #[serde(rename = "id_rarity")]
    pub id_rarity: i32,
    #[serde(rename = "id_equipment_type")]
    pub id_type: i32,
    #[serde(rename = "name_equipment_type")]
    pub equipment_type: String,
    pub effects: Vec<Effect>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Effect {
    pub name_effect: String,
    pub values: Vec<EffectValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EffectValue {
    pub damage: f32, 
    pub statistics: Option<StatInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatInfo {
    pub id_stats: i32,
    pub name_stats: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Spell {
    pub id_spell: i32,
    pub name_spell: String,
}
