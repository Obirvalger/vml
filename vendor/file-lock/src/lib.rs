//! File locking via POSIX advisory record locks.
//!
//! This crate provides the facility to obtain a write-lock and unlock a file
//! following the advisory record lock scheme as specified by UNIX IEEE Std 1003.1-2001
//! (POSIX.1) via `fcntl()`.
//!
//! # Examples
//!
//! Please note that the examples use `tempfile` merely to quickly create a file
//! which is removed automatically. In the common case, you would want to lock
//! a file which is known to multiple processes.
//!
//! ```
//! extern crate file_lock;
//!
//! use file_lock::{FileLock, FileOptions};
//! use std::fs::OpenOptions;
//! use std::io::prelude::*;
//!
//! fn main() {
//!     let should_we_block  = true;
//!     let options = FileOptions::new()
//!                         .write(true)
//!                         .create(true)
//!                         .append(true);
//!
//!     let mut filelock = match FileLock::lock("myfile.txt", should_we_block, options) {
//!         Ok(lock) => lock,
//!         Err(err) => panic!("Error getting write lock: {}", err),
//!     };
//!
//!     filelock.file.write_all(b"Hello, World!").is_ok();
//!
//!     // Manually unlocking is optional as we unlock on Drop
//!     filelock.unlock();
//! }
//! ```

mod file_options;

use libc::c_int;
use std::fs::File;
use std::io::Error;
use std::os::fd::AsRawFd;
use std::path::Path;

pub use file_options::FileOptions;

extern "C" {
    fn c_lock(fd: i32, is_blocking: i32, is_writeable: i32) -> c_int;
    fn c_unlock(fd: i32) -> c_int;
}

/// Represents the actually locked file
#[derive(Debug)]
pub struct FileLock {
    /// the `std::fs::File` of the file that's locked
    pub file: File,
}

impl FileLock {
    /// Try to lock the specified file
    ///
    /// # Parameters
    ///
    /// `path` is the path of the file we want to lock on
    ///
    /// `is_blocking` is a flag to indicate if we should block if it's already locked
    ///
    /// `options` is a mutable reference to a [`std::fs::OpenOptions`] object to configure the underlying file
    ///
    /// # Examples
    ///
    ///```
    ///extern crate file_lock;
    ///
    ///use file_lock::{FileLock, FileOptions};
    ///use std::fs::OpenOptions;
    ///use std::io::prelude::*;
    ///
    ///fn main() {
    ///    let should_we_block  = true;
    ///    let options = FileOptions::new()
    ///                        .write(true)
    ///                        .create(true)
    ///                        .append(true);
    ///
    ///    let mut filelock = match FileLock::lock("myfile.txt", should_we_block, options) {
    ///        Ok(lock) => lock,
    ///        Err(err) => panic!("Error getting write lock: {}", err),
    ///    };
    ///
    ///    filelock.file.write_all(b"Hello, World!").is_ok();
    ///}
    ///```
    ///
    pub fn lock<P: AsRef<Path>>(
        path: P,
        is_blocking: bool,
        options: FileOptions,
    ) -> Result<FileLock, Error> {
        let file = options.open(path)?;
        let is_writeable = options.writeable;

        let errno = unsafe { c_lock(file.as_raw_fd(), is_blocking as i32, is_writeable as i32) };

        match errno {
            0 => Ok(FileLock { file }),
            _ => Err(Error::from_raw_os_error(errno)),
        }
    }

