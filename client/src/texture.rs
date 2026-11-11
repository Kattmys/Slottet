use common::Result;

use macroquad::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct TextureManager {
    map: HashMap<String, Texture2D>,
}

impl TextureManager {
    pub async fn init(path_str: &str) -> Result<Self> {
        let path = Path::new(path_str);

        let mut map = HashMap::new();

        Self::traverse_textures(
            &path,
            path_str,
            &mut map,
        ).await?;

        for key in map.keys() {
            println!("{}", key);
        }

        Ok(Self { map })
    }
    
    pub async fn get(&self, path: &str) -> Option<&Texture2D> {
        return self.map.get(path)
    }

    async fn traverse_textures(
        dir: &Path,
        prefix: &str,
        data: &mut HashMap<String, Texture2D>
    ) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    Box::pin(Self::traverse_textures(&path, prefix, data)).await?;

                } else if let Some(str_path) = path.to_str() {
                    let texture = load_texture(str_path).await?;
                    if let Some(key) = path
                        .strip_prefix(prefix)?
                        .to_str() {
                        data.insert(key.to_string(), texture);
                    }
                }
            }
        }

        Ok(())
    }
}
