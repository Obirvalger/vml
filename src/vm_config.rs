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
    pub image_name: Option<String>,
    #[serde(alias = "mem")]
    pub memory: Option<String>,
    pub minimum_disk_size: Option<Byte>,
    pub nic_model: Option<String>,
    pub nproc: Option<StringOrUint>,
    pub properties: Option<BTreeSet<String>>,
    pub qemu_arch_options: Option<Vec<String>>,
    pub qemu_binary: Option<String>,
    pub qemu_bios: Option<String>,
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
            ref mut image_name,
            ref mut memory,
            ref mut minimum_disk_size,
            ref mut net,
            ref mut nic_model,
            ref mut nproc,
            ref mut properties,
            ref mut qemu_binary,
            ref mut qemu_arch_options,
            ref mut qemu_bios,
            ref mut ssh,
            ref mut tags,
        } = self;

        if cloud_init.is_none() {
            other.cloud_init.clone_into(cloud_init);
        }
        if cloud_init_image.is_none() {
            other.cloud_init_image.clone_into(cloud_init_image);
        }
        if cpu_model.is_none() {
            other.cpu_model.clone_into(cpu_model);
        }
        if data.is_none() {
            other.data.clone_into(data);
        }
        if disk.is_none() {
            other.disk.clone_into(disk);
        }
        if display.is_none() {
            other.display.clone_into(display);
        }
        if gui.is_none() {
            other.gui.clone_into(gui);
        }
        if name.is_none() {
            other.name.clone_into(name);
        }
        if image_name.is_none() {
            other.image_name.clone_into(image_name);
        }
        if memory.is_none() {
            other.memory.clone_into(memory);
        }
        if minimum_disk_size.is_none() {
            other.minimum_disk_size.clone_into(minimum_disk_size);
        }
        match net {
            None => other.net.clone_into(net),
            Some(net) => {
                if let Some(other_net) = &other.net {
                    *net = other_net.updated(net)
                }
            }
        }
        if nic_model.is_none() {
            other.nic_model.clone_into(nic_model);
        }
        if nproc.is_none() {
            other.nproc.clone_into(nproc);
        }
        if properties.is_none() {
            other.properties.clone_into(properties);
        }
        if qemu_binary.is_none() {
            other.qemu_binary.clone_into(qemu_binary);
        }
        if qemu_arch_options.is_none() {
            other.qemu_arch_options.clone_into(qemu_arch_options);
        }
        if qemu_bios.is_none() {
            other.qemu_bios.clone_into(qemu_bios);
        }
        match ssh {
            None => other.ssh.clone_into(ssh),
            Some(ssh) => {
                if let Some(other_ssh) = &other.ssh {
                    *ssh = other_ssh.updated(ssh)
                }
            }
        }
        if tags.is_none() {
            other.tags.clone_into(tags);
        }
    }
}
