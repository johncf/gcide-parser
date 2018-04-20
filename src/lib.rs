#[macro_use]
extern crate nom;

use std::io::{Error, Read};
use std::fs::File;
use std::path::Path;

pub mod parser;

pub use parser::Parser;

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let mut contents = Vec::with_capacity(2 << 20);
    File::open(path)?.read_to_end(&mut contents)?;
    Ok(String::from_utf8_lossy(&contents).into_owned())
}
