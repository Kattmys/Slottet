use common::Result;

use macroquad::prelude::*;
use std::collections::HashMap;

pub struct TextureManager {
    textures: HashMap<Box<str>, Texture2D>,
    error_texture: Texture2D,
    root: Box<str>,
}

impl TextureManager {
    pub fn init(root: &str) -> Self {
        const UNKNOWN_TEXTURE: &'static [u8] = &[
            0,   0, 0,   255, // BLACK
            255, 0, 255, 255, // PURPLE
            255, 0, 255, 255, // PURPLE
            0,   0, 0,   255, // BLACK
        ];

        let error_texture = Texture2D::from_rgba8(2, 2, UNKNOWN_TEXTURE);
        let textures = HashMap::new();

        Self {
            textures,
            error_texture,
            root: root.into(),
        }
    }

    pub async fn get(&self, id: &str) -> &Texture2D {
        self.textures.get(id).unwrap_or(&self.error_texture)
    }

    pub async fn load_textures(&mut self, id_path_map: &[(&str, &str)]) {
        for (id, path) in id_path_map {
            let result = self.load_texture(id, path).await;
            if let Err(e) = result {
                eprintln!("error when loading texture {id} from {path}: {e}");
            }
        }
    }
    
    pub async fn load_texture(&mut self, id: &str, path: &str) -> Result<()> {
        let path = format!("{}/{}", self.root, path);
        let texture = load_texture(&path).await?;
        println!("loaded {id} from {path}");
        texture.set_filter(macroquad::texture::FilterMode::Nearest);
        self.textures.insert(id.into(), texture);
        Ok(())
    }
}
