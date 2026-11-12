pub mod network;
pub mod components;
pub mod utils;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>; // TODO: make anyhow
