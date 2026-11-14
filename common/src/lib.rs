pub mod network;
pub mod components;
pub mod systems;
pub mod utils;

// TODO: make anyhow
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const LATENCY: u128 = 100;
pub const TICK_RATE: f32 = 60.0;
