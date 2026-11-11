mod setup;
pub use setup::*;
mod utils;
pub use utils::*;
mod messages;
pub use messages::*;

pub static RENET_CHANNELS: &[u8] = &[
    0, // Unreliable
    1, // ReliableOrdered
    2, // ReliableUnordered
];
