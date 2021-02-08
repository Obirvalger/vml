use std::fs;
use std::path::PathBuf;

use crate::Result;

#[derive(Clone, Debug)]
pub struct Cache {
    name: String,
    dir: PathBuf,
}

impl Cache {
    pub fn new(name: &str, dir: &PathBuf) -> Result<Cache> {
        fs::create_dir_all(&dir)?;
        Ok(Cache { name: name.to_string(), dir: dir.to_owned() })
    }

    pub fn store(&self, key: &str, value: &str) -> Result<()> {
        fs::write(self.dir.join(key), value.as_bytes())?;
        Ok(())
    }

    pub fn load(&self, key: &str) -> Result<String> {
        Ok(fs::read_to_string(self.dir.join(key))?)
    }
}
