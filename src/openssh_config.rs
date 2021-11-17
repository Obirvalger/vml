use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

use crate::VM;

fn str_to_options<S: AsRef<str>>(s: S) -> Vec<String> {
    s.as_ref().replace("-o ", "").split(' ').skip(1).map(|s| s.replace('=', " ")).collect()
}

fn encode<S: AsRef<str>>(data: S) -> String {
    urlencoding::encode(data.as_ref()).into_owned()
}

pub fn add(configs_dir: &Path, vm: &VM) -> Result<()> {
    fs::create_dir_all(&configs_dir)?;
    let name = &vm.name;
    let config_path = configs_dir.join(encode(name));

    let mut config = File::create(&config_path).with_context(|| {
        format!("could not create vm openssh config `{}`", &config_path.display())
    })?;
    writeln!(config, "Host {}", &vm.name)?;

    let info = vm.info();

    if let Some(host) = info.get("ssh_host") {
        writeln!(config, "    Hostname {}", host)?;
    }
    if let Some(port) = info.get("ssh_port") {
        writeln!(config, "    Port {}", port)?;
    }
    if let Some(user) = info.get("ssh_user") {
        writeln!(config, "    User {}", user)?;
    }
    if let Some(key) = info.get("ssh_key") {
        writeln!(config, "    IdentityFile {}", key)?;
    }
    if let Some(options) = info.get("ssh_options") {
        for option in str_to_options(options) {
            writeln!(config, "    {}", option)?;
        }
    }

    Ok(())
}

pub fn rm(configs_dir: &Path, name: &str) -> Result<()> {
    let config_path = configs_dir.join(encode(name));
    if config_path.exists() {
        fs::remove_file(&config_path)?;
    }

    Ok(())
}
