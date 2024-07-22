# NAME

file-lock - File locking via POSIX advisory record locks

This crate provides the facility to lock and unlock a file following the
advisory record lock scheme as specified by UNIX IEEE Std 1003.1-2001 (POSIX.1)
via fcntl().

# USAGE

    extern crate file_lock;

    use file_lock::{FileLock, FileOptions};
    use std::io::prelude::*;

    fn main() {
        let should_we_block  = true;
        let lock_for_writing = FileOptions::new().write(true).create_new(true);

        let mut filelock = match FileLock::lock("myfile.txt", should_we_block, lock_for_writing) {
            Ok(lock) => lock,
            Err(err) => panic!("Error getting write lock: {}", err),
        };

        filelock.file.write_all(b"Hello, World!").is_ok();

        // Manually unlocking is optional as we unlock on Drop
        filelock.unlock();
    }

# DOCUMENTATION

* [https://docs.rs/file-lock/](https://docs.rs/file-lock/)

# SUPPORT

Please report any bugs or feature requests at:

* [https://github.com/alfiedotwtf/file-lock/issues](https://github.com/alfiedotwtf/file-lock/issues)

Feel free to fork the repository and submit pull requests :)

# DEPENDENCIES

* [gcc](https://gcc.gnu.org/)

# SEE ALSO

* [Lock, Stock and Two Smoking Barrels](https://www.imdb.com/title/tt0120735/)

# AUTHORS

[Alfie John](https://www.alfie.wtf)

Corey Richardson

Ed Branch

Jacob Turner

Mateusz Kondej

Michael Lohr

Quang Luong

Sebastian Thiel

# WARRANTY

IT COMES WITHOUT WARRANTY OF ANY KIND.

# COPYRIGHT AND LICENSE

MIT License

Copyright (c) 2021 Alfie John

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
