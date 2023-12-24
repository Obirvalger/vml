extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/file_lock.c")
        .compile("libfile_lock.a")
}
