use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::config_dir;
use crate::{Error, Result};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct Image {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(transparent)]
struct Images {
    pub images: BTreeMap<String, Image>,
}

fn images_file_path() -> PathBuf {
    config_dir().join("images.toml")
}

fn parse(images_file_path: &PathBuf) -> Result<Images> {
    let images_str = &fs::read_to_string(images_file_path)?;
    let images = toml::from_str(images_str).map_err(|e| {
        Error::parse_images_file(&images_file_path.to_string_lossy(), &e.to_string())
    })?;

    Ok(images)
}

pub fn path(images_dir: &PathBuf, image_name: &str) -> Result<PathBuf> {
    let image_path = images_dir.join(image_name);
    if image_path.is_file() {
        Ok(image_path)
    } else {
        Err(Error::ImageDoesNotExists(image_name.to_string()))
    }
}

pub fn find(images_dirs: &[&PathBuf], image_name: &str) -> Result<PathBuf> {
    for images_dir in images_dirs {
        let image_path = images_dir.join(image_name);
        if image_path.is_file() {
            return Ok(image_path);
        }
    }

    Err(Error::ImageDoesNotExists(image_name.to_string()))
}

pub fn list(images_dirs: &[&PathBuf]) -> Result<Vec<String>> {
    let mut images = BTreeSet::new();

    for dir in images_dirs {
        for path in fs::read_dir(dir)? {
            let name = path.unwrap().file_name().to_string_lossy().to_string();
            images.insert(name);
        }
    }

    Ok(images.into_iter().collect())
}

pub fn available() -> Result<Vec<String>> {
    let images = parse(&images_file_path())?.images;
    let images = images.keys().map(|s| s.to_string()).collect();

    Ok(images)
}

pub fn pull(images_dir: &PathBuf, image_name: &str) -> Result<PathBuf> {
    let images = parse(&images_file_path())?.images;

    if let Some(image) = images.get(image_name) {
        let mut body =
            reqwest::blocking::get(&image.url).map_err(|e| Error::DownloadImage(e.to_string()))?;
        let image_path = images_dir.join(image_name);
        let mut tmp = tempfile::Builder::new().tempfile_in(images_dir)?;

        println!("Downloading image {} {}", image_name, image.url);
        body.copy_to(&mut tmp).map_err(|e| Error::DownloadImage(e.to_string()))?;

        fs::rename(tmp.path(), &image_path)?;

        Ok(image_path)
    } else {
        Err(Error::UnknownImage(image_name.to_string()))
    }
}
