use std::path::PathBuf;

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("bad cidr `{0}`")]
    BadCidr(String),
    #[error("create existing vm `{0}`")]
    CreateExistingVM(String),
    #[error("cloud init image `{0}` does not exist")]
    CloudInitImageDoesNotExists(PathBuf),
    #[error("disk `{disk_path}` does not exist for vm `{vm_name}`")]
    DiskDoesNotExists { disk_path: String, vm_name: String },
    #[error("download image `{0}` error")]
    DownloadImage(String),
    #[error("no vm found for given criteria")]
    EmptyVMsList,
    #[error("embedded file `{0}` does not exist")]
    GetWrongEmbeddedFile(String),
    #[error("image `{0}` does not exist")]
    ImageDoesNotExists(String),
    #[error("try to remove running vm `{0}`")]
    RemoveRuuningVM(String),
    #[error("try to store image to existing file `{0}`")]
    RewriteExistsPath(String),
    #[error("start runnig vm `{0}`")]
    StartRunningVM(String),
    #[error("can't ssh to vm `{0}`")]
    SshFailed(String),
    #[error("can't find private ssh key for vm `{0}`")]
    SshPrivateKeyDoesNotExists(String),
    #[error("can't find public ssh key for vm `{0}`")]
    SshPublicKeyDoesNotExists(String),
    #[error("can't start vm `{0}`")]
    StartVmFailed(String),
    #[error("trying to store running vm")]
    StoreRunningVM(String),
    #[error("unset tap device for tap network")]
    TapNetworkTapUnset,
    #[error("unknown image `{0}`")]
    UnknownImage(String),
    #[error("vm `{0}` is not runnig or its pid could not be found")]
    VMHasNoPid(String),
    #[error("no ssh options specified for vm `{0}`")]
    VMHasNoSsh(String),
}
