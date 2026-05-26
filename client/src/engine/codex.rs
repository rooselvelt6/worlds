#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CodexEntry {
    pub name: String,
    pub icon: String,
    pub discovered: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Codex {
    pub biomes: Vec<CodexEntry>,
    pub structures: Vec<CodexEntry>,
    pub minerals: Vec<CodexEntry>,
    pub creatures: Vec<CodexEntry>,
}

impl Codex {
    pub fn new() -> Self {
        let biomes = vec![
            CodexEntry { name: "Forest".into(), icon: "🌲".into(), discovered: true },
            CodexEntry { name: "Plains".into(), icon: "🌾".into(), discovered: false },
            CodexEntry { name: "Desert".into(), icon: "🏜️".into(), discovered: false },
            CodexEntry { name: "Tundra".into(), icon: "❄️".into(), discovered: false },
            CodexEntry { name: "Jungle".into(), icon: "🌴".into(), discovered: false },
            CodexEntry { name: "Volcanic".into(), icon: "🌋".into(), discovered: false },
            CodexEntry { name: "Ocean".into(), icon: "🌊".into(), discovered: false },
            CodexEntry { name: "Crystal".into(), icon: "💎".into(), discovered: false },
            CodexEntry { name: "Cave".into(), icon: "🕳️".into(), discovered: false },
            CodexEntry { name: "Lava".into(), icon: "🔥".into(), discovered: false },
            CodexEntry { name: "Fungus".into(), icon: "🍄".into(), discovered: false },
            CodexEntry { name: "Abyss".into(), icon: "👁️".into(), discovered: false },
            CodexEntry { name: "Storm".into(), icon: "⛈️".into(), discovered: false },
            CodexEntry { name: "Aurora".into(), icon: "🌌".into(), discovered: false },
            CodexEntry { name: "Magma".into(), icon: "🟠".into(), discovered: false },
        ];
        let structures = vec![
            CodexEntry { name: "Hut".into(), icon: "🏠".into(), discovered: false },
            CodexEntry { name: "Tower".into(), icon: "🗼".into(), discovered: false },
            CodexEntry { name: "Ruins".into(), icon: "🏛️".into(), discovered: false },
            CodexEntry { name: "Arch".into(), icon: "⛩️".into(), discovered: false },
            CodexEntry { name: "Pillar".into(), icon: "🗽".into(), discovered: false },
            CodexEntry { name: "Dome".into(), icon: "🏟️".into(), discovered: false },
            CodexEntry { name: "Pyramid".into(), icon: "🔺".into(), discovered: false },
            CodexEntry { name: "Crystal Spire".into(), icon: "🔮".into(), discovered: false },
            CodexEntry { name: "Mushroom Hut".into(), icon: "🍄".into(), discovered: false },
            CodexEntry { name: "Obelisk".into(), icon: "🗿".into(), discovered: false },
            CodexEntry { name: "Plaza".into(), icon: "🏛️".into(), discovered: false },
            CodexEntry { name: "Muralla".into(), icon: "🧱".into(), discovered: false },
            CodexEntry { name: "Dungeon Entrance".into(), icon: "🕳️".into(), discovered: false },
        ];
        let minerals = vec![
            CodexEntry { name: "Iron".into(), icon: "🪨".into(), discovered: false },
            CodexEntry { name: "Copper".into(), icon: "🪨".into(), discovered: false },
            CodexEntry { name: "Coal".into(), icon: "🪨".into(), discovered: false },
            CodexEntry { name: "Crystal".into(), icon: "💎".into(), discovered: false },
            CodexEntry { name: "Gold".into(), icon: "🪙".into(), discovered: false },
            CodexEntry { name: "Ruby".into(), icon: "💠".into(), discovered: false },
            CodexEntry { name: "Sapphire".into(), icon: "🔷".into(), discovered: false },
            CodexEntry { name: "Amber".into(), icon: "🟧".into(), discovered: false },
        ];
        let creatures = vec![
            CodexEntry { name: "Deer".into(), icon: "🦌".into(), discovered: false },
            CodexEntry { name: "Snake".into(), icon: "🐍".into(), discovered: false },
            CodexEntry { name: "Bird".into(), icon: "🦅".into(), discovered: false },
            CodexEntry { name: "Spirit".into(), icon: "👻".into(), discovered: false },
            CodexEntry { name: "Bat".into(), icon: "🦇".into(), discovered: false },
            CodexEntry { name: "Lizard".into(), icon: "🦎".into(), discovered: false },
            CodexEntry { name: "Rabbit".into(), icon: "🐰".into(), discovered: false },
            CodexEntry { name: "Fox".into(), icon: "🦊".into(), discovered: false },
            CodexEntry { name: "Fish".into(), icon: "🐟".into(), discovered: false },
            CodexEntry { name: "Crab".into(), icon: "🦀".into(), discovered: false },
        ];
        Self { biomes, structures, minerals, creatures }
    }

    pub fn discover_biome(&mut self, name: &str) {
        if let Some(e) = self.biomes.iter_mut().find(|e| e.name == name) {
            e.discovered = true;
        }
    }

    pub fn discover_structure(&mut self, name: &str) {
        if let Some(e) = self.structures.iter_mut().find(|e| e.name == name) {
            e.discovered = true;
        }
    }

    pub fn discover_mineral(&mut self, mineral_type: u8) {
        let names = ["Iron", "Copper", "Coal", "Crystal", "Gold", "Ruby", "Sapphire", "Amber"];
        if let Some(name) = names.get(mineral_type as usize) {
            if let Some(e) = self.minerals.iter_mut().find(|e| e.name == *name) {
                e.discovered = true;
            }
        }
    }

    pub fn discover_creature(&mut self, creature_type: u8) {
        let names = ["Deer", "Snake", "Bird", "Spirit", "Bat", "Lizard", "Rabbit", "Fox", "Fish", "Crab"];
        if let Some(name) = names.get(creature_type as usize) {
            if let Some(e) = self.creatures.iter_mut().find(|e| e.name == *name) {
                e.discovered = true;
            }
        }
    }
}
