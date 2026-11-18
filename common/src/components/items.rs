#[derive(Clone, Copy, Debug)]
pub struct Item {
    ty: ItemType,
    durability: f32,
}

impl Item {
    pub fn new(ty: ItemType) -> Self {
        Self {
            ty,
            durability: ty.durability().unwrap_or(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ItemType {
    Axe,
}

impl ItemType {
    pub fn durability(&self) -> Option<f32> {
        match self {
            ItemType::Axe => Some(5.0),
        }
    }
}

pub struct Inventory {
    pub slots: Vec<Item>,
    pub selected: u8,
    pub max_size: u8,
}
