use std::env;
use std::fs;
use std::path::PathBuf;

use byte_unit::Byte;
use serde::Deserialize;

use crate::net::ConfigNet;
use crate::ssh::ConfigSSH;
use crate::string_like::StringOrUint;
use crate::{Error, Result};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct VMsDefault {
    pub memory: String,
    pub display: Option<String>,
    #[serde(default)]
    pub net: ConfigNet,
    pub nproc: StringOrUint,
    #[serde(default)]
    pub ssh: ConfigSSH,
    pub minimum_disk_size: Option<Byte>,
    pub cloud_init_image: Option<PathBuf>,
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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct WaitSSH {
    #[serde(default = "default_wait_ssh_attempts")]
    pub attempts: u64,
    #[serde(default = "default_wait_ssh_repeat")]
    pub repeat: u64,
    #[serde(default = "default_wait_ssh_sleep")]
    pub sleep: u64,
    #[serde(default = "default_wait_ssh_timeout")]
    pub timeout: u64,
}

impl Default for WaitSSH {
    fn default() -> WaitSSH {
        WaitSSH {
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
    pub cloud_init: bool,
    #[serde(default)]
    pub wait_ssh: WaitSSH,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct Commands {
    pub create: CreateCommand,
    pub list: ListCommand,
    pub start: StrartCommand,
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
    pub commands: Commands,
    pub default: VMsDefault,
    pub images: Images,
    pub nameservers: Option<Vec<String>>,
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
