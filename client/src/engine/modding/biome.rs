use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeDef {
    pub display_name: Option<String>,
    pub color: Option<[f32; 3]>,
    pub rock_color: Option<[f32; 3]>,
    pub gradient_stops: Option<Vec<[f32; 3]>>,
    pub surface_block: Option<String>,
    pub height_multiplier: Option<f64>,
    pub height_offset: Option<f64>,
    pub vegetation_density: Option<f64>,
    pub vegetation_types: Option<Vec<VegTypeDef>>,
    pub struct_density: Option<f64>,
    pub struct_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VegTypeDef {
    pub r#type: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorOverride {
    pub block_name: String,
    pub color: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModPalette {
    pub name: String,
    pub colors: Vec<ColorOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomBiomeDef {
    pub id: String,
    pub display_name: String,
    pub color: [f32; 3],
    pub rock_color: [f32; 3],
    pub gradient_stops: Vec<[f32; 3]>,
    pub surface_block: String,
    pub height_multiplier: f64,
    pub height_offset: f64,
    pub vegetation_density: f64,
    pub vegetation_types: Vec<VegTypeDef>,
    pub struct_density: f64,
    pub struct_types: Vec<String>,
}

pub fn block_name_to_id(name: &str) -> u8 {
    match name {
        "air" => 0, "grass" => 1, "dirt" => 2, "stone" => 3,
        "sand" => 4, "snow" => 5, "coal_ore" => 6, "iron_ore" => 7,
        "gold_ore" => 8, "diamond_ore" => 9, "gravel" => 10, "clay" => 11,
        "water" => 12, "lava" => 13,
        "packed_ice" => 20, "obsidian" => 21, "moss" => 22,
        "glow_shroom" => 23, "magma_block" => 24, "soul_sand" => 25, "basalt" => 26,
        _ => 1,
    }
}

pub fn veg_type_name_to_id(name: &str) -> Option<u8> {
    match name {
        "tree" => Some(0), "bush" => Some(1), "rock" => Some(2),
        "cactus" => Some(3), "mushroom" => Some(4), "crystal" => Some(5),
        "dead_tree" => Some(6), "flower" => Some(7), "coral" => Some(8),
        "kelp" => Some(9), "seaweed" => Some(10), "anemone" => Some(11),
        "sponge" => Some(12),
        _ => None,
    }
}
