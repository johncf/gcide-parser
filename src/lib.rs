#[macro_use]
extern crate nom;

#[cfg(feature = "binaries")]
#[macro_use]
extern crate structopt;

#[macro_use]
extern crate bitflags;

extern crate unicode_normalization;

#[cfg(feature = "binaries")]
pub mod binutils;

pub mod parser;
pub mod exporter;

pub use parser::{Entry, EntryParser};
pub use exporter::CIDE;
