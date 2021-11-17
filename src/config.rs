use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use byte_unit::Byte;
use serde::Deserialize;

use crate::net::ConfigNet;
use crate::ssh::ConfigSsh;
use crate::string_like::StringOrUint;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct VMsDefault {
    pub memory: String,
    pub display: Option<String>,
    pub net: ConfigNet,
    #[serde(default = "default_nic_model")]
    pub nic_model: String,
    pub nproc: StringOrUint,
    #[serde(default)]
    pub ssh: ConfigSsh,
    pub minimum_disk_size: Option<Byte>,
    pub cloud_init: bool,
    pub cloud_init_image: Option<PathBuf>,
}

fn default_nic_model() -> String {
    "virtio-net-pci".to_string()
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CreateExistsAction {
    Fail,
    Ignore,
    Replace,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct CreateCommand {
    pub pull: bool,
    pub exists: CreateExistsAction,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct ListCommand {
    pub all: bool,
    pub fold: bool,
}

fn default_remove_interactive() -> bool {
    true
}

fn default_remove_verbose() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct RemoveCommand {
    #[serde(default = "default_remove_interactive")]
    pub interactive: bool,
    #[serde(default = "default_remove_verbose")]
    pub verbose: bool,
}

impl Default for RemoveCommand {
    fn default() -> RemoveCommand {
        RemoveCommand {
            interactive: default_remove_interactive(),
            verbose: default_remove_verbose(),
        }
    }
}

fn default_wait_ssh_timeout() -> u64 {
    1
}

fn default_wait_ssh_repeat() -> u64 {
    1
}

fn default_wait_ssh_sleep() -> u64 {
    60
}

fn default_wait_ssh_attempts() -> u64 {
    60
}

fn default_wait_ssh_on() -> bool {
    false
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct WaitSsh {
    #[serde(default = "default_wait_ssh_on")]
    pub on: bool,
    #[serde(default = "default_wait_ssh_attempts")]
    pub attempts: u64,
    #[serde(default = "default_wait_ssh_repeat")]
    pub repeat: u64,
    #[serde(default = "default_wait_ssh_sleep")]
    pub sleep: u64,
    #[serde(default = "default_wait_ssh_timeout")]
    pub timeout: u64,
}

impl Default for WaitSsh {
    fn default() -> WaitSsh {
        WaitSsh {
            on: default_wait_ssh_on(),
            attempts: default_wait_ssh_attempts(),
            repeat: default_wait_ssh_repeat(),
            sleep: default_wait_ssh_sleep(),
            timeout: default_wait_ssh_timeout(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct StrartCommand {
    #[serde(default)]
    pub wait_ssh: WaitSsh,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Commands {
    pub create: CreateCommand,
    pub list: ListCommand,
    #[serde(default)]
    pub remove: RemoveCommand,
    pub start: StrartCommand,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Images {
    pub directory: PathBuf,
    pub other_directories_ro: Vec<PathBuf>,
    pub default: String,
    #[serde(default)]
    pub update_on_create: bool,
    pub update_after_days: Option<u64>,
}

fn default_openssh_config_main_config() -> PathBuf {
    PathBuf::from("~/.local/share/vml/openssh/main-config")
}

fn default_openssh_config_vm_configs_dir() -> PathBuf {
    PathBuf::from("~/.local/share/vml/openssh/vm-configs")
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct OpensshConfig {
    #[serde(default = "default_openssh_config_main_config")]
    pub main_config: PathBuf,
    #[serde(default = "default_openssh_config_vm_configs_dir")]
    pub vm_configs_dir: PathBuf,
}

impl Default for OpensshConfig {
    fn default() -> OpensshConfig {
        OpensshConfig {
            main_config: default_openssh_config_main_config(),
            vm_configs_dir: default_openssh_config_vm_configs_dir(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub vms_dir: PathBuf,
    pub config_hierarchy: bool,
    pub commands: Commands,
    #[serde(default)]
    pub openssh_config: OpensshConfig,
    pub default: VMsDefault,
    pub images: Images,
    pub nameservers: Option<Vec<String>>,
}

fn expand_tilde(path: &Path) -> PathBuf {
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
        let config_str = &fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read config `{}`", &config_path.display()))?;

        let mut config: Config = toml::from_str(config_str)
            .with_context(|| format!("failed to parse config `{}`", &config_path.display()))?;
        config.images.directory = expand_tilde(&config.images.directory);
        config.vms_dir = expand_tilde(&config.vms_dir);
        if !config.vms_dir.is_dir() {
            fs::create_dir_all(&config.vms_dir)?;
        } else {
            config.vms_dir = fs::canonicalize(&config.vms_dir)?;
        }
        config.openssh_config.main_config = expand_tilde(&config.openssh_config.main_config);
        config.openssh_config.vm_configs_dir = expand_tilde(&config.openssh_config.vm_configs_dir);
        config.default.cloud_init_image =
            config.default.cloud_init_image.map(|i| expand_tilde(&i));

        Ok(config)
    }
}
