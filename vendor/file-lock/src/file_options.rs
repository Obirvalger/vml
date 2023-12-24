use std::fs::{File, OpenOptions};
use std::path::Path;

/// A wrapper around the open options type
/// that can be queried for the write permissions
///
/// This type has the exact same API as [`std::fs::OpenOptions`]
/// with the exception that the confluent interface passes `self`
/// rather than `&mut self`.
pub struct FileOptions {
    open_options: OpenOptions,
    pub(crate) writeable: bool,
}

impl FileOptions {
    pub fn new() -> Self {
        Self {
            open_options: OpenOptions::new(),
            writeable: false,
        }
    }

    pub fn append(mut self, append: bool) -> Self {
        self.open_options.append(append);
        self.writeable = true;
        self
    }

    pub fn create(mut self, create: bool) -> Self {
        self.open_options.create(create);
        self
    }

    pub fn create_new(mut self, create_new: bool) -> Self {
        self.open_options.create_new(create_new);
        self
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> std::io::Result<File> {
        self.open_options.open(path)
    }

    pub fn read(mut self, read: bool) -> Self {
        self.open_options.read(read);
        self.writeable = read;
        self
    }

    pub fn truncate(mut self, truncate: bool) -> Self {
        self.open_options.truncate(truncate);
        self
    }

    pub fn write(mut self, write: bool) -> Self {
        self.open_options.write(write);
        self.writeable = write;
        self
    }
}

impl Default for FileOptions {
    fn default() -> Self {
        Self::new()
    }
}
