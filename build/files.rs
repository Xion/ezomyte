//! Utilities for working with files.

use std::env;
use std::io;
use std::fs::{self, File};
use std::path::Path;


/// Open a file inside the build's $OUT_DIR for writing.
pub fn create_out_file<P: AsRef<Path>>(path: P) -> io::Result<File> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join(path);
    fs::create_dir_all(path.parent().unwrap())?;
    fs::OpenOptions::new()
        .create(true).truncate(true).write(true)
        .open(path)
}
