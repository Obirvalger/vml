use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::{Error, Result};

const IMAGES_FILE: &str = "images.toml";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Image {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(transparent)]
struct Images {
    pub images: BTreeMap<String, Image>,
}

fn parse(images_file_path: &PathBuf) -> Result<Images> {
    let images_str = &fs::read_to_string(images_file_path)?;
    let images = toml::from_str(images_str).map_err(|e| {
        Error::parse_images_file(&images_file_path.to_string_lossy(), &e.to_string())
    })?;

    Ok(images)
}

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

pub fn available(images_dir: &PathBuf) -> Result<Vec<String>> {
    let images = parse(&images_dir.join(IMAGES_FILE))?.images;
    let images = images.keys().map(|s| s.to_string()).collect();

    Ok(images)
}

pub fn pull(images_dir: &PathBuf, image_name: &str) -> Result<()> {
    let images = parse(&images_dir.join(IMAGES_FILE))?.images;

    if let Some(image) = images.get(image_name) {
        let mut body =
            reqwest::blocking::get(&image.url).map_err(|e| Error::DownloadImage(e.to_string()))?;
        let image_path = images_dir.join(image_name);
        let mut tmp = tempfile::Builder::new().tempfile_in(images_dir)?;

        println!("Downloading image {} {}", image_name, image.url);
        body.copy_to(&mut tmp).map_err(|e| Error::DownloadImage(e.to_string()))?;

        fs::rename(tmp.path(), image_path)?;
    }

    Ok(())
}
