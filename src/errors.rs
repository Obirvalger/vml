use std::io;

#[derive(Debug, Clone)]
pub enum Error {
    EmptyVMsList,
    VMHasNoPid(String),
    VMHasNoSSH(String),
    ParseConfig(String),
    Template { place: String, error: String },
    ParseVMConfig { config_path: String, error: String },
    ParseVMConfigField { vm_name: String, field: String },
    DiskDoesNotExists { disk_path: String, vm_name: String },
    Other(String, String),
}

impl Error {
    pub fn disk_does_not_exists(disk_path: &str, vm_name: &str) -> Error {
        let vm_name = vm_name.to_string();
        let disk_path = disk_path.to_string();
        Error::DiskDoesNotExists { disk_path, vm_name }
    }

    pub fn parse_vm_config_field(vm_name: &str, field: &str) -> Error {
        let vm_name = vm_name.to_string();
        let field = field.to_string();
        Error::ParseVMConfigField { vm_name, field }
    }

    pub fn parse_vm_config(config_path: &str, error: &str) -> Error {
        let config_path = config_path.to_string();
        let error = error.to_string();
        Error::ParseVMConfig { config_path, error }
    }

    pub fn template(place: &str, error: &str) -> Error {
        let place = place.to_string();
        let error = error.to_string();
        Error::Template { place, error }
    }

    pub fn other(place: &str, error: &str) -> Error {
        Error::Other(place.to_string(), error.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Other("io".to_string(), error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
