use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tower {
    pub owner: String,   // "ALLY" or "ENEMY"
    pub x: f32,
    pub y: f32,
    pub hp_frac: f32,    // 0.0–1.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Unit {
    pub owner: String,   // "ALLY" or "ENEMY"
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LegalMasks {
    pub cards: Vec<bool>,       // len = 8
    pub tiles_flat: Vec<bool>,  // len = place_W * place_H
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CRState {
    pub t_ms: u64,
    pub ally_elixir: f32,
    pub time_left: f32,
    pub overtime: bool,

    pub ally_towers: Vec<Tower>,
    pub enemy_towers: Vec<Tower>,
    pub ally_units: Vec<Unit>,
    pub enemy_units: Vec<Unit>,

    pub legal: LegalMasks,

    pub win: bool,
    pub lose: bool,

    pub enemy_tower_hp_drop: f32,
    pub ally_tower_hp_drop: f32,
}