#[macro_use]
extern crate nom;

#[cfg(feature = "binaries")]
#[macro_use]
extern crate structopt;

#[cfg(feature = "binaries")]
pub mod binutils;

pub mod parser;
pub mod exporter;

pub use parser::EntryParser;
pub use exporter::CIDE;
