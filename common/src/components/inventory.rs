#[derive(Clone, Copy, Debug)]
pub enum Item {
    Axe,
}

pub struct Inventory {
    slots: Vec<Item>,
    selected: u8,
    max_size: u8,
}
