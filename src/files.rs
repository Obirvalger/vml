use std::fs;
use std::path::Path;

use rust_embed::RustEmbed;

use crate::config::Config;
use crate::config_dir;
use crate::Result;

#[derive(RustEmbed)]
#[folder = "files/"]
struct Asset;

fn install_config(filename: &str) -> Result<()> {
    let directory = config_dir();
    fs::create_dir_all(&directory)?;

    let config = &directory.join(filename);
    if !config.exists() {
        let etc_config = Path::new("/etc/vml").join(filename);
        if etc_config.exists() {
            fs::copy(etc_config, config)?;
        } else {
            let content = Asset::get(filename).unwrap();
            fs::write(&config, content)?;
        }
    }

    Ok(())
}

pub fn install_main_config() -> Result<()> {
    install_config("config.toml")?;

    Ok(())
}

pub fn install_all(config: &Config) -> Result<()> {
    if !config.vms_dir.exists() {
        fs::create_dir_all(&config.vms_dir)?;
    }
    if !config.images.directory.exists() {
        fs::create_dir_all(&config.images.directory)?;
    }
    install_config("images.toml")?;

    Ok(())
}
