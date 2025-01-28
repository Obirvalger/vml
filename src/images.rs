use std::borrow::Cow;
use std::cmp::min;
use std::cmp::Ordering;
use std::collections::btree_map;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::env::consts::ARCH;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};
use cmd_lib::run_fun;
use file_lock::{FileLock, FileOptions};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use infer;
use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Images as ConfigImages;
use crate::config_dir;
use crate::files;
use crate::template;
use crate::Error;

#[derive(Clone, Debug)]
pub struct Image<'a> {
    pub description: Option<String>,
    pub name: String,
    pub properties: BTreeSet<String>,
    url: String,
    get_url_prog: Option<PathBuf>,
    config: &'a ConfigImages,
    update_after_days: Option<u64>,
}

impl<'a> Image<'a> {
    fn from_deserialize(
        image: DeserializeImage,
        name: impl AsRef<str>,
        config: &'a ConfigImages,
    ) -> Self {
        let mut arch = ARCH.to_string();
        if let Some(arch_mapping) = &image.arch_mapping {
            if let Some(mapped_arch) = arch_mapping.get(&arch) {
                arch = mapped_arch.to_string();
            }
        }

        let mut url = image.url;
        let context = template::create_context(&[("arch".to_string(), arch)]);
        if let Ok(rendered_url) = template::render(&context, &url, "read image url") {
            url = rendered_url;
        }

        Image {
            name: name.as_ref().to_string(),
            url,
            get_url_prog: image.get_url_prog,
            description: image.description,
            config,
            properties: image.properties,
            update_after_days: image.update_after_days,
        }
    }

    fn path(&self) -> PathBuf {
        self.config.directory.join(&self.name)
    }

    fn outdate_option(&self) -> Option<bool> {
        let default_update_after_days = self.config.update_after_days;
        let image_path = self.path();
        let modified_time = fs::metadata(image_path).and_then(|m| m.modified()).ok()?;
        let sys_time = SystemTime::now();
        let duration = sys_time.duration_since(modified_time).ok()?;
        let update_after_days = self.update_after_days.or(default_update_after_days);
        let update_after = Duration::from_secs(update_after_days? * 60 * 60 * 24);

        Some(duration > update_after)
    }

    pub fn outdate(&self) -> bool {
        self.outdate_option().unwrap_or(false)
    }

    pub fn exists(&self) -> bool {
        self.path().is_file()
    }

    fn url(&self) -> String {
        let mut url = self.url.to_string();
        let name = &self.name;

        if let Some(get_url_prog) = &self.get_url_prog {
            let prog = if get_url_prog.is_absolute() {
                get_url_prog.to_owned()
            } else {
                config_dir().join("get-url-progs").join(get_url_prog)
            };

            if let Ok(output) = run_fun!($prog $name) {
                if !output.is_empty() {
                    url = output
                }
            }
        };

        url
    }

    pub async fn pull(&self, show_pb: bool) -> Result<PathBuf> {
        let url = &self.url();
        let image_path = self.path();
        let images_dir = &self.config.directory;
        let mut tmp = tempfile::Builder::new().tempfile_in(images_dir)?;

        info!("Downloading image {} {}", &self.name, url);
        download_file(url, &mut tmp, show_pb).await.unwrap();

        if let Ok(Some(mtype)) = infer::get_from_path(tmp.path()) {
            let mime_type = mtype.mime_type();
            if mime_type == "text/html" {
                bail!(Error::PullHtmlImage)
            }
            if mime_type != "application/x-qemu-disk" {
                bail!(Error::PullUsupportedTypeImage(mime_type.to_string()))
            }
        } else {
            bail!(Error::PullUnknownTypeImage)
        }

        fs::rename(tmp.path(), &image_path)?;

        Ok(image_path)
    }
}

impl PartialEq for Image<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Image<'_> {}

impl PartialOrd for Image<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Image<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Images<'a>(BTreeMap<String, Image<'a>>);

