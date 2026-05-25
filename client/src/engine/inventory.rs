const MAX_INVENTORY_SLOTS: usize = 16;

#[derive(Clone, Debug)]
pub struct InventoryItem {
    pub mineral_type: u8,
    pub count: u32,
}

#[derive(Clone, Debug)]
pub struct Inventory {
    pub items: Vec<InventoryItem>,
    pub selected_slot: u8,
}

impl Inventory {
    pub fn new() -> Self {
        let mut items = Vec::with_capacity(MAX_INVENTORY_SLOTS);
        for i in 0..9 {
            items.push(InventoryItem { mineral_type: i as u8, count: 0 });
        }
        Self { items, selected_slot: 0 }
    }

    pub fn add_mineral(&mut self, mineral_type: u8, amount: u32) {
        if let Some(item) = self.items.iter_mut().find(|i| i.mineral_type == mineral_type) {
            item.count += amount;
        } else if self.items.len() < MAX_INVENTORY_SLOTS {
            self.items.push(InventoryItem { mineral_type, count: amount });
        }
    }

    pub fn selected_item(&self) -> Option<&InventoryItem> {
        self.items.get(self.selected_slot as usize)
    }

    pub fn has(&self, mineral_type: u8, count: u32) -> bool {
        self.items.iter().any(|i| i.mineral_type == mineral_type && i.count >= count)
    }

    pub fn consume(&mut self, mineral_type: u8, count: u32) -> bool {
        if let Some(item) = self.items.iter_mut().find(|i| i.mineral_type == mineral_type) {
            if item.count >= count {
                item.count -= count;
                return true;
            }
        }
        false
    }
}

#[derive(Clone, Debug)]
pub struct CraftRecipe {
    pub name: &'static str,
    pub icon: &'static str,
    pub ingredients: &'static [(u8, u32)],
    pub result: (&'static str, u8, u32),
}

pub const RECIPES: &[CraftRecipe] = &[
    CraftRecipe { name: "Pico de hierro", icon: "⛏️", ingredients: &[(0, 3)], result: ("Pico de hierro", 10, 1) },
    CraftRecipe { name: "Espada de piedra", icon: "🗡️", ingredients: &[(1, 2), (2, 1)], result: ("Espada de piedra", 11, 1) },
    CraftRecipe { name: "Antorcha", icon: "🔥", ingredients: &[(2, 1), (7, 1)], result: ("Antorcha", 12, 4) },
    CraftRecipe { name: "Varita de cristal", icon: "🪄", ingredients: &[(3, 2), (6, 1)], result: ("Varita de cristal", 13, 1) },
    CraftRecipe { name: "Poción de vida", icon: "🧪", ingredients: &[(7, 2)], result: ("Poción de vida", 14, 1) },
    CraftRecipe { name: "Pico avanzado", icon: "⚒️", ingredients: &[(0, 2), (3, 1), (4, 1)], result: ("Pico avanzado", 15, 1) },
    CraftRecipe { name: "Escudo de oro", icon: "🛡️", ingredients: &[(6, 3), (1, 2)], result: ("Escudo de oro", 16, 1) },
    CraftRecipe { name: "Anillo de poder", icon: "💍", ingredients: &[(4, 2), (6, 1), (7, 1)], result: ("Anillo de poder", 17, 1) },
];

pub fn can_craft(recipe: &CraftRecipe, inv: &Inventory) -> bool {
    recipe.ingredients.iter().all(|(mt, count)| inv.has(*mt, *count))
}

pub fn perform_craft(recipe: &CraftRecipe, inv: &mut Inventory) -> String {
    for (mt, count) in recipe.ingredients {
        if !inv.consume(*mt, *count) {
            return "".to_string();
        }
    }
    inv.add_mineral(recipe.result.1, recipe.result.2);
    format!("{}: {}", recipe.icon, recipe.name)
}

#[derive(Clone)]
pub struct BlockType {
    pub id: u8,
    pub name: &'static str,
    pub color: [f32; 3],
}

pub const BLOCK_TYPES: &[BlockType] = &[
    BlockType { id: 0, name: "Dirt", color: [0.6, 0.45, 0.3] },
    BlockType { id: 1, name: "Stone", color: [0.5, 0.5, 0.5] },
    BlockType { id: 2, name: "Wood", color: [0.55, 0.35, 0.15] },
    BlockType { id: 3, name: "Leaves", color: [0.2, 0.6, 0.2] },
    BlockType { id: 4, name: "Crystal", color: [0.7, 0.4, 1.0] },
    BlockType { id: 5, name: "Lava Stone", color: [0.8, 0.3, 0.05] },
    BlockType { id: 6, name: "Ice", color: [0.7, 0.9, 1.0] },
    BlockType { id: 7, name: "Sand", color: [0.85, 0.75, 0.5] },
    BlockType { id: 8, name: "Moss", color: [0.3, 0.5, 0.2] },
];
