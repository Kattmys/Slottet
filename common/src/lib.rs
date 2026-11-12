pub mod network;
pub mod components;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>; // TODO: make anyhow
