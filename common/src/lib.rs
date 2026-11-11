pub mod network;
pub mod components;

// temporary
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
