use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::net::{self, ConfigNet};
use crate::string_like::StringOrUint;
use crate::Error;

pub struct Keys {
    authorized_keys: Vec<String>,
    key: Option<String>,
}

impl Keys {
    pub fn private(&self) -> Option<String> {
        self.key.to_owned()
    }

    pub fn authorized_keys(&self) -> Vec<String> {
        let mut authorized_keys = self.authorized_keys.to_owned();
        if let Some(key) = &self.key {
            let authorized_key = fs::read_to_string(public(key)).expect("pub key should exists");
            authorized_keys.push(authorized_key.trim_end().to_string());
        }

        authorized_keys
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct ConfigSsh {
    pub authorized_keys: Option<Vec<String>>,
    pub key: Option<String>,
    pub options: Option<Vec<String>>,
    pub port: Option<StringOrUint>,
    pub port_user_network: Option<StringOrUint>,
    pub host_user_network: Option<String>,
    pub user: Option<String>,
}

impl ConfigSsh {
    pub fn updated(&self, other: &Self) -> ConfigSsh {
        ConfigSsh {
            authorized_keys: self
                .authorized_keys
                .as_ref()
                .or(other.authorized_keys.as_ref())
                .cloned(),
            key: self.key.as_ref().or(other.key.as_ref()).cloned(),
            options: self.options.as_ref().or(other.options.as_ref()).cloned(),
            port: self.port.as_ref().or(other.port.as_ref()).cloned(),
            port_user_network: self
                .port_user_network
                .as_ref()
                .or(other.port_user_network.as_ref())
                .cloned(),
            host_user_network: self
                .host_user_network
                .as_ref()
                .or(other.host_user_network.as_ref())
                .cloned(),
            user: self.user.as_ref().or(other.user.as_ref()).cloned(),
        }
    }
}

fn public(pvt_key: &str) -> String {
    format!("{}.pub", pvt_key)
}

#[derive(Clone, Debug)]
pub struct Ssh {
    host: String,
    authorized_keys: Vec<String>,
    key: Option<String>,
    options: Vec<String>,
    port: String,
    user: Option<String>,
}

impl Ssh {
    pub fn new(config: &ConfigSsh, config_net: &ConfigNet) -> Option<Ssh> {
        let host = match config_net {
            ConfigNet::Tap { address, .. } => address.as_ref().and_then(net::address)?,
            ConfigNet::User => {
                config.host_user_network.to_owned().unwrap_or_else(|| "127.0.0.1".to_string())
            }
            _ => return None,
        };

        let authorized_keys = if let Some(authorized_keys) = &config.authorized_keys {
            authorized_keys.to_owned()
        } else {
            Vec::new()
        };

        let mut key = config.key.to_owned();
        if let Some(key_str) = &key {
            if key_str.to_lowercase() == "none" {
                key = None;
            }
        }

        let port = if config_net.is_user() {
            config.port_user_network.as_ref()
        } else {
            config.port.as_ref()
        };

        let port = if let Some(port) = port { port.to_string() } else { return None };

        let options =
            if let Some(options) = &config.options { options.to_owned() } else { Vec::new() };
        let user = config.user.to_owned();

        Some(Ssh { host, authorized_keys, key, options, port, user })
    }

    pub fn has_key(&self) -> bool {
        self.key.is_some()
    }

    pub fn ensure_keys(&self, work_dir: &Path) -> Result<Keys> {
        let mut ensured_key = self.key.to_owned();
        if let Some(key) = &self.key {
            if key.to_lowercase() == "create" {
                ensured_key = Some(generate_key(work_dir)?);
            } else {
                let pvt_key = key;
                let pub_key = public(pvt_key);
                if !Path::new(&pub_key).exists() {
                    bail!(Error::SshPublicKeyDoesNotExists(pub_key));
                };
                if !Path::new(pvt_key).exists() {
                    bail!(Error::SshPrivateKeyDoesNotExists(pub_key));
                };
            }
        }

        Ok(Keys { key: ensured_key, authorized_keys: self.authorized_keys.to_owned() })
    }

    pub fn user(&self) -> Option<String> {
        self.user.to_owned()
    }

    pub fn user_host<S: AsRef<str>>(&self, user: &Option<S>) -> String {
        if let Some(user) = user {
            format!("{}@{}", user.as_ref(), self.host)
        } else if let Some(user) = &self.user {
            format!("{}@{}", user, self.host)
        } else {
            self.host.to_owned()
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn options(&self) -> Vec<&str> {
        let mut options = Vec::with_capacity(self.options.len() * 2);
        for option in &self.options {
            options.push("-o");
            options.push(option);
        }
        options
    }

    pub fn port(&self) -> &str {
        &self.port
    }
}

fn generate_key(work_dir: &Path) -> Result<String> {
    let key_type = "ed25519";
    let pvt_key = work_dir.join(key_type);
    let pub_key = work_dir.join(public(key_type));
    let pvt_exists = pvt_key.exists();
    let pub_exists = pub_key.exists();

    if !pvt_exists || !pub_exists {
        if pvt_exists {
            fs::remove_file(&pvt_key)?;
        } else if pub_exists {
            fs::remove_file(&pub_key)?;
        }

        fs::create_dir_all(&work_dir)?;
        let mut ssh_keygen = Command::new("ssh-keygen");
        ssh_keygen.args(&["-q", "-N", "", "-t", "ed25519", "-f"]).arg(&pvt_key);
        ssh_keygen.spawn().context("failed to run executable ssh_keygen")?.wait()?;
    }

    Ok(pvt_key.to_string_lossy().to_string())
}
