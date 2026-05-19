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

    pub fn craft(&mut self) -> String {
        let types: Vec<u8> = self.items.iter().filter(|i| i.count >= 2).map(|i| i.mineral_type).collect();
        if types.is_empty() { return "".to_string(); }
        let craft_type = types[0];
        if let Some(item) = self.items.iter_mut().find(|i| i.mineral_type == craft_type) {
            item.count -= 2;
        }
        format!("craft_{}_ingot", craft_type)
    }
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
