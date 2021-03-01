use std::fs;
use std::path::PathBuf;

use crate::Result;

const IMAGES_FILE: &str = "images.toml";

pub fn list(images_dir: &PathBuf) -> Result<Vec<String>> {
    let mut images = Vec::new();

    for path in fs::read_dir(images_dir)? {
        let name = path.unwrap().file_name().to_string_lossy().to_string();
        if name != IMAGES_FILE {
            images.push(name);
        }
    }

    Ok(images)
}
