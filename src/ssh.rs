use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use crate::string_like::StringOrUint;
use crate::{Error, Result};

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
            let authorized_key = fs::read_to_string(public(&key)).expect("pub key should exists");
            authorized_keys.push(authorized_key.trim_end().to_string());
        }

        authorized_keys
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigSSH {
    pub authorized_keys: Option<Vec<String>>,
    pub key: Option<String>,
    pub options: Option<Vec<String>>,
    pub port: Option<StringOrUint>,
    pub port_user_network: Option<StringOrUint>,
    pub user: Option<String>,
}

impl ConfigSSH {
    pub fn updated(&self, other: &Self) -> ConfigSSH {
        ConfigSSH {
            authorized_keys: self
                .authorized_keys
                .as_ref()
                .or_else(|| other.authorized_keys.as_ref())
                .cloned(),
            key: self.key.as_ref().or_else(|| other.key.as_ref()).cloned(),
            options: self.options.as_ref().or_else(|| other.options.as_ref()).cloned(),
            port: self.port.as_ref().or_else(|| other.port.as_ref()).cloned(),
            port_user_network: self
                .port_user_network
                .as_ref()
                .or_else(|| other.port_user_network.as_ref())
                .cloned(),
            user: self.user.as_ref().or_else(|| other.user.as_ref()).cloned(),
        }
    }
}

impl Default for ConfigSSH {
    fn default() -> ConfigSSH {
        ConfigSSH {
            authorized_keys: None,
            key: None,
            options: None,
            port: None,
            port_user_network: None,
            user: None,
        }
    }
}

fn public(pvt_key: &str) -> String {
    format!("{}.pub", pvt_key)
}

#[derive(Clone, Debug)]
pub struct SSH {
    host: String,
    authorized_keys: Vec<String>,
    key: Option<String>,
    options: Vec<String>,
    port: String,
    user: Option<String>,
}

impl SSH {
    pub fn new(config: &ConfigSSH, address: &Option<String>, user_network: bool) -> Option<SSH> {
        let host = if let Some(address) = address {
            address.to_string()
        } else if user_network {
            "localhost".to_string()
        } else {
            return None;
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

        let port =
            if user_network { config.port_user_network.as_ref() } else { config.port.as_ref() };

        let port = if let Some(port) = port { port.to_string() } else { return None };

        let options =
            if let Some(options) = &config.options { options.to_owned() } else { Vec::new() };
        let user = config.user.to_owned();

        Some(SSH { host, authorized_keys, key, options, port, user })
    }

    pub fn has_key(&self) -> bool {
        self.key.is_some()
    }

    pub fn ensure_keys(&self, work_dir: &PathBuf) -> Result<Keys> {
        let mut ensured_key = self.key.to_owned();
        if let Some(key) = &self.key {
            if key.to_lowercase() == "create" {
                ensured_key = Some(generate_key(work_dir)?);
            } else {
                let pvt_key = key;
                let pub_key = public(pvt_key);
                if !Path::new(&pub_key).exists() {
                    return Err(Error::SSHPublicKeyDoesNotExists(pub_key));
                };
                if !Path::new(pvt_key).exists() {
                    return Err(Error::SSHPrivateKeyDoesNotExists(pub_key));
                };
            }
        }

        Ok(Keys { key: ensured_key, authorized_keys: self.authorized_keys.to_owned() })
    }

    pub fn user_host(&self, user: &Option<&str>) -> String {
        if let Some(user) = user {
            format!("{}@{}", user, self.host)
        } else if let Some(user) = &self.user {
            format!("{}@{}", user, self.host)
        } else {
            self.host.to_owned()
        }
    }

    pub fn options(&self) -> Vec<&str> {
        let mut options = Vec::with_capacity(self.options.len() * 2);
        for option in &self.options {
            options.push("-o");
            options.push(&option);
        }
        options
    }

    pub fn port(&self) -> &str {
        &self.port
    }
}

fn generate_key(work_dir: &PathBuf) -> Result<String> {
    let key_type = "ed25519";
    let pvt_key = work_dir.join(key_type);
    let pub_key = work_dir.join(public(&key_type));
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
        ssh_keygen.spawn().map_err(|e| Error::executable("ssh_keygen", &e.to_string()))?.wait()?;
    }

    Ok(pvt_key.to_string_lossy().to_string())
}
