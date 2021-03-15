use std::env;
use std::fs;
use std::path::PathBuf;

use byte_unit::Byte;
use serde::Deserialize;

use crate::ssh::ConfigSSH;
use crate::string_like::StringOrUint;
use crate::{Error, Result};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct VMsDefault {
    pub memory: String,
    pub display: Option<String>,
    pub nproc: StringOrUint,
    #[serde(default)]
    pub ssh: ConfigSSH,
    pub minimum_disk_size: Option<Byte>,
    pub cloud_init_image: Option<PathBuf>,
    pub user_network: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Images {
    pub directory: PathBuf,
    pub other_directories_ro: Vec<PathBuf>,
    pub default: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub vms_dir: PathBuf,
    pub list_fold: bool,
    pub default: VMsDefault,
    pub images: Images,
}

fn expand_tilde(path: &PathBuf) -> PathBuf {
    let s = path.to_string_lossy().to_string();
    PathBuf::from(shellexpand::tilde(&s).to_string())
}

pub fn config_dir() -> PathBuf {
    let home_config_dir = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| "~/.config".to_string());
    expand_tilde(&PathBuf::from(home_config_dir)).join("vml")
}

impl Config {
    pub fn new() -> Result<Config> {
        let config_path = config_dir().join("config.toml");
        let config_str = &fs::read_to_string(&config_path).map_err(|e| {
            Error::ParseConfig(format!("unable to read config `{:?}`: {}", &config_path, &e))
        })?;

        let mut config: Config =
            toml::from_str(config_str).map_err(|e| Error::ParseConfig(e.to_string()))?;
        config.images.directory = expand_tilde(&config.images.directory);
        config.vms_dir = expand_tilde(&config.vms_dir);
        if !config.vms_dir.is_dir() {
            fs::create_dir_all(&config.vms_dir)?;
        } else {
            config.vms_dir = fs::canonicalize(&config.vms_dir)?;
        }
        config.default.cloud_init_image =
            config.default.cloud_init_image.map(|i| expand_tilde(&i));

        Ok(config)
    }
}
