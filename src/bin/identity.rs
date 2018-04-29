extern crate gcide;

use gcide::{binutils, EntryBuilder};

fn patch(contents: &str) -> String {
    let mut patched = String::with_capacity(contents.len());
    let mut entry_iter = EntryBuilder::new(contents);
    if let Some(preface) = entry_iter.get_preface() {
        patched.push_str(preface);
        patched.push('\n');
    }
    while let Some(entry_res) = entry_iter.next() {
        use std::fmt::Write;
        match entry_res {
            Ok(entry) => write!(patched, "\n{}\n", entry).unwrap(),
            Err(err) => write!(patched, "\n{}\n", err).unwrap(),
        }
    }
    patched.push_str(entry_iter.remaining());
    patched
}

fn main() {
    binutils::patch_using(patch);
}
