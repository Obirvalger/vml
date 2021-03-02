use std::fs;
use std::path::PathBuf;

use rust_embed::RustEmbed;

use crate::config::Config;
use crate::Result;

#[derive(RustEmbed)]
#[folder = "files/"]
struct Asset;

fn install(filename: &str, directory: &str) -> Result<()> {
    let directory = PathBuf::from(shellexpand::tilde(directory).to_string());
    fs::create_dir_all(&directory)?;

    let file = &directory.join(filename);
    let content = Asset::get(filename).unwrap();
    if !file.exists() {
        fs::write(&file, content)?;
    }

    Ok(())
}

pub fn install_config() -> Result<()> {
    install("config.toml", "~/.config/vml")?;

    Ok(())
}

pub fn install_all(config: &Config) -> Result<()> {
    if !config.vms_dir.exists() {
        fs::create_dir_all(&config.vms_dir)?;
    }
    install("images.toml", &config.images.directory.to_string_lossy())?;

    Ok(())
}
