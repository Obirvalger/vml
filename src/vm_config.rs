use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use byte_unit::Byte;
use serde::Deserialize;

use crate::string_like::StringOrUint;
use crate::{Error, Result};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct VMConfig {
    pub address: Option<String>,
    pub cloud_init_image: Option<PathBuf>,
    pub data: Option<HashMap<String, String>>,
    pub disk: Option<PathBuf>,
    pub display: Option<String>,
    pub name: Option<String>,
    pub memory: Option<String>,
    pub minimum_disk_size: Option<Byte>,
    pub nproc: Option<StringOrUint>,
    pub ssh_options: Option<Vec<String>>,
    pub ssh_port: Option<StringOrUint>,
    pub ssh_user: Option<String>,
    pub tags: Option<HashSet<String>>,
    pub tap: Option<String>,
    pub user_network: Option<bool>,
}

impl VMConfig {
    pub fn from_str(config_str: &str) -> Result<VMConfig> {
        let config = toml::from_str(config_str)
            .map_err(|e| Error::parse_vm_config("config from str", &e.to_string()))?;

        Ok(config)
    }

    pub fn new(config_path: &PathBuf) -> Result<VMConfig> {
        let config_str = &fs::read_to_string(config_path).map_err(|e| {
            Error::ParseConfig(format!(
                "unable to read config `{}`: {}",
                config_path.to_string_lossy(),
                e
            ))
        })?;

        let config = toml::from_str(config_str)
            .map_err(|e| Error::parse_vm_config(&config_path.to_string_lossy(), &e.to_string()))?;

        Ok(config)
    }

    pub fn minimal_config_string() -> String {
        r#"
        name = "{{name}}"
        disk = "{{disk}}"
        [data]
        address = "{{address}}"
        tap = "{{tap}}"
        "#.to_owned()
    }
}
