use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintBlock {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
    pub h: f32,
    pub d: f32,
    pub color: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintDef {
    pub name: Option<String>,
    pub blocks: Vec<BlueprintBlock>,
}

#[derive(Debug, Clone)]
pub struct ResolvedBlueprint {
    pub name: String,
    pub blocks: Vec<ResolvedBlock>,
}

#[derive(Debug, Clone)]
pub struct ResolvedBlock {
    pub x: f32, pub y: f32, pub z: f32,
    pub w: f32, pub h: f32, pub d: f32,
    pub color: [f32; 3],
}

pub fn resolve_blueprint(def: &BlueprintDef, default_color: [f32; 3]) -> ResolvedBlueprint {
    let blocks = def.blocks.iter().map(|b| ResolvedBlock {
        x: b.x, y: b.y, z: b.z,
        w: b.w, h: b.h, d: b.d,
        color: b.color.unwrap_or(default_color),
    }).collect();
    ResolvedBlueprint {
        name: def.name.clone().unwrap_or_default(),
        blocks,
    }
}
