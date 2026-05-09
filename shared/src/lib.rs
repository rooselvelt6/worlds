use serde::{Deserialize, Serialize};
use noise::{NoiseFn, Perlin, Simplex};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FormulaType {
    FBM,
    Perlin,
    Simplex,
    Voronoi,
    Mandelbrot,
    Sierpinski,
    Billow,
    RidgedMF,
    DomainWarp,
    Hybrid,
}

impl FormulaType {
    pub fn name(&self) -> &'static str {
        match self {
            FormulaType::FBM => "FBM",
            FormulaType::Perlin => "Perlin",
            FormulaType::Simplex => "Simplex",
            FormulaType::Voronoi => "Voronoi",
            FormulaType::Mandelbrot => "Mandelbrot",
            FormulaType::Sierpinski => "Sierpinski",
            FormulaType::Billow => "Billow",
            FormulaType::RidgedMF => "RidgedMF",
            FormulaType::DomainWarp => "DomainWarp",
            FormulaType::Hybrid => "Hybrid",
        }
    }
    
    pub fn all() -> Vec<FormulaType> {
        vec![
            FormulaType::FBM,
            FormulaType::Perlin,
            FormulaType::Simplex,
            FormulaType::Voronoi,
            FormulaType::Mandelbrot,
            FormulaType::Sierpinski,
            FormulaType::Billow,
            FormulaType::RidgedMF,
            FormulaType::DomainWarp,
            FormulaType::Hybrid,
        ]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SubWorldKind {
    FractalForest,
    CrystalCavern,
    VoronoiCity,
    MandelbrotRealm,
    SierpinskiTemple,
    SimplexPlains,
    RidgedMountains,
    DomainWarpVoid,
    BillowOcean,
    HybridDream,
}

impl SubWorldKind {
    pub fn formula(&self) -> FormulaType {
        match self {
            SubWorldKind::FractalForest => FormulaType::FBM,
            SubWorldKind::CrystalCavern => FormulaType::Simplex,
            SubWorldKind::VoronoiCity => FormulaType::Voronoi,
            SubWorldKind::MandelbrotRealm => FormulaType::Mandelbrot,
            SubWorldKind::SierpinskiTemple => FormulaType::Sierpinski,
            SubWorldKind::SimplexPlains => FormulaType::Simplex,
            SubWorldKind::RidgedMountains => FormulaType::RidgedMF,
            SubWorldKind::DomainWarpVoid => FormulaType::DomainWarp,
            SubWorldKind::BillowOcean => FormulaType::Billow,
            SubWorldKind::HybridDream => FormulaType::Hybrid,
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            SubWorldKind::FractalForest => "Fractal Forest",
            SubWorldKind::CrystalCavern => "Crystal Cavern",
            SubWorldKind::VoronoiCity => "Voronoi City",
            SubWorldKind::MandelbrotRealm => "Mandelbrot Realm",
            SubWorldKind::SierpinskiTemple => "Sierpinski Temple",
            SubWorldKind::SimplexPlains => "Simplex Plains",
            SubWorldKind::RidgedMountains => "Ridged Mountains",
            SubWorldKind::DomainWarpVoid => "Domain Warp Void",
            SubWorldKind::BillowOcean => "Billow Ocean",
            SubWorldKind::HybridDream => "Hybrid Dream",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            SubWorldKind::FractalForest => "Fractal Brownian Motion terrain",
            SubWorldKind::CrystalCavern => "Simplex noise crystal caves",
            SubWorldKind::VoronoiCity => "Cellular Voronoi structures",
            SubWorldKind::MandelbrotRealm => "Mandelbrot set fractals",
            SubWorldKind::SierpinskiTemple => "Sierpinski triangle fractals",
            SubWorldKind::SimplexPlains => "Smooth Simplex noise plains",
            SubWorldKind::RidgedMountains => "Ridged multifractal mountains",
            SubWorldKind::DomainWarpVoid => "Domain warped void dimension",
            SubWorldKind::BillowOcean => "Billow noise ocean floor",
            SubWorldKind::HybridDream => "Hybrid fractal terrain",
        }
    }
    
    pub fn all() -> Vec<SubWorldKind> {
        vec![
            SubWorldKind::FractalForest,
            SubWorldKind::CrystalCavern,
            SubWorldKind::VoronoiCity,
            SubWorldKind::MandelbrotRealm,
            SubWorldKind::SierpinskiTemple,
            SubWorldKind::SimplexPlains,
            SubWorldKind::RidgedMountains,
            SubWorldKind::DomainWarpVoid,
            SubWorldKind::BillowOcean,
            SubWorldKind::HybridDream,
        ]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BlockType {
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Water,
    Snow,
    Ice,
    Wood,
    Leaves,
    Cobblestone,
    Lava,
    Obsidian,
    Granite,
    Marble,
    Crystal,
    Magma,
    Ash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldParams {
    pub seed: u32,
    pub scale: f64,
    pub octaves: u8,
    pub amplitude: f64,
    pub water_level: f64,
    pub cave_density: f64,
    pub cave_size: f64,
    pub mountain_scale: f64,
    pub biome_weight: f64,
}

impl Default for WorldParams {
    fn default() -> Self {
        Self {
            seed: 42,
            scale: 0.02,
            octaves: 6,
            amplitude: 20.0,
            water_level: 8.0,
            cave_density: 0.4,
            cave_size: 0.1,
            mountain_scale: 1.5,
            biome_weight: 0.3,
        }
    }
}

impl WorldParams {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
    
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        serde_json::from_slice(data).ok()
    }
}

impl BlockType {
    pub fn to_u8(&self) -> u8 {
        match self {
            BlockType::Air => 0,
            BlockType::Grass => 1,
            BlockType::Dirt => 2,
            BlockType::Stone => 3,
            BlockType::Sand => 4,
            BlockType::Water => 5,
            BlockType::Snow => 6,
            BlockType::Ice => 7,
            BlockType::Wood => 8,
            BlockType::Leaves => 9,
            BlockType::Cobblestone => 10,
            BlockType::Lava => 11,
            BlockType::Obsidian => 12,
            BlockType::Granite => 13,
            BlockType::Marble => 14,
            BlockType::Crystal => 15,
            BlockType::Magma => 16,
            BlockType::Ash => 17,
        }
    }
    
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => BlockType::Air,
            1 => BlockType::Grass,
            2 => BlockType::Dirt,
            3 => BlockType::Stone,
            4 => BlockType::Sand,
            5 => BlockType::Water,
            6 => BlockType::Snow,
            7 => BlockType::Ice,
            8 => BlockType::Wood,
            9 => BlockType::Leaves,
            10 => BlockType::Cobblestone,
            11 => BlockType::Lava,
            12 => BlockType::Obsidian,
            13 => BlockType::Granite,
            14 => BlockType::Marble,
            15 => BlockType::Crystal,
            16 => BlockType::Magma,
            17 => BlockType::Ash,
            _ => BlockType::Air,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkData {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub blocks: Vec<u8>,
    pub heightmap: Vec<f32>,
    pub biome: String,
}

impl ChunkData {
    pub const SIZE: usize = 16;
    pub const VOLUME: usize = Self::SIZE * Self::SIZE * Self::SIZE;

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            x,
            y,
            z,
            blocks: vec![0; Self::VOLUME],
            heightmap: vec![0.0; Self::SIZE * Self::SIZE],
            biome: String::new(),
        }
    }

    pub fn index(x: usize, y: usize, z: usize) -> usize {
        (y * Self::SIZE * Self::SIZE) + (z * Self::SIZE) + x
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        let idx = Self::index(x, y, z);
        if idx < self.blocks.len() {
            self.blocks[idx] = block.to_u8();
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        let idx = Self::index(x, y, z);
        if idx < self.blocks.len() {
            BlockType::from_u8(self.blocks[idx])
        } else {
            BlockType::Air
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubWorldData {
    pub id: u64,
    pub parent_id: u64,
    pub kind: SubWorldKind,
    pub chunks: Vec<ChunkData>,
}

#[derive(Clone)]
pub struct WorldGenerator {
    seed: u32,
    perlin: Perlin,
    simplex: Simplex,
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        let perlin = Perlin::new(seed);
        let simplex = Simplex::new(seed);
        Self {
            seed,
            perlin,
            simplex,
        }
    }

    pub fn with_params(seed: u32) -> Self {
        Self::new(seed)
    }
    
    pub fn set_seed(&mut self, seed: u32) {
        self.seed = seed;
    }
    
    pub fn get_seed(&self) -> u32 {
        self.seed
    }
    
    // ALL FORMULA TYPES
    pub fn calculate(&self, x: f64, z: f64, formula: FormulaType, seed: u32) -> f64 {
        match formula {
            FormulaType::FBM => self.fbm(x, z, 0.0, 6, 0.5),
            FormulaType::Perlin => self.perlin.get([x, z, seed as f64]),
            FormulaType::Simplex => self.simplex.get([x, z, seed as f64]),
            FormulaType::Voronoi => self.voronoi(x, z),
            FormulaType::Mandelbrot => self.mandelbrot(x, z, 4.0),
            FormulaType::Sierpinski => self.sierpinski(x as i32, z as i32),
            FormulaType::Billow => self.billow(x, z, 0.0, 4),
            FormulaType::RidgedMF => self.ridged_fbm(x, z, 0.0, 6),
            FormulaType::DomainWarp => self.domain_warp(x, z),
            FormulaType::Hybrid => self.hybrid(x, z),
        }
    }
    
    fn voronoi(&self, x: f64, z: f64) -> f64 {
        let ix = x.floor() as i32;
        let iz = z.floor() as i32;
        let mut min_dist = f64::MAX;
        for dx in -1..=1 {
            for dz in -1..=1 {
                let cx = ix + dx;
                let cz = iz + dz;
                let hash = self.hash2d(cx, cz, self.seed);
                let cell_x = cx as f64 + hash.0;
                let cell_z = cz as f64 + hash.1;
                let dist = ((x - cell_x).powi(2) + (z - cell_z).powi(2)).sqrt();
                min_dist = min_dist.min(dist);
            }
        }
        min_dist
    }
    
    fn hash2d(&self, x: i32, z: i32, seed: u32) -> (f64, f64) {
        let ix = x as i64;
        let iz = z as i64;
        let iseed = seed as i64;
        let n = (ix.wrapping_mul(12742).wrapping_mul(iz.wrapping_mul(31337)).wrapping_add(iseed)) as f64;
        let n = n.sin() * 43758.5453;
        let fract = n - n.floor();
        (fract, (n * 1.3).fract())
    }
    
    fn mandelbrot(&self, cx: f64, cy: f64, max_iter: f64) -> f64 {
        let x0 = cx * 2.5 - 2.0;
        let y0 = cy * 2.0 - 1.0;
        let mut x = 0.0;
        let mut y = 0.0;
        let mut iter = 0.0;
        while x * x + y * y <= 4.0 && iter < max_iter {
            let xtemp = x * x - y * y + x0;
            y = 2.0 * x * y + y0;
            x = xtemp;
            iter += 1.0;
        }
        iter / max_iter
    }
    
    fn sierpinski(&self, x: i32, z: i32) -> f64 {
        let mut px = x as u32;
        let mut pz = z as u32;
        let mut count = 0;
        while (px | pz) != 0 {
            if (px & 1) == 1 && (pz & 1) == 1 {
                count += 1;
            }
            px >>= 1;
            pz >>= 1;
        }
        if count % 2 == 0 { 0.0 } else { 1.0 }
    }
    
    fn billow(&self, x: f64, y: f64, z: f64, octaves: u8) -> f64 {
        let val = self.fbm(x, y, z, octaves, 0.5);
        if val >= 0.0 { val } else { -val }
    }
    
    fn domain_warp(&self, x: f64, y: f64) -> f64 {
        let warp_x = x + self.fbm(x + 10.0, y, 0.0, 3, 0.5) * 2.0;
        let warp_y = y + self.fbm(x, y + 10.0, 0.0, 3, 0.5) * 2.0;
        self.fbm(warp_x, warp_y, 0.0, 4, 0.5)
    }
    
    fn hybrid(&self, x: f64, y: f64) -> f64 {
        let f = self.fbm(x, y, 0.0, 4, 0.5);
        let r = self.ridged_fbm(x * 1.5, y * 1.5, 0.0, 4);
        let v = self.voronoi(x * 0.5, y * 0.5);
        (f + r + v) / 3.0
    }
    
    pub fn get_height_formula(&self, wx: f64, wz: f64, params: &WorldParams, formula: FormulaType) -> f64 {
        let nx = wx * params.scale;
        let nz = wz * params.scale;
        
        let base = self.calculate(nx, nz, formula, self.seed);
        
        match formula {
            FormulaType::Mandelbrot => base * params.amplitude * 0.3,
            FormulaType::Sierpinski => base * params.amplitude * 2.0,
            FormulaType::Voronoi => base * params.amplitude * 0.8 + params.water_level,
            _ => base * params.amplitude + params.water_level,
        }.max(0.0)
    }
    
    fn fbm(&self, x: f64, y: f64, z: f64, octaves: u8, persistence: f64) -> f64 {
        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;
        
        for _ in 0..octaves {
            total += self.simplex.get([x * frequency, y * frequency, z * frequency]) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= 2.0;
        }
        
        total / max_value
    }

    fn ridged_fbm(&self, x: f64, y: f64, z: f64, octaves: u8) -> f64 {
        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut prev = 1.0;
        
        for _i in 0..octaves {
            let n = 1.0 - self.simplex.get([x * frequency, y * frequency, z * frequency]).abs();
            let ridge = n * n;
            total += ridge * amplitude * prev;
            prev = ridge;
            amplitude *= 0.5;
            frequency *= 2.0;
        }
        
        total
    }

    fn get_temp_hum(&self, wx: f64, wz: f64) -> (f64, f64) {
        let temp = self.simplex.get([wx * 0.008, wz * 0.008, 100.0]);
        let hum = self.simplex.get([wx * 0.008, wz * 0.008, 200.0]);
        (temp, hum)
    }

    pub fn get_height(&self, wx: f64, wz: f64, params: &WorldParams) -> f64 {
        let formula = FormulaType::FBM;
        self.get_height_formula(wx, wz, params, formula)
    }
    
    pub fn get_height_with_formula(&self, wx: f64, wz: f64, params: &WorldParams, formula: FormulaType) -> f64 {
        self.get_height_formula(wx, wz, params, formula)
    }
    
    pub fn get_height_simple(&self, wx: i32, wz: i32, _chunk_y: i32) -> f64 {
        let params = WorldParams::default();
        self.get_height(wx as f64, wz as f64, &params)
    }

    pub fn get_biome(&self, wx: f64, wz: f64, params: &WorldParams) -> &'static str {
        let (temp, hum) = self.get_temp_hum(wx, wz);
        
        if params.cave_density > 0.6 && (temp > 0.4 || temp < -0.4) {
            return "crystal_cavern";
        }
        
        if temp < -0.4 {
            return "tundra";
        } else if temp > 0.5 {
            if hum < -0.3 {
                return "desert";
            } else {
                return "volcanic";
            }
        } else if hum < -0.3 {
            return "plains";
        } else if hum > 0.5 {
            return "jungle";
        } else if hum > 0.2 {
            return "swamp";
        } else {
            return "forest";
        }
    }
    
    fn _get_biome_simple(&self, wx: i32, wz: i32) -> &'static str {
        let params = WorldParams::default();
        self.get_biome(wx as f64, wz as f64, &params)
    }

    pub fn generate_chunk(&self, x: i32, y: i32, z: i32) -> ChunkData {
        let params = WorldParams::default();
        self.generate_chunk_with_params(x, y, z, &params)
    }
    
    pub fn generate_chunk_with_params(&self, x: i32, y: i32, z: i32, params: &WorldParams) -> ChunkData {
        let mut chunk = ChunkData::new(x, y, z);
        
        let chunk_world_x = x as i32 * ChunkData::SIZE as i32;
        let chunk_world_z = z as i32 * ChunkData::SIZE as i32;
        
        let biome = self.get_biome(chunk_world_x as f64, chunk_world_z as f64, params);
        chunk.biome = biome.to_string();
        
        for lx in 0..ChunkData::SIZE {
            for lz in 0..ChunkData::SIZE {
                let wx = chunk_world_x + lx as i32;
                let wz = chunk_world_z + lz as i32;
                
                let ground_height = self.get_height(wx as f64, wz as f64, params).max(0.0) as f32;
                chunk.heightmap[lz * ChunkData::SIZE + lx] = ground_height;
                
                let surface_block = match biome {
                    "desert" => BlockType::Sand,
                    "tundra" => BlockType::Snow,
                    "volcanic" => BlockType::Ash,
                    "swamp" => BlockType::Dirt,
                    _ => BlockType::Grass,
                };
                
                let ground_y = (ground_height as usize).min(ChunkData::SIZE - 1);
                
                for ly in 0..=ground_y {
                    let mut block = if ly == ground_y {
                        surface_block
                    } else if ly > ground_y - 3 {
                        BlockType::Dirt
                    } else {
                        BlockType::Stone
                    };
                    
                    if biome == "crystal_cavern" && ly < ground_y - 2 {
                        if (lx + ly + lz) % 7 == 0 {
                            block = BlockType::Crystal;
                        }
                    }
                    
                    if biome == "volcanic" && ly < ground_y - 1 {
                        if (lx + ly + lz) % 9 == 0 {
                            block = BlockType::Magma;
                        }
                    }
                    
                    chunk.set_block(lx, ly, lz, block);
                }
                
                if ground_y > 0 && ground_y < ChunkData::SIZE - 1 && params.water_level > 0.0 {
                    let water_surface = params.water_level as usize;
                    if ground_y < water_surface {
                        for ly in (ground_y + 1)..=water_surface.min(ChunkData::SIZE - 1) {
                            let water_block = if biome == "tundra" { BlockType::Ice } else { BlockType::Water };
                            chunk.set_block(lx, ly, lz, water_block);
                        }
                    }
                }
            }
        }
        
        if y < 0 {
            self.generate_cave_chunk(&mut chunk, params);
        }
        
        chunk
    }
    
    fn generate_cave_chunk(&self, chunk: &mut ChunkData, params: &WorldParams) {
        let chunk_world_x = chunk.x as i32 * ChunkData::SIZE as i32;
        let chunk_world_y = chunk.y as i32 * ChunkData::SIZE as i32;
        let chunk_world_z = chunk.z as i32 * ChunkData::SIZE as i32;
        
        for lx in 0..ChunkData::SIZE {
            for ly in 0..ChunkData::SIZE {
                for lz in 0..ChunkData::SIZE {
                    let wx = chunk_world_x + lx as i32;
                    let wy = chunk_world_y + ly as i32;
                    let wz = chunk_world_z + lz as i32;
                    
                    let cave_noise = self.simplex.get([
                        wx as f64 * params.cave_size,
                        wy as f64 * params.cave_size,
                        wz as f64 * params.cave_size,
                    ]);
                    
                    let cave_threshold = 0.6 - params.cave_density * 0.4;
                    
                    if cave_noise > cave_threshold && chunk.get_block(lx, ly, lz) != BlockType::Air {
                        chunk.set_block(lx, ly, lz, BlockType::Air);
                        
                        if cave_noise > cave_threshold + 0.2 {
                            let crystal_pos = (lx + ly + lz) % 12 == 0;
                            if crystal_pos {
                                chunk.set_block(lx, ly, lz, BlockType::Crystal);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_block_at(&self, x: i32, y: i32, z: i32, params: &WorldParams) -> BlockType {
        let ground_height = self.get_height(x as f64, z as f64, params) as i32;
        
        if y > ground_height {
            if params.water_level > 0.0 && y as f64 <= params.water_level {
                return BlockType::Water;
            }
            return BlockType::Air;
        }
        
        let biome = self.get_biome(x as f64, z as f64, params);
        
        if y == ground_height {
            match biome {
                "desert" => BlockType::Sand,
                "tundra" => BlockType::Snow,
                "volcanic" => BlockType::Ash,
                _ => BlockType::Grass,
            }
        } else if y > ground_height - 3 {
            BlockType::Dirt
        } else {
            BlockType::Stone
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldMessage {
    GenerateChunk { x: i32, y: i32, z: i32, seed: u32 },
    ChunkReady { x: i32, y: i32, z: i32, data: ChunkData },
    GenerateSubWorld { parent_id: u64, kind: SubWorldKind },
    SubWorldReady { parent_id: u64, subworld: SubWorldData },
    AvatarMoved { x: f32, y: f32, z: f32 },
    Error { message: String },
}