    /// Unlock our locked file
    ///
    /// *Note:* This method is optional as the file lock will be unlocked automatically when dropped
    ///
    /// # Examples
    ///
    ///```
    ///extern crate file_lock;
    ///
    ///use file_lock::{FileLock, FileOptions};
    ///use std::io::prelude::*;
    ///
    ///fn main() {
    ///    let should_we_block  = true;
    ///    let lock_for_writing = FileOptions::new().write(true).create(true);
    ///
    ///    let mut filelock = match FileLock::lock("myfile.txt", should_we_block, lock_for_writing) {
    ///        Ok(lock) => lock,
    ///        Err(err) => panic!("Error getting write lock: {}", err),
    ///    };
    ///
    ///    filelock.file.write_all(b"Hello, World!").is_ok();
    ///
    ///    match filelock.unlock() {
    ///        Ok(_)    => println!("Successfully unlocked the file"),
    ///        Err(err) => panic!("Error unlocking the file: {}", err),
    ///    };
    ///}
    ///```
    ///
    pub fn unlock(&self) -> Result<(), Error> {
        let errno = unsafe { c_unlock(self.file.as_raw_fd()) };

        match errno {
            0 => Ok(()),
            _ => Err(Error::from_raw_os_error(errno)),
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.unlock().is_ok();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use nix::unistd::fork;
    use nix::unistd::ForkResult::{Child, Parent};
    use std::fs::{remove_file, OpenOptions};
    use std::process;
    use std::thread::sleep;
    use std::time::Duration;

    fn standard_options(is_writable: &bool) -> FileOptions {
        FileOptions::new()
            .read(!*is_writable)
            .write(*is_writable)
            .create(*is_writable)
    }

    #[test]
    fn lock_and_unlock() {
        let filename = "filelock.test";

        for already_exists in &[true, false] {
            for already_locked in &[true, false] {
                for already_writable in &[true, false] {
                    for is_blocking in &[true, false] {
                        for is_writable in &[true, false] {
                            if !*already_exists && (*already_locked || *already_writable) {
                                // nonsensical tests
                                continue;
                            }

                            let _ = remove_file(&filename).is_ok();

                            let parent_lock = match *already_exists {
                                false => None,
                                true => {
                                    OpenOptions::new()
                                        .write(true)
                                        .create(true)
                                        .open(&filename)
                                        .expect("Test failed");

                                    match *already_locked {
                                        false => None,
                                        true => {
                                            let options = standard_options(already_writable);
                                            match FileLock::lock(filename, true, options) {
                                                Ok(lock) => Some(lock),
                                                Err(err) => {
                                                    panic!("Error creating parent lock ({})", err)
                                                }
                                            }
                                        }
                                    }
                                }
                            };

                            unsafe {
                                match fork() {
                                    Ok(Parent { child: _ }) => {
                                        sleep(Duration::from_millis(150));

                                        if let Some(lock) = parent_lock {
                                            lock.unlock().expect("Test failed");
                                        }

                                        sleep(Duration::from_millis(350));
                                    }
                                    Ok(Child) => {
                                        let mut try_count = 0;
                                        let mut locked = false;

                                        match *already_locked {
                                            true => match *is_blocking {
                                                true => {
                                                    let options = standard_options(is_writable);
                                                    match FileLock::lock(filename, *is_blocking, options) {
                                                    Ok(_)  => { locked = true },
                                                    Err(_) => panic!("Error getting lock after wating for release"),
                                                }
                                                }
                                                false => {
                                                    for _ in 0..5 {
                                                        let options = standard_options(is_writable);
                                                        match FileLock::lock(
                                                            filename,
                                                            *is_blocking,
                                                            options,
                                                        ) {
                                                            Ok(_) => {
                                                                locked = true;
                                                                break;
                                                            }
                                                            Err(_) => {
                                                                sleep(Duration::from_millis(50));
                                                                try_count += 1;
                                                            }
                                                        }
                                                    }
                                                }
                                            },
                                            false => {
                                                let options = standard_options(is_writable);
                                                match FileLock::lock(
                                                    filename,
                                                    *is_blocking,
                                                    options,
                                                ) {
                                                    Ok(_) => locked = true,
                                                    Err(_) => {
                                                        match !*already_exists && !*is_writable {
                                                            true => {}
                                                            false => {
                                                                panic!("Error getting lock with no competition")
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        match !already_exists && !is_writable {
                                            true => assert!(
                                            !locked,
                                            "Locking a non-existent file for reading should fail"
                                        ),
                                            false => {
                                                assert!(locked, "Lock should have been successful")
                                            }
                                        }

                                        match *is_blocking {
                                        true  => assert_eq!(try_count, 0, "Try count should be zero when blocking"),
                                        false => {
                                            match *already_locked {
                                                false => assert_eq!(try_count, 0, "Try count should be zero when no competition"),
                                                true  => match !already_writable && !is_writable {
                                                    true  => assert_eq!(try_count, 0, "Read lock when locked for reading should succeed first go"),
                                                    false => assert!(try_count >= 3, "Try count should be >= 3"),
                                                },
                                            }
                                        },
                                    }

                                        process::exit(7);
                                    }
                                    Err(_) => {
                                        panic!("Error forking tests :(");
                                    }
                                }
                            }

                            let _ = remove_file(&filename).is_ok();
                        }
                    }
                }
            }
        }
    }
}
