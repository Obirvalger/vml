use std::fs;
use std::path::PathBuf;

use rust_embed::RustEmbed;

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

pub fn install_all() -> Result<()> {
    install("config.toml", "~/.config/vml")?;
    install("images.toml", "~/.local/share/vml/images")?;

    Ok(())
}
