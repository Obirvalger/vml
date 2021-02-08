use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Cache {
    name: String,
    dir: PathBuf,
}

impl Cache {
    pub fn new(name: &str, dir: &PathBuf) -> Cache {
        fs::create_dir_all(&dir).unwrap();
        Cache { name: name.to_string(), dir: dir.to_owned() }
    }

    pub fn store(&self, key: &str, value: &str) {
        fs::write(self.dir.join(key), value.as_bytes()).unwrap();
    }

    pub fn load(&self, key: &str) -> String {
        fs::read_to_string(self.dir.join(key)).unwrap()
    }
}
