use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config_dir;
use crate::{Error, Result};

#[derive(Clone, Debug)]
pub struct Image {
    pub description: Option<String>,
    pub name: String,
    pub url: String,
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Image {}

impl PartialOrd for Image {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Image {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Clone, Debug)]
pub struct Images {
    pub images: BTreeSet<Image>,
}

impl Images {
    pub fn names(&self) -> BTreeSet<String> {
        self.images.iter().map(|i| i.name.to_string()).collect()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct DeserializeImage {
    pub description: Option<String>,
    pub url: String,
    #[serde(default)]
    change: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(transparent)]
struct DeserializeImages {
    pub images: BTreeMap<String, DeserializeImage>,
}

pub fn update_images_file(embedded_iamges_toml: Cow<'static, [u8]>) -> Result<()> {
    let mut embedded_images = toml::from_slice::<DeserializeImages>(&embedded_iamges_toml)
        .expect("Bad embedded images.toml")
        .images
        .into_iter();
    let images_str = &fs::read_to_string(images_file_path())?;
    let mut config_images = toml::from_str::<DeserializeImages>(images_str)
        .map_err(|e| {
            Error::parse_images_file(&images_file_path().to_string_lossy(), &e.to_string())
        })?
        .images
        .into_iter();

    let mut embedded_image = embedded_images.next();
    let mut config_image = config_images.next();
    let mut images: BTreeMap<String, DeserializeImage> = BTreeMap::new();

    while let (Some(ei), Some(ci)) = (&embedded_image, &config_image) {
        // .0 gets name of the image
        // .1 gets other information of the image
        match ei.0.cmp(&ci.0) {
            Ordering::Greater => {
                if !ci.1.change.contains(&"delete".to_string()) {
                    images.insert(ci.0.to_owned(), ci.1.to_owned());
                };
                config_image = config_images.next();
            }
            Ordering::Less => {
                images.insert(ei.0.to_owned(), ei.1.to_owned());
                embedded_image = embedded_images.next();
            }
            Ordering::Equal => {
                if ci.1.change.contains(&"update".to_string()) {
                    images.insert(ei.0.to_owned(), ei.1.to_owned());
                } else {
                    images.insert(ci.0.to_owned(), ci.1.to_owned());
                }
                embedded_image = embedded_images.next();
                config_image = config_images.next();
            }
        }
    }

    let images_file = toml::to_string(&images).expect("Bad internal images representation");
    fs::write(images_file_path(), images_file)?;

    Ok(())
}

fn images_file_path() -> PathBuf {
    config_dir().join("images.toml")
}

fn parse(images_file_path: &Path) -> Result<DeserializeImages> {
    let images_str = &fs::read_to_string(images_file_path)?;
    let images = toml::from_str(images_str).map_err(|e| {
        Error::parse_images_file(&images_file_path.to_string_lossy(), &e.to_string())
    })?;

    Ok(images)
}

pub fn path(images_dir: &Path, image_name: &str) -> Result<PathBuf> {
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

pub fn available() -> Result<Images> {
    let images = parse(&images_file_path())?.images;
    let images = images
        .into_iter()
        .map(|(k, v)| Image { name: k, url: v.url, description: v.description })
        .collect();

    Ok(Images { images })
}

pub fn remove(images_dir: &Path, image_name: &str) -> Result<()> {
    let image_path = images_dir.join(image_name);
    fs::remove_file(&image_path)?;
    Ok(())
}

pub fn pull(images_dir: &Path, image_name: &str) -> Result<PathBuf> {
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
