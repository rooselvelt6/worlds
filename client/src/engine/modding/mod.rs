pub mod biome;
pub mod blueprint;
pub mod formula;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use biome::{BiomeDef, VegTypeDef, CustomBiomeDef, ModPalette};
use blueprint::BlueprintDef;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModFile {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub biomes: Option<HashMap<String, BiomeDef>>,
    pub custom_biomes: Option<Vec<CustomBiomeDef>>,
    pub palettes: Option<Vec<ModPalette>>,
    pub formulas: Option<HashMap<String, String>>,
    pub blueprints: Option<HashMap<String, BlueprintDef>>,
}

#[derive(Debug, Clone, Default)]
pub struct ModContext {
    pub active: bool,
    pub mod_url: Option<String>,
    pub mod_name: String,

    pub biome_overrides: HashMap<String, BiomeDef>,
    pub custom_biomes: Vec<CustomBiomeDef>,
    pub custom_biome_index: HashMap<String, u32>,

    pub palettes: HashMap<String, [f32; 3]>,

    pub formulas: HashMap<String, formula::Expr>,

    pub blueprints: HashMap<String, blueprint::ResolvedBlueprint>,
}

thread_local! {
    static MOD_CTX: std::cell::RefCell<ModContext> = std::cell::RefCell::new(ModContext::default());
}

impl ModContext {
    pub fn new() -> Self {
        ModContext {
            active: false,
            mod_url: None,
            mod_name: String::new(),
            biome_overrides: HashMap::new(),
            custom_biomes: Vec::new(),
            custom_biome_index: HashMap::new(),
            palettes: HashMap::new(),
            formulas: HashMap::new(),
            blueprints: HashMap::new(),
        }
    }

    pub fn load_from_json(json: &str) -> Result<Self, String> {
        let mod_file: ModFile = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse mod JSON: {}", e))?;

        let mut ctx = ModContext::new();
        ctx.active = true;
        ctx.mod_name = mod_file.name.clone().unwrap_or_else(|| "Unnamed Mod".into());

        if let Some(biomes) = mod_file.biomes {
            ctx.biome_overrides = biomes;
        }

        if let Some(custom) = mod_file.custom_biomes {
            for (i, biome) in custom.iter().enumerate() {
                ctx.custom_biome_index.insert(biome.id.clone(), i as u32);
            }
            ctx.custom_biomes = custom;
        }

        if let Some(palettes) = mod_file.palettes {
            for palette in palettes {
                for override_color in palette.colors {
                    ctx.palettes.insert(override_color.block_name.clone(), override_color.color);
                }
            }
        }

        if let Some(formulas) = mod_file.formulas {
            for (name, expr_str) in formulas {
                match formula::parse(&expr_str) {
                    Ok(ast) => { ctx.formulas.insert(name, ast); }
                    Err(e) => {
                        web_sys::console::log_1(&format!("[Mod] Formula '{}' parse error: {}", name, e).into());
                    }
                }
            }
        }

        if let Some(blueprints) = mod_file.blueprints {
            for (name, def) in blueprints {
                let resolved = blueprint::resolve_blueprint(&def, [0.5, 0.5, 0.5]);
                ctx.blueprints.insert(name, resolved);
            }
        }

        Ok(ctx)
    }

    pub fn set_active(ctx: ModContext) {
        MOD_CTX.with(|c| *c.borrow_mut() = ctx);
    }

    pub fn with<T>(f: impl FnOnce(&ModContext) -> T) -> T {
        MOD_CTX.with(|c| f(&c.borrow()))
    }

    pub fn with_mut<T>(f: impl FnOnce(&mut ModContext) -> T) -> T {
        MOD_CTX.with(|c| f(&mut c.borrow_mut()))
    }

    pub fn get_biome_color(&self, zone_name: &str) -> Option<[f32; 3]> {
        self.biome_overrides.get(zone_name).and_then(|b| b.color)
    }

    pub fn get_biome_rock_color(&self, zone_name: &str) -> Option<[f32; 3]> {
        self.biome_overrides.get(zone_name).and_then(|b| b.rock_color)
    }

