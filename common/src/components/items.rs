use serde::{ Serialize, Deserialize };

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub ty: ItemType,
    pub durability: f32,
    pub last_used: u128,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ItemType {
    Axe,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<Item>,
    pub selected: u8,
    pub max_size: u8,
}

impl Item {
    pub fn new(ty: ItemType) -> Self {
        let durability = ty.durability().unwrap_or(0.0);
        Self {
            ty,
            durability,
            last_used: 0,
        }
    }
}

impl ItemType {
    pub fn durability(&self) -> Option<f32> {
        match self {
            ItemType::Axe => Some(5.0),
        }
    }

    pub fn deploy_time(&self) -> u128 {
        // deploy time in ms
        match self {
            ItemType::Axe => 400,
        }
    }

    pub fn firing_cooldown_time(&self) -> u128 {
        // firing cooldown in ms
        match self {
            ItemType::Axe => 800,
        }
    }
}

impl Inventory {
    pub fn len(&self) -> u8 {
        self.slots.len().try_into().unwrap()
    }

    pub fn selected(&mut self) -> Option<&mut Item> {
        let len = self.len();
        match len {
            // inventory is empty
            0 => None,
            // not empty and self.selected is valid
            _ if self.selected < len => self.slots.get_mut(self.selected as usize),
            // self.selected is invalid
            _ => {
                self.selected = len - 1;
                self.slots.get_mut((len - 1) as usize)
            }
        }
    }
}
