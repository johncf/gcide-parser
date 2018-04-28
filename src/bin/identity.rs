extern crate gcide;

use gcide::{binutils, Parser};

fn patch(contents: &str) -> String {
    let mut patched = String::with_capacity(contents.len());
    let mut block_iter = Parser::new(contents);
    if let Some(preface) = block_iter.get_preface() {
        patched.push_str(preface);
        patched.push('\n');
    }
    while let Some(block_res) = block_iter.next() {
        use std::fmt::Write;
        match block_res {
            Ok(block) => write!(patched, "\n{}\n", block).unwrap(),
            Err(err) => write!(patched, "\n{}\n", err).unwrap(),
        }
    }
    patched.push_str(block_iter.remaining());
    patched
}

fn main() {
    binutils::patch_using(patch);
}
