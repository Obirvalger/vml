use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use byte_unit::Byte;
use serde::{Deserialize, Serialize};

use crate::gui::ConfigGui;
use crate::net::ConfigNet;
use crate::ssh::ConfigSsh;
use crate::string_like::StringOrUint;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct VMConfig {
    pub cloud_init: Option<bool>,
    pub cloud_init_image: Option<PathBuf>,
    pub cpu_model: Option<String>,
    pub data: Option<HashMap<String, String>>,
    pub disk: Option<PathBuf>,
    pub display: Option<String>,
    pub name: Option<String>,
    pub memory: Option<String>,
    pub minimum_disk_size: Option<Byte>,
    pub nic_model: Option<String>,
    pub nproc: Option<StringOrUint>,
    pub properties: Option<BTreeSet<String>>,
    pub qemu_arch_options: Option<Vec<String>>,
    pub qemu_binary: Option<String>,
    pub tags: Option<HashSet<String>>,
    // inset table values at the end
    pub gui: Option<ConfigGui>,
    pub ssh: Option<ConfigSsh>,
    pub net: Option<ConfigNet>,
}

impl VMConfig {
    pub fn from_config_str(config_str: &str) -> Result<VMConfig> {
        let config = toml::from_str(config_str)
            .with_context(|| format!("failed to parse vm config from str:\n{}", config_str))?;

        Ok(config)
    }

    pub fn new(config_path: &Path) -> Result<VMConfig> {
        let config_str = &fs::read_to_string(config_path)
            .with_context(|| format!("failed to read config `{}`", config_path.display()))?;
        let config = toml::from_str(config_str)
            .with_context(|| format!("failed to parse vm config `{}`", &config_path.display()))?;

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
            ref mut cpu_model,
            ref mut data,
            ref mut disk,
            ref mut display,
            ref mut gui,
            ref mut name,
            ref mut memory,
            ref mut minimum_disk_size,
            ref mut net,
            ref mut nic_model,
            ref mut nproc,
            ref mut properties,
            ref mut qemu_binary,
            ref mut qemu_arch_options,
            ref mut ssh,
            ref mut tags,
        } = self;

        if cloud_init.is_none() {
            *cloud_init = other.cloud_init.to_owned();
        }
        if cloud_init_image.is_none() {
            *cloud_init_image = other.cloud_init_image.to_owned();
        }
        if cpu_model.is_none() {
            *cpu_model = other.cpu_model.to_owned();
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
        if gui.is_none() {
            *gui = other.gui.to_owned();
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
        if nic_model.is_none() {
            *nic_model = other.nic_model.to_owned();
        }
        if nproc.is_none() {
            *nproc = other.nproc.to_owned();
        }
        if properties.is_none() {
            *properties = other.properties.to_owned();
        }
        if qemu_binary.is_none() {
            *qemu_binary = other.qemu_binary.to_owned();
        }
        if qemu_arch_options.is_none() {
            *qemu_arch_options = other.qemu_arch_options.to_owned();
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
