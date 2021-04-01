use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use byte_unit::Byte;
use serde::{Deserialize, Serialize};

use crate::net::ConfigNet;
use crate::ssh::ConfigSSH;
use crate::string_like::StringOrUint;
use crate::{Error, Result};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct VMConfig {
    pub cloud_init: Option<bool>,
    pub cloud_init_image: Option<PathBuf>,
    pub data: Option<HashMap<String, String>>,
    pub disk: Option<PathBuf>,
    pub display: Option<String>,
    pub name: Option<String>,
    pub memory: Option<String>,
    pub minimum_disk_size: Option<Byte>,
    pub net: Option<ConfigNet>,
    pub nproc: Option<StringOrUint>,
    pub ssh: Option<ConfigSSH>,
    pub tags: Option<HashSet<String>>,
}

impl VMConfig {
    pub fn from_config_str(config_str: &str) -> Result<VMConfig> {
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
        net = "{{net}}"
        "#
        .to_owned()
    }

    // if self value is None set it to others
    pub fn update(&mut self, other: &Self) {
        let VMConfig {
            ref mut cloud_init,
            ref mut cloud_init_image,
            ref mut data,
            ref mut disk,
            ref mut display,
            ref mut name,
            ref mut memory,
            ref mut minimum_disk_size,
            ref mut net,
            ref mut nproc,
            ref mut ssh,
            ref mut tags,
        } = self;

        if cloud_init.is_none() {
            *cloud_init = other.cloud_init.to_owned();
        }
        if cloud_init_image.is_none() {
            *cloud_init_image = other.cloud_init_image.to_owned();
        }
        if data.is_none() {
            *data = other.data.to_owned();
        }
        if disk.is_none() {
            *disk = other.disk.to_owned();
        }
        if display.is_none() {
            *display = other.display.to_owned();
        }
        if name.is_none() {
            *name = other.name.to_owned();
        }
        if memory.is_none() {
            *memory = other.memory.to_owned();
        }
        if minimum_disk_size.is_none() {
            *minimum_disk_size = other.minimum_disk_size.to_owned();
        }
        match net {
            None => *net = other.net.to_owned(),
            Some(net) => {
                if let Some(other_net) = &other.net {
                    *net = other_net.updated(net)
                }
            }
        }
        if nproc.is_none() {
            *nproc = other.nproc.to_owned();
        }
        match ssh {
            None => *ssh = other.ssh.to_owned(),
            Some(ssh) => {
                if let Some(other_ssh) = &other.ssh {
                    *ssh = other_ssh.updated(ssh)
                }
            }
        }
        if tags.is_none() {
            *tags = other.tags.to_owned();
        }
    }
}
