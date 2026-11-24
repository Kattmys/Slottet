pub mod network;
pub mod components;
pub mod systems;
pub mod utils;
pub mod map;

// TODO: make anyhow
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const LATENCY: u128 = 10;
pub const TICK_RATE: u64 = 60;
