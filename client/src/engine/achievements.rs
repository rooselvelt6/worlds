use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Achievement {
    FirstMine,
    Explorer,
    Adventurer,
    StructureHunter,
    MasterCrafter,
    Walk1km,
    Walk10km,
    Biomes5,
    BiomesAll,
    Builder,
}

pub const ALL_ACHIEVEMENTS: &[Achievement] = &[
    Achievement::FirstMine,
    Achievement::Explorer,
    Achievement::Adventurer,
    Achievement::StructureHunter,
    Achievement::MasterCrafter,
    Achievement::Walk1km,
    Achievement::Walk10km,
    Achievement::Biomes5,
    Achievement::BiomesAll,
    Achievement::Builder,
];

pub fn achievement_name(a: &Achievement) -> &'static str {
    match a {
        Achievement::FirstMine => "Primera minería",
        Achievement::Explorer => "Explorador",
        Achievement::Adventurer => "Aventurero",
        Achievement::StructureHunter => "Cazador de estructuras",
        Achievement::MasterCrafter => "Maestro artesano",
        Achievement::Walk1km => "Caminante (1km)",
        Achievement::Walk10km => "Maratonista (10km)",
        Achievement::Biomes5 => "Biomas: 5 descubiertos",
        Achievement::BiomesAll => "Biomas: todos descubiertos",
        Achievement::Builder => "Constructor",
    }
}

pub fn achievement_icon(a: &Achievement) -> &'static str {
    match a {
        Achievement::FirstMine => "⛏️",
        Achievement::Explorer => "🧭",
        Achievement::Adventurer => "🗺️",
        Achievement::StructureHunter => "🏛️",
        Achievement::MasterCrafter => "⚒️",
        Achievement::Walk1km => "🚶",
        Achievement::Walk10km => "🏃",
        Achievement::Biomes5 => "🌿",
        Achievement::BiomesAll => "🌍",
        Achievement::Builder => "🏗️",
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AchievementState {
    pub unlocked: HashSet<Achievement>,
    pub total_distance: f64,
    pub structures_found: u32,
    pub items_crafted: u32,
    pub blocks_placed: u32,
    pub pending: Vec<Achievement>,
}

impl AchievementState {
    pub fn new() -> Self {
        Self {
            unlocked: HashSet::new(),
            total_distance: 0.0,
            structures_found: 0,
            items_crafted: 0,
            blocks_placed: 0,
            pending: Vec::new(),
        }
    }

    pub fn try_unlock(&mut self, a: Achievement) -> Option<&Achievement> {
        if self.unlocked.insert(a.clone()) {
            self.pending.push(a.clone());
            Some(&self.pending[self.pending.len() - 1])
        } else {
            None
        }
    }

    pub fn check_distance(&mut self, total: f64) {
        if total > 1000.0 { self.try_unlock(Achievement::Walk1km); }
        if total > 10000.0 { self.try_unlock(Achievement::Walk10km); }
    }

    pub fn check_biomes(&mut self, discovered: &[String], all_biomes: &[&str]) {
        let count = discovered.len();
        if count >= 5 { self.try_unlock(Achievement::Biomes5); }
        if count >= all_biomes.len() { self.try_unlock(Achievement::BiomesAll); }
    }

    pub fn check_structures(&mut self, count: u32) {
        if count >= 1 { self.try_unlock(Achievement::Adventurer); }
        if count >= 10 { self.try_unlock(Achievement::StructureHunter); }
    }

    pub fn check_craft(&mut self, count: u32) {
        if count >= 1 { self.try_unlock(Achievement::MasterCrafter); }
    }

    pub fn check_build(&mut self, count: u32) {
        if count >= 10 { self.try_unlock(Achievement::Builder); }
    }
}
