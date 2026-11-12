use serde::{ Serialize, Deserialize };
use super::Transform;

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct Tree;

#[derive(hecs::Bundle)]
pub struct TreeBundle {
    tree: Tree,
    transform: Transform,
}
