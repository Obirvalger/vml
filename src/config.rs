use std::env;
use std::fs;
use std::path::PathBuf;

use byte_unit::Byte;
use serde::Deserialize;

use crate::string_like::StringOrUint;
use crate::{Error, Result};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct VMsDefault {
    pub memory: String,
    pub display: Option<String>,
    pub nproc: StringOrUint,
    pub ssh_options: Option<Vec<String>>,
    pub ssh_port_user_network: Option<StringOrUint>,
    pub ssh_port: Option<StringOrUint>,
    pub ssh_user: Option<String>,
    pub minimum_disk_size: Option<Byte>,
    pub cloud_init_image: Option<PathBuf>,
    pub user_network: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub vms_dir: PathBuf,
    pub list_fold: bool,
    pub default: VMsDefault,
}

impl Config {
    pub fn new() -> Result<Config> {
        let config_dir = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            let home = env::var("HOME").expect("HOME variable unset");
            format!("{}/.config", home)
        });

        let config_path = &format!("{}/vml/config.toml", config_dir);
        let config_str = &fs::read_to_string(config_path).map_err(|e| {
            Error::ParseConfig(format!("unable to read config `{}`: {}", config_path, e))
        })?;

        // NOTE realpath of vms_dir should be used in the config, due to
        // std::fs::canonicalize does not resolve all symbolic links
        let config = toml::from_str(config_str).map_err(|e| Error::ParseConfig(e.to_string()))?;

        Ok(config)
    }
}