impl Images<'_> {
    pub fn exists(self) -> Self {
        let images = self.0.into_iter().filter(|(_, i)| i.exists()).collect();

        Images(images)
    }

    pub fn outdate(self) -> Self {
        let images = self.0.into_iter().filter(|(_, i)| i.outdate()).collect();

        Images(images)
    }

    pub fn filter(self, predicate: impl Fn(&Image) -> bool) -> Self {
        let images = self.0.into_iter().filter(|(_, i)| predicate(i)).collect();

        Images(images)
    }

    pub fn names(&self) -> BTreeSet<String> {
        self.0.keys().map(|name| name.to_string()).collect()
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<&Image> {
        self.0.get(name.as_ref())
    }

    pub fn get_result(&self, name: impl AsRef<str>) -> Result<&Image> {
        self.0
            .get(name.as_ref())
            .ok_or_else(|| Error::UnknownImage(name.as_ref().to_string()).into())
    }
}

impl<'a> IntoIterator for Images<'a> {
    type Item = Image<'a>;
    type IntoIter = ImagesIntoIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ImagesIntoIter(self.0.into_iter())
    }
}

pub struct ImagesIntoIter<'a>(btree_map::IntoIter<String, Image<'a>>);

impl<'a> Iterator for ImagesIntoIter<'a> {
    type Item = Image<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, i)| i)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
struct DeserializeImage {
    pub description: Option<String>,
    pub url: String,
    pub get_url_prog: Option<PathBuf>,
    #[serde(default)]
    change: Vec<String>,
    #[serde(default)]
    properties: BTreeSet<String>,
    update_after_days: Option<u64>,
    arch_mapping: Option<BTreeMap<String, String>>,
}

