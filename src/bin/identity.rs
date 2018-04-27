extern crate gcide;

use gcide::{binutils, Parser};

fn patch(contents: &str) -> String {
    let mut patched = String::with_capacity(contents.len());
    let mut block_iter = Parser::new(contents);
    while let Some((skipped, block_res)) = block_iter.next() {
        use std::fmt::Write;
        if skipped.len() > 0 && skipped.starts_with("<--") {
            if !skipped.starts_with("<-- This file is part ") {
                patched.push('\n');
            }
            patched.push_str(skipped);
            patched.push('\n');
        }
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
