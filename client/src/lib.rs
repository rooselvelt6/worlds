use wasm_bindgen::prelude::*;
use worlds_shared::{WorldGenerator, WorldParams};

#[derive(Clone)]
pub struct WorldConfig {
    pub seed: u32,
    pub scale: f64,
    pub octaves: u8,
    pub amplitude: f64,
    pub water_level: f64,
    pub zone: String,
    pub noise_type: String,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            scale: 0.025,
            octaves: 5,
            amplitude: 20.0,
            water_level: 6.0,
            zone: "forest".to_string(),
            noise_type: "fbm".to_string(),
        }
    }
}

#[wasm_bindgen]
pub struct WorldEngine {
    seed: u32,
    scale: f64,
    octaves: u8,
    amplitude: f64,
    water_level: f64,
    zone: String,
    generator: WorldGenerator,
}

#[wasm_bindgen]
impl WorldEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Self {
        console_error_panic_hook::set_once();
        
        Self {
            seed,
            scale: 0.025,
            octaves: 5,
            amplitude: 20.0,
            water_level: 6.0,
            zone: "forest".to_string(),
            generator: WorldGenerator::new(seed),
        }
    }
    
    #[wasm_bindgen]
    pub fn set_seed(&mut self, seed: u32) {
        self.seed = seed;
        self.generator = WorldGenerator::new(seed);
    }
    
    #[wasm_bindgen]
    pub fn set_zone(&mut self, zone: &str) {
        self.zone = zone.to_string();
    }
    
    #[wasm_bindgen]
    pub fn set_params(&mut self, scale: f64, octaves: u8, amplitude: f64, water: f64) {
        self.scale = scale;
        self.octaves = octaves;
        self.amplitude = amplitude;
        self.water_level = water;
    }
    
    #[wasm_bindgen]
    pub fn get_height(&self, x: f64, z: f64) -> f64 {
        let params = WorldParams {
            seed: self.seed,
            scale: self.scale,
            octaves: self.octaves,
            amplitude: self.amplitude,
            water_level: self.water_level,
            cave_density: 0.4,
            cave_size: 0.1,
            mountain_scale: 1.5,
            biome_weight: 0.3,
        };
        
        let base_height = self.generator.get_height(x, z, &params);
        
        match self.zone.as_str() {
            "ocean" => base_height + self.water_level,
            "volcanic" => base_height * 1.5 + 2.0,
            "crystal" => base_height * 0.5,
            "cave" => 3.0 + (x * 0.5).sin() * 2.0,
            "lava" => base_height * 2.0 + 5.0,
            "tundra" => base_height * 0.7,
            "desert" => base_height * 1.2 + 1.0,
            "jungle" => base_height * 1.3,
            _ => base_height,
        }
    }
    
    #[wasm_bindgen]
    pub fn get_biome(&self, x: f64, z: f64) -> String {
        if self.zone != "forest" {
            return self.zone.clone();
        }
        
        let params = WorldParams::default();
        String::from(self.generator.get_biome(x, z, &params))
    }
    
    #[wasm_bindgen]
    pub fn get_height_map(&self, center_x: f64, center_z: f64, size: u32, step: f64) -> Vec<f32> {
        let mut heights = Vec::with_capacity((size * size) as usize);
        
        let half_size = size as f64 / 2.0;
        
        for z in 0..size {
            for x in 0..size {
                let wx = center_x + (x as f64 - half_size) * step;
                let wz = center_z + (z as f64 - half_size) * step;
                let h = self.get_height(wx, wz) as f32;
                heights.push(h);
            }
        }
        
        heights
    }
    
    #[wasm_bindgen]
    pub fn get_chunk_data(&self, chunk_x: i32, chunk_z: i32, chunk_size: usize) -> String {
        let mut data = String::new();
        let world_x = chunk_x * chunk_size as i32;
        let world_z = chunk_z * chunk_size as i32;
        
        let params = WorldParams {
            seed: self.seed,
            scale: self.scale,
            octaves: self.octaves,
            amplitude: self.amplitude,
            water_level: self.water_level,
            cave_density: 0.4,
            cave_size: 0.1,
            mountain_scale: 1.5,
            biome_weight: 0.3,
        };
        
        data.push_str(&format!("chunk:{},{}\n", chunk_x, chunk_z));
        
        for lz in 0..chunk_size {
            for lx in 0..chunk_size {
                let wx = (world_x + lx as i32) as f64;
                let wz = (world_z + lz as i32) as f64;
                let h = self.generator.get_height(wx, wz, &params);
                let b = self.generator.get_biome(wx, wz, &params);
                data.push_str(&format!("({},{}):{}:{}\n", lx, lz, h as i32, b));
            }
        }
        
        data
    }
    
    #[wasm_bindgen]
    pub fn get_info(&self) -> String {
        format!(
            "WORLDS Engine (Pure Rust + WASM)\n\
             Seed: {}\n\
             Zone: {}\n\
             Scale: {}\n\
             Octaves: {}\n\
             Amplitude: {}\n\
             Water: {}",
            self.seed,
            self.zone,
            self.scale,
            self.octaves,
            self.amplitude,
            self.water_level
        )
    }
    
    #[wasm_bindgen]
    pub fn get_seed(&self) -> u32 {
        self.seed
    }
    
    #[wasm_bindgen]
    pub fn get_zone(&self) -> String {
        self.zone.clone()
    }
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!(
        "Hello, {}!\n\
         Running on 100% Pure Rust + WASM!\n\
         🦀 Rust Power Active 🦀",
        name
    )
}

#[wasm_bindgen]
pub fn generate_preview(seed: u32) -> String {
    let generator = WorldGenerator::new(seed);
    let params = WorldParams::default();
    
    let mut output = String::from("Rust Noise Preview:\n");
    
    for z in 0..8 {
        for x in 0..8 {
            let h = generator.get_height(x as f64 * 8.0, z as f64 * 8.0, &params);
            output.push_str(&format!("{:5.1} ", h));
        }
        output.push('\n');
    }
    
    output
}

#[wasm_bindgen]
pub fn test_noise(x: f64, z: f64, seed: u32) -> f64 {
    let generator = WorldGenerator::new(seed);
    let params = WorldParams::default();
    generator.get_height(x, z, &params)
}