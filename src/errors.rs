use std::io;

#[derive(Debug, Clone)]
pub enum Error {
    DiskDoesNotExists { disk_path: String, vm_name: String },
    DownloadImage(String),
    EmptyVMsList,
    Other(String, String),
    ParseConfig(String),
    ParseImagesFile { images_file_path: String, error: String },
    ParseVMConfig { config_path: String, error: String },
    ParseVMConfigField { vm_name: String, field: String },
    RemoveRuuningVM(String),
    Template { place: String, error: String },
    VMHasNoPid(String),
    VMHasNoSSH(String),
}

impl Error {
    pub fn disk_does_not_exists(disk_path: &str, vm_name: &str) -> Error {
        let vm_name = vm_name.to_string();
        let disk_path = disk_path.to_string();
        Error::DiskDoesNotExists { disk_path, vm_name }
    }

    pub fn parse_images_file(images_file_path: &str, error: &str) -> Error {
        let images_file_path = images_file_path.to_string();
        let error = error.to_string();
        Error::ParseImagesFile { images_file_path, error }
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
