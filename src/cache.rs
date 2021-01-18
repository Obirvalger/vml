use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Cache {
    name: String,
    dir: PathBuf,
}

impl Cache {
    pub fn new(name: &str) -> Cache {
        let tmp =
            env::var("TMP").or_else(|_| env::var("TMPDIR")).unwrap_or_else(|_| "/tmp".to_string());
        let dir = PathBuf::from(tmp).join("vml").join(name).join(".cache");
        fs::create_dir_all(&dir).unwrap();
        Cache { name: name.to_string(), dir }
    }

    pub fn store(&self, key: &str, value: &str) {
        fs::write(self.dir.join(key), value.as_bytes()).unwrap();
    }

    pub fn load(&self, key: &str) -> String {
        fs::read_to_string(self.dir.join(key)).unwrap()
    }
}
