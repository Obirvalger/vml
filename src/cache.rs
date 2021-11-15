use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

#[derive(Clone, Debug)]
pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    pub fn new(dir: &Path) -> Result<Cache> {
        fs::create_dir_all(&dir)?;
        Ok(Cache { dir: dir.to_owned() })
    }

    pub fn store(&self, key: &str, value: &str) -> Result<()> {
        fs::write(self.dir.join(key), value.as_bytes())?;
        Ok(())
    }

    pub fn load(&self, key: &str) -> Result<String> {
        Ok(fs::read_to_string(self.dir.join(key))?)
    }
}