    pub fn get_biome_gradient(&self, zone_name: &str) -> Option<&Vec<[f32; 3]>> {
        self.biome_overrides.get(zone_name).and_then(|b| b.gradient_stops.as_ref())
    }

    pub fn get_biome_height_mult(&self, zone_name: &str) -> Option<f64> {
        self.biome_overrides.get(zone_name).and_then(|b| b.height_multiplier)
    }

    pub fn get_biome_height_offset(&self, zone_name: &str) -> Option<f64> {
        self.biome_overrides.get(zone_name).and_then(|b| b.height_offset)
    }

    pub fn get_biome_struct_density(&self, zone_name: &str) -> Option<f64> {
        self.biome_overrides.get(zone_name).and_then(|b| b.struct_density)
    }

    pub fn get_biome_struct_types(&self, zone_name: &str) -> Option<&Vec<String>> {
        self.biome_overrides.get(zone_name).and_then(|b| b.struct_types.as_ref())
    }

    pub fn get_biome_veg_density(&self, zone_name: &str) -> Option<f64> {
        self.biome_overrides.get(zone_name).and_then(|b| b.vegetation_density)
    }

    pub fn get_biome_veg_types(&self, zone_name: &str) -> Option<&Vec<VegTypeDef>> {
        self.biome_overrides.get(zone_name).and_then(|b| b.vegetation_types.as_ref())
    }

    pub fn get_palette_color(&self, block_name: &str) -> Option<[f32; 3]> {
        self.palettes.get(block_name).copied()
    }

    pub fn eval_formula(&self, name: &str, vars: &HashMap<String, f64>) -> Option<f64> {
        self.formulas.get(name).and_then(|ast| {
            formula::eval(ast, vars).ok()
        })
    }

    pub fn get_blueprint(&self, name: &str) -> Option<&blueprint::ResolvedBlueprint> {
        self.blueprints.get(name)
    }

    pub fn get_custom_biome(&self, id: u32) -> Option<&CustomBiomeDef> {
        self.custom_biomes.get(id as usize)
    }
}

pub fn validate_mod_url(url: &str) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("Mod URL must use https://".to_string());
    }
    if url.contains(' ') || url.contains('\t') || url.contains('\n') || url.contains('\r') {
        return Err("Mod URL contains invalid characters".to_string());
    }
    if url.len() > 2048 {
        return Err("Mod URL too long (max 2048 chars)".to_string());
    }
    Ok(())
}

pub async fn fetch_and_apply_mod(url: &str) -> Result<(), String> {
    validate_mod_url(url)?;
    let window = window().ok_or("No window")?;
    let promise = window.fetch_with_str(url);
    let resp = JsFuture::from(promise).await
        .map_err(|e| format!("Fetch error: {:?}", e))?;
    let resp: web_sys::Response = resp.dyn_into()
        .map_err(|_| "Not a Response".to_string())?;

    // Check content length before reading
    if let Ok(Some(content_length)) = resp.headers().get("Content-Length") {
        if let Ok(size) = content_length.parse::<u64>() {
            if size > 5 * 1024 * 1024 {
                return Err("Mod file too large (max 5MB)".to_string());
            }
        }
    }

    let text_promise = resp.text()
        .map_err(|e| format!("text() error: {:?}", e))?;
    let text = JsFuture::from(text_promise).await
        .map_err(|e| format!("text future error: {:?}", e))?;
    let json_str = text.as_string().ok_or("Not a string")?;

    // Limit response size
    if json_str.len() > 5 * 1024 * 1024 {
        return Err("Mod response too large (max 5MB)".to_string());
    }

    let ctx = ModContext::load_from_json(&json_str)?;
    ModContext::set_active(ctx);
    web_sys::console::log_1(&format!("[Mod] Loaded mod with {} biome overrides, {} formulas, {} blueprints",
        MOD_CTX.with(|c| c.borrow().biome_overrides.len()),
        MOD_CTX.with(|c| c.borrow().formulas.len()),
        MOD_CTX.with(|c| c.borrow().blueprints.len()),
    ).into());
    Ok(())
}
