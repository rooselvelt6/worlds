use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use worlds_shared::{WorldGenerator, WorldParams, ChunkData, BlockType};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkMesh {
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
    pub colors: Vec<f32>,
    pub indices: Vec<u32>,
}

#[wasm_bindgen]
pub struct VoxelEngine {
    seed: u32,
    scale: f64,
    octaves: u8,
    amplitude: f64,
    water_level: f64,
    cave_density: f64,
    position: Position,
    yaw: f64,
    pitch: f64,
    chunks: HashMap<(i32, i32, i32), ChunkMesh>,
    render_distance: i32,
}

#[wasm_bindgen]
impl VoxelEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Self {
        console_error_panic_hook::set_once();
        
        VoxelEngine {
            seed,
            scale: 0.02,
            octaves: 6,
            amplitude: 20.0,
            water_level: 8.0,
            cave_density: 0.4,
            position: Position { x: 0.0, y: 30.0, z: 0.0 },
            yaw: 0.0,
            pitch: -0.3,
            chunks: HashMap::new(),
            render_distance: 4,
        }
    }
    
    pub fn set_seed(&mut self, seed: u32) {
        self.seed = seed;
    }
    
    pub fn get_seed(&self) -> u32 {
        self.seed
    }
    
    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }
    
    pub fn get_scale(&self) -> f64 {
        self.scale
    }
    
    pub fn set_octaves(&mut self, octaves: u8) {
        self.octaves = octaves;
    }
    
    pub fn get_octaves(&self) -> u8 {
        self.octaves
    }
    
    pub fn set_amplitude(&mut self, amplitude: f64) {
        self.amplitude = amplitude;
    }
    
    pub fn get_amplitude(&self) -> f64 {
        self.amplitude
    }
    
    pub fn set_water_level(&mut self, level: f64) {
        self.water_level = level;
    }
    
    pub fn get_water_level(&self) -> f64 {
        self.water_level
    }
    
    pub fn set_cave_density(&mut self, density: f64) {
        self.cave_density = density;
    }
    
    pub fn get_cave_density(&self) -> f64 {
        self.cave_density
    }
    
    pub fn set_render_distance(&mut self, distance: i32) {
        self.render_distance = distance;
    }
    
    pub fn get_render_distance(&self) -> i32 {
        self.render_distance
    }
    
    pub fn get_position(&self) -> String {
        format!("{:.1}, {:.1}, {:.1}", self.position.x, self.position.y, self.position.z)
    }
    
    pub fn get_chunk_coords(&self) -> String {
        let cx = (self.position.x / 16.0).floor() as i32;
        let cy = (self.position.y / 16.0).floor() as i32;
        let cz = (self.position.z / 16.0).floor() as i32;
        format!("{}, {}, {}", cx, cy, cz)
    }
    
    pub fn move_forward(&mut self, amount: f64) {
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();
        self.position.x -= sin_yaw * amount;
        self.position.z -= cos_yaw * amount;
    }
    
    pub fn move_back(&mut self, amount: f64) {
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();
        self.position.x += sin_yaw * amount;
        self.position.z += cos_yaw * amount;
    }
    
    pub fn move_left(&mut self, amount: f64) {
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();
        self.position.x -= cos_yaw * amount;
        self.position.z += sin_yaw * amount;
    }
    
    pub fn move_right(&mut self, amount: f64) {
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();
        self.position.x += cos_yaw * amount;
        self.position.z -= sin_yaw * amount;
    }
    
    pub fn move_up(&mut self, amount: f64) {
        self.position.y += amount;
    }
    
    pub fn move_down(&mut self, amount: f64) {
        self.position.y = (self.position.y - amount).max(1.0);
    }
    
    pub fn turn_left(&mut self, amount: f64) {
        self.yaw -= amount;
    }
    
    pub fn turn_right(&mut self, amount: f64) {
        self.yaw += amount;
    }
    
    pub fn look_up(&mut self, amount: f64) {
        self.pitch = (self.pitch - amount).max(-1.5).min(1.5);
    }
    
    pub fn look_down(&mut self, amount: f64) {
        self.pitch = (self.pitch + amount).max(-1.5).min(1.5);
    }
    
    pub fn get_terrain_height(&self, x: f64, z: f64) -> f64 {
        use worlds_shared::{WorldGenerator, WorldParams};
        
        let generator = WorldGenerator::new(self.seed);
        let params = WorldParams {
            seed: self.seed,
            scale: self.scale,
            octaves: self.octaves,
            amplitude: self.amplitude,
            water_level: self.water_level,
            cave_density: self.cave_density,
            cave_size: 0.1,
            mountain_scale: 1.5,
            biome_weight: 0.3,
        };
        
        generator.get_height(x, z, &params)
    }
    
    pub fn get_biome(&self, x: f64, z: f64) -> String {
        use worlds_shared::{WorldGenerator, WorldParams};
        
        let generator = WorldGenerator::new(self.seed);
        let params = WorldParams {
            seed: self.seed,
            scale: self.scale,
            octaves: self.octaves,
            amplitude: self.amplitude,
            water_level: self.water_level,
            cave_density: self.cave_density,
            cave_size: 0.1,
            mountain_scale: 1.5,
            biome_weight: 0.3,
        };
        
        String::from(generator.get_biome(x, z, &params))
    }
    
    pub fn generate_chunk_mesh(&self, cx: i32, cy: i32, cz: i32) -> JsValue {
        let generator = WorldGenerator::new(self.seed);
        let params = WorldParams {
            seed: self.seed,
            scale: self.scale,
            octaves: self.octaves,
            amplitude: self.amplitude,
            water_level: self.water_level,
            cave_density: self.cave_density,
            cave_size: 0.1,
            mountain_scale: 1.5,
            biome_weight: 0.3,
        };
        
        let chunk = generator.generate_chunk_with_params(cx, cy, cz, &params);
        let mesh = self.chunk_to_mesh(&chunk, cx, cy, cz);
        
        serde_wasm_bindgen::to_value(&mesh).unwrap_or(JsValue::NULL)
    }
    
    fn chunk_to_mesh(&self, chunk: &ChunkData, cx: i32, cy: i32, cz: i32) -> ChunkMesh {
        let mut vertices: Vec<f32> = Vec::new();
        let mut normals: Vec<f32> = Vec::new();
        let mut colors: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        
        let offset_x = cx as i32 * 16;
        let offset_y = cy as i32 * 16;
        let offset_z = cz as i32 * 16;
        
        let mut index_count: u32 = 0;
        
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let block = chunk.get_block(x, y, z);
                    if block == BlockType::Air {
                        continue;
                    }
                    
                    let (r, g, b) = self.block_color(&block);
                    
                    let faces = self.get_visible_faces(chunk, x, y, z);
                    
                    for face in faces {
                        let (vs, ns) = self.face_vertices(x, y, z, face, offset_x, offset_y, offset_z);
                        
                        for i in 0..vs.len() {
                            vertices.push(vs[i]);
                        }
                        for i in 0..ns.len() {
                            normals.push(ns[i]);
                        }
                        for _ in 0..4 {
                            colors.push(r);
                            colors.push(g);
                            colors.push(b);
                        }
                        
                        indices.push(index_count);
                        indices.push(index_count + 1);
                        indices.push(index_count + 2);
                        indices.push(index_count);
                        indices.push(index_count + 2);
                        indices.push(index_count + 3);
                        index_count += 4;
                    }
                }
            }
        }
        
        ChunkMesh {
            vertices,
            normals,
            colors,
            indices,
        }
    }
    
    fn block_color(&self, block: &BlockType) -> (f32, f32, f32) {
        use worlds_shared::BlockType;
        
        match block {
            BlockType::Air => (0.0, 0.0, 0.0),
            BlockType::Grass => (0.2, 0.6, 0.1),
            BlockType::Dirt => (0.4, 0.3, 0.15),
            BlockType::Stone => (0.5, 0.5, 0.5),
            BlockType::Sand => (0.9, 0.85, 0.6),
            BlockType::Water => (0.1, 0.3, 0.8),
            BlockType::Snow => (0.95, 0.95, 1.0),
            BlockType::Ice => (0.7, 0.9, 1.0),
            BlockType::Wood => (0.4, 0.25, 0.1),
            BlockType::Leaves => (0.1, 0.4, 0.1),
            BlockType::Cobblestone => (0.4, 0.4, 0.4),
            BlockType::Lava => (1.0, 0.3, 0.0),
            BlockType::Obsidian => (0.1, 0.05, 0.15),
            BlockType::Granite => (0.4, 0.35, 0.4),
            BlockType::Marble => (0.9, 0.9, 0.9),
            BlockType::Crystal => (0.3, 0.8, 0.9),
            BlockType::Magma => (0.8, 0.1, 0.0),
            BlockType::Ash => (0.3, 0.3, 0.35),
            _ => (0.5, 0.5, 0.5),
        }
    }
    
    fn get_visible_faces(&self, chunk: &ChunkData, x: usize, y: usize, z: usize) -> Vec<Face> {
        use worlds_shared::BlockType;
        
        let mut faces = Vec::new();
        
        let block = chunk.get_block(x, y, z);
        if block == BlockType::Water || block == BlockType::Air {
            return faces;
        }
        
        let air_or_water = |ch: &ChunkData, px: usize, py: usize, pz: usize| -> bool {
            if px >= 16 || py >= 16 || pz >= 16 {
                return true;
            }
            let b = ch.get_block(px, py, pz);
            b == BlockType::Air || b == BlockType::Water
        };
        
        if x == 0 || air_or_water(chunk, x - 1, y, z) { faces.push(Face::Left); }
        if x == 15 || air_or_water(chunk, x + 1, y, z) { faces.push(Face::Right); }
        if y == 0 || air_or_water(chunk, x, y - 1, z) { faces.push(Face::Bottom); }
        if y == 15 || air_or_water(chunk, x, y + 1, z) { faces.push(Face::Top); }
        if z == 0 || air_or_water(chunk, x, y, z - 1) { faces.push(Face::Front); }
        if z == 15 || air_or_water(chunk, x, y, z + 1) { faces.push(Face::Back); }
        
        faces
    }
    
    fn face_vertices(&self, x: usize, y: usize, z: usize, face: Face, ox: i32, oy: i32, oz: i32) -> (Vec<f32>, Vec<f32>) {
        let (vx, ny): (Vec<f32>, Vec<f32>) = match face {
            Face::Top => (
                vec![
                    (x as f32) + ox as f32, (y as f32 + 1.0) + oy as f32, (z as f32) + oz as f32,
                    (x as f32 + 1.0) + ox as f32, (y as f32 + 1.0) + oy as f32, (z as f32) + oz as f32,
                    (x as f32 + 1.0) + ox as f32, (y as f32 + 1.0) + oy as f32, (z as f32 + 1.0) + oz as f32,
                    (x as f32) + ox as f32, (y as f32 + 1.0) + oy as f32, (z as f32 + 1.0) + oz as f32,
                ],
                vec![0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0],
            ),
            Face::Bottom => (
                vec![
                    (x as f32) + ox as f32, (y as f32) + oy as f32, (z as f32 + 1.0) + oz as f32,
                    (x as f32 + 1.0) + ox as f32, (y as f32) + oy as f32, (z as f32 + 1.0) + oz as f32,
                    (x as f32 + 1.0) + ox as f32, (y as f32) + oy as f32, (z as f32) + oz as f32,
                    (x as f32) + ox as f32, (y as f32) + oy as f32, (z as f32) + oz as f32,
                ],
                vec![0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0],
            ),
            Face::Front => (
                vec![
                    (x as f32) + ox as f32, (y as f32) + oy as f32, (z as f32) + oz as f32,
                    (x as f32 + 1.0) + ox as f32, (y as f32) + oy as f32, (z as f32) + oz as f32,
                    (x as f32 + 1.0) + ox as f32, (y as f32 + 1.0) + oy as f32, (z as f32) + oz as f32,
                    (x as f32) + ox as f32, (y as f32 + 1.0) + oy as f32, (z as f32) + oz as f32,
                ],
                vec![0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0],
            ),
            Face::Back => (
                vec![
                    (x as f32 + 1.0) + ox as f32, (y as f32) + oy as f32, (z as f32 + 1.0) + oz as f32,
                    (x as f32) + ox as f32, (y as f32) + oy as f32, (z as f32 + 1.0) + oz as f32,
                    (x as f32) + ox as f32, (y as f32 + 1.0) + oy as f32, (z as f32 + 1.0) + oz as f32,
                    (x as f32 + 1.0) + ox as f32, (y as f32 + 1.0) + oy as f32, (z as f32 + 1.0) + oz as f32,
                ],
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
            ),
            Face::Left => (
                vec![
                    (x as f32) + ox as f32, (y as f32) + oy as f32, (z as f32 + 1.0) + oz as f32,
                    (x as f32) + ox as f32, (y as f32) + oy as f32, (z as f32) + oz as f32,
                    (x as f32) + ox as f32, (y as f32 + 1.0) + oy as f32, (z as f32) + oz as f32,
                    (x as f32) + ox as f32, (y as f32 + 1.0) + oy as f32, (z as f32 + 1.0) + oz as f32,
                ],
                vec![-1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0],
            ),
            Face::Right => (
                vec![
                    (x as f32 + 1.0) + ox as f32, (y as f32) + oy as f32, (z as f32) + oz as f32,
                    (x as f32 + 1.0) + ox as f32, (y as f32) + oy as f32, (z as f32 + 1.0) + oz as f32,
                    (x as f32 + 1.0) + ox as f32, (y as f32 + 1.0) + oy as f32, (z as f32 + 1.0) + oz as f32,
                    (x as f32 + 1.0) + ox as f32, (y as f32 + 1.0) + oy as f32, (z as f32) + oz as f32,
                ],
                vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            ),
        };
        
        (vx, ny)
    }
    
    pub fn init(&self) {
        console_log!("VoxelEngine initialized with seed: {}", self.seed);
    }
}

#[derive(Clone, Copy)]
enum Face {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}