impl DeserializeImage {
    fn from_builder(builder: ImageBuilder) -> Self {
        DeserializeImage {
            description: builder.fields.description,
            url: builder.url,
            get_url_prog: None,
            change: builder.fields.change,
            properties: builder.fields.properties,
            update_after_days: builder.fields.update_after_days,
            arch_mapping: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(transparent)]
struct DeserializeImages(BTreeMap<String, DeserializeImage>);

#[derive(Clone, Debug, Default)]
struct ImageBuilderFields {
    description: Option<String>,
    change: Vec<String>,
    properties: BTreeSet<String>,
    update_after_days: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct ImageBuilder {
    name: String,
    url: String,
    fields: ImageBuilderFields,
}

impl ImageBuilder {
    pub fn new<N: AsRef<str>, U: AsRef<str>>(name: N, url: U) -> Self {
        ImageBuilder {
            name: name.as_ref().to_string(),
            url: url.as_ref().to_string(),
            fields: ImageBuilderFields::default(),
        }
    }

    pub fn description<S: AsRef<str>>(&mut self, description: S) -> &Self {
        self.fields.description = Some(description.as_ref().to_string());
        self
    }

    pub fn change(&mut self, change: &[String]) -> &Self {
        self.fields.change = Vec::from(change);
        self
    }

    pub fn properties(&mut self, properties: &[String]) -> &Self {
        self.fields.properties =
            properties.iter().cloned().map(String::from).collect::<BTreeSet<_>>();
        self
    }

    pub fn update_after_days(&mut self, update_after_days: u64) -> &Self {
        self.fields.update_after_days = Some(update_after_days);
        self
    }
}

fn update_images(
    embedded_images: &mut btree_map::IntoIter<String, DeserializeImage>,
    config_images: &mut btree_map::IntoIter<String, DeserializeImage>,
) -> BTreeMap<String, DeserializeImage> {
    let mut embedded_image = embedded_images.next();
    let mut config_image = config_images.next();
    let mut images: BTreeMap<String, DeserializeImage> = BTreeMap::new();

    loop {
        match (&embedded_image, &config_image) {
            (Some(ei), Some(ci)) => {
                let old_name = &ci.0;
                let new_name = &ei.0;
                let old = &ci.1;
                let new = &ei.1;
                let change_set: HashSet<&str> = old.change.iter().map(AsRef::as_ref).collect();
                match new_name.cmp(old_name) {
                    Ordering::Greater => {
                        if !change_set.contains("delete") {
                            images.insert(old_name.to_owned(), old.to_owned());
                        };
                        config_image = config_images.next();
                    }
                    Ordering::Less => {
                        images.insert(new_name.to_owned(), new.to_owned());
                        embedded_image = embedded_images.next();
                    }
                    Ordering::Equal => {
                        let update_all = change_set.contains("update-all");
                        let url = if change_set.contains("keep-url")
                            || !update_all && !change_set.contains("update-url")
                        {
                            old.url.to_owned()
                        } else {
                            new.url.to_owned()
                        };
                        let get_url_prog = if change_set.contains("keep-get-url-prog")
                            || !update_all && !change_set.contains("update-get-url-prog")
                        {
                            old.get_url_prog.to_owned()
                        } else {
                            new.get_url_prog.to_owned()
                        };
                        let description = if change_set.contains("keep-description")
                            || !update_all && !change_set.contains("update-description")
                        {
                            old.description.to_owned()
                        } else {
                            new.description.to_owned()
                        };
                        let change = if change_set.contains("keep-change")
                            || !update_all && !change_set.contains("update-change")
                        {
                            old.change.to_owned()
                        } else {
                            new.change.to_owned()
                        };
                        let properties = if change_set.contains("keep-properties")
                            || !update_all && !change_set.contains("update-properties")
                        {
                            old.properties.to_owned()
                        } else {
                            new.properties.to_owned()
                        };
                        let update_after_days = if change_set.contains("keep-update-after-days")
                            || !update_all && !change_set.contains("update-update-after-days")
                        {
                            old.update_after_days.to_owned()
                        } else {
                            new.update_after_days.to_owned()
                        };
                        let arch_mapping = if change_set.contains("keep-arch-mapping")
                            || !update_all && !change_set.contains("update-arch-mapping")
                        {
                            old.arch_mapping.to_owned()
                        } else {
                            new.arch_mapping.to_owned()
                        };
                        images.insert(
                            old_name.to_owned(),
                            DeserializeImage {
                                url,
                                get_url_prog,
                                description,
                                change,
                                properties,
                                update_after_days,
                                arch_mapping,
                            },
                        );
                        embedded_image = embedded_images.next();
                        config_image = config_images.next();
                    }
                }
            }
            (Some(ei), None) => {
                let new_name = &ei.0;
                let new = &ei.1;
                images.insert(new_name.to_owned(), new.to_owned());
                embedded_image = embedded_images.next();
            }
            (None, Some(ci)) => {
                let old_name = &ci.0;
                let old = &ci.1;
                let change_set: HashSet<&str> = old.change.iter().map(AsRef::as_ref).collect();
                if !change_set.contains("delete") {
                    images.insert(old_name.to_owned(), old.to_owned());
                };
                config_image = config_images.next();
            }
            (&None, &None) => break,
        }
    }

    images
}

pub fn update_images_file(embedded_iamges_toml: Cow<'static, [u8]>) -> Result<()> {
    let embedded_iamges_toml = String::from_utf8_lossy(&embedded_iamges_toml);
    let mut embedded_images = toml::from_str::<DeserializeImages>(&embedded_iamges_toml)
        .expect("Bad embedded images.toml")
        .0
        .into_iter();
    let images_file_path = images_file_path();
    let images_str = &fs::read_to_string(&images_file_path).with_context(|| {
        format!("failed to read images file `{}`", &images_file_path.display())
    })?;
    let mut config_images = toml::from_str::<DeserializeImages>(images_str)
        .with_context(|| format!("failed to parse images file `{}`", &images_file_path.display()))?
        .0
        .into_iter();

    let images = update_images(&mut embedded_images, &mut config_images);

    let header = files::get_config("images-header")?;
    let images_string = toml::to_string(&images).expect("Bad internal images representation");
    let options = FileOptions::new().create(true).truncate(true).write(true);
    let block = true;
    if let Ok(mut images_filelock) = FileLock::lock(&images_file_path, block, options) {
        images_filelock.file.write_all(&header)?;
        images_filelock.file.write_all(images_string.as_bytes())?;
    }

    Ok(())
}

fn images_file_path() -> PathBuf {
    config_dir().join("images.toml")
}

fn parse(images_file_path: &Path) -> Result<DeserializeImages> {
    let images_str = &fs::read_to_string(images_file_path)
        .with_context(|| format!("failed to read images file `{}`", images_file_path.display()))?;
    let images = toml::from_str(images_str).with_context(|| {
        format!("failed to parse images file `{}`", images_file_path.display())
    })?;

    Ok(images)
}

pub fn path(images_dir: &Path, image_name: &str) -> Result<PathBuf> {
    let image_path = images_dir.join(image_name);
    if image_path.is_file() {
        Ok(image_path)
    } else {
        bail!(Error::ImageDoesNotExists(image_name.to_string()))
    }
}

pub fn find(images_dirs: &[&PathBuf], image_name: &str) -> Result<PathBuf> {
    for images_dir in images_dirs {
        let image_path = images_dir.join(image_name);
        if image_path.is_file() {
            return Ok(image_path);
        }
    }

    bail!(Error::ImageDoesNotExists(image_name.to_string()))
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

pub fn available(config: &ConfigImages) -> Result<Images> {
    let images = parse(&images_file_path())?.0;
    let images = images
        .into_iter()
        .map(|(k, v)| (k.to_owned(), Image::from_deserialize(v, &k, config)))
        .collect();

    Ok(Images(images))
}

pub fn add(builder: &ImageBuilder) -> Result<()> {
    let url = &builder.url;
    let name = &builder.name;
    if !url.starts_with("https://") && !url.starts_with("http://") {
        bail!(Error::BadUrl(url.to_string()))
    }

    let image = DeserializeImage::from_builder(builder.to_owned());

    let mut images = parse(&images_file_path())?;
    images.0.insert(name.to_string(), image.to_owned());

    let header = files::get_config("images-header")?;
    let images_string = toml::to_string(&images)?;
    let options = FileOptions::new().create(true).truncate(true).write(true);
    let block = true;
    if let Ok(mut images_filelock) = FileLock::lock(images_file_path(), block, options) {
        images_filelock.file.write_all(&header)?;
        images_filelock.file.write_all(images_string.as_bytes())?;
    }

    Ok(())
}

pub async fn pull(config: &ConfigImages, builder: &ImageBuilder, show_pb: bool) -> Result<()> {
    let name = builder.name.to_string();
    let image = DeserializeImage::from_builder(builder.to_owned());
    let image = Image::from_deserialize(image, name, config);
    image.pull(show_pb).await?;

    Ok(())
}

pub fn remove(images_dir: &Path, image_name: &str) -> Result<()> {
    let image_path = images_dir.join(image_name);
    fs::remove_file(image_path)?;
    Ok(())
}

async fn download_file<F: Write>(url: &str, file: &mut F, show_pb: bool) -> Result<()> {
    let res = Client::new().get(url).send().await?;
    let total_size = res.content_length().unwrap_or(0);

    let pb = if show_pb && total_size != 0 {
        ProgressBar::new(total_size)
    } else {
        ProgressBar::hidden()
    };

    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {wide_bar:.green/.white} {bytes}/{total_bytes} \
             ({bytes_per_sec}, ~{eta})",
        )
        .expect("Bad progress bar style template")
        .progress_chars("\u{2588}\u{2588}\u{2591}"),
    );

    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    Ok(())
}
