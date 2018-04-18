extern crate gcide;

#[macro_use]
extern crate structopt;

use gcide::Parser;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(name = "INFILE", help = "GNU CIDE file", parse(from_os_str))]
    input: PathBuf,
    #[structopt(name = "OUTFILE", help = "output file (default: overwrite)", parse(from_os_str))]
    output: Option<PathBuf>,
}

fn patch(contents: &str, output: &Path) -> Result<(), std::io::Error> {
    use std::io::Write;
    let mut patched = String::with_capacity(contents.len());
    let mut block_iter = Parser::new(contents);
    while let Some((skipped, block)) = block_iter.next() {
        use std::fmt::Write;
        patched.push_str(skipped);
        let block = block.unwrap();
        write!(patched, "{}", block).unwrap();
    }
    patched.push_str(block_iter.remaining());
    std::fs::File::create(output)?.write_all(patched.as_bytes())
}

fn main() {
    let opt = Opt::from_args();
    let output = opt.output.as_ref().unwrap_or(&opt.input);
    let contents = gcide::read_file(&opt.input).unwrap();
    patch(&contents, &output).unwrap();
}
