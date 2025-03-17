use std::borrow::Cow;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use anyhow::Result;
use cmd_lib::run_cmd;
use file_lock::{FileLock, FileOptions};
use rust_embed::RustEmbed;

use crate::config::Config;
use crate::config_dir;
use crate::Error;

#[derive(RustEmbed)]
#[folder = "files/configs"]
struct AssetConfigs;

#[derive(RustEmbed)]
#[folder = "files/get-url-progs"]
struct AssetGetUrlProgs;

#[derive(RustEmbed)]
#[folder = "files/efis/x86_64"]
struct AssetEfisX86_64;

#[derive(RustEmbed)]
#[folder = "files/efis/aarch64"]
struct AssetEfisAarch64;

#[derive(RustEmbed)]
#[folder = "files/scripts"]
struct AssetScripts;

#[derive(RustEmbed)]
#[folder = "files"]
struct AssetAllFiles;

fn lock_write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    let options = FileOptions::new().create(true).truncate(true).write(true);
    let block = true;
    if let Ok(mut filelock) = FileLock::lock(path.as_ref(), block, options) {
        filelock.file.write_all(contents.as_ref())?;
    }

    Ok(())
}

pub fn get_config<S: AsRef<str>>(path: S) -> Result<Cow<'static, [u8]>> {
    AssetConfigs::get(path.as_ref())
        .map(|f| f.data)
        .ok_or_else(|| Error::GetWrongEmbeddedFile(path.as_ref().to_string()).into())
}

fn install_executables_in_config_dir<E: RustEmbed, S: AsRef<str>>(
    _assert: &E,
    files: S,
) -> Result<()> {
    let directory = config_dir().join(files.as_ref());
    fs::create_dir_all(&directory)?;

    for filename in E::iter() {
        let filename = filename.as_ref();
        let filepath = directory.join(filename);
        let content = E::get(filename).unwrap();
        lock_write(&filepath, content.data)?;
        run_cmd!(chmod +x $filepath)?
    }

    Ok(())
}

fn install_get_url_progs() -> Result<()> {
    install_executables_in_config_dir(&AssetGetUrlProgs, "get-url-progs")
}

fn install_scripts() -> Result<()> {
    install_executables_in_config_dir(&AssetScripts, "scripts")
}

fn install_files_in_config_dir<E: RustEmbed, S: AsRef<str>>(_assert: &E, files: S) -> Result<()> {
    let directory = config_dir().join(files.as_ref());
    fs::create_dir_all(&directory)?;

    for filename in E::iter() {
        let filename = filename.as_ref();
        let filepath = directory.join(filename);
        let content = E::get(filename).unwrap();
        lock_write(&filepath, content.data)?;
    }

    Ok(())
}

fn install_efis() -> Result<()> {
    install_files_in_config_dir(&AssetEfisX86_64, "efis/x86_64")?;
    install_files_in_config_dir(&AssetEfisAarch64, "efis/aarch64")?;
    Ok(())
}

fn install_config(filename: &str) -> Result<()> {
    let directory = config_dir();
    fs::create_dir_all(&directory)?;

    let config = &directory.join(filename);
    if !config.exists() {
        let etc_config = Path::new("/etc/vml").join(filename);
        if etc_config.exists() {
            fs::copy(etc_config, config)?;
        } else {
            let content = AssetConfigs::get(filename).unwrap();
            lock_write(config, content.data)?;
        }
    }

    Ok(())
}

pub fn install_main_config() -> Result<()> {
    install_config("config.toml")?;

    Ok(())
}

fn install_openssh_config(config: &Config) -> Result<()> {
    let main_config = &config.openssh_config.main_config;
    if let Some(dir) = main_config.parent() {
        fs::create_dir_all(dir)?;
    }
    lock_write(
        main_config,
        format!("Include {}/*", &config.openssh_config.vm_configs_dir.display()),
    )?;

    Ok(())
}

fn is_new_build() -> Result<bool> {
    let last = config_dir().join(".last");
    let exe = std::env::current_exe()?;

    if last.exists() {
        let last_time = fs::metadata(&last)?.modified()?;
        let exe_time = fs::metadata(&exe)?.modified()?;
        Ok(last_time.cmp(&exe_time).is_lt())
    } else {
        Ok(true)
    }
}

fn install_other_files() -> Result<()> {
    if is_new_build()? {
        install_config("images.toml")?;
        install_get_url_progs()?;
        install_scripts()?;
        install_efis()?;

        let last = config_dir().join(".last");
        fs::write(last, "")?;
    }

    Ok(())
}

pub fn install_all(config: &Config) -> Result<()> {
    if !config.vms_dir.exists() {
        fs::create_dir_all(&config.vms_dir)?;
    }
    if !config.images.directory.exists() {
        fs::create_dir_all(&config.images.directory)?;
    }

    install_openssh_config(config)?;

    install_other_files()?;

    Ok(())
}

fn get_file<S: AsRef<str>>(path: S) -> Result<Cow<'static, [u8]>> {
    AssetAllFiles::get(path.as_ref())
        .map(|f| f.data)
        .ok_or_else(|| Error::GetWrongEmbeddedFile(path.as_ref().to_string()).into())
}

pub fn show_file<S: AsRef<str>>(path: S) -> Result<()> {
    io::stdout().write_all(&get_file(path)?)?;
    Ok(())
}
