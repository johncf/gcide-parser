use std::{fmt, process};
use std::fs::File;
use std::io::{Error, Read};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct PatchOpt {
    #[structopt(name = "INFILE", help = "GNU CIDE file", parse(from_os_str))]
    infile: PathBuf,
    #[structopt(name = "OUTFILE", help = "output file (default: overwrite)", parse(from_os_str))]
    outfile: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
struct PipeOpt {
    #[structopt(name = "INFILE", help = "GNU CIDE file", parse(from_os_str))]
    infile: PathBuf,
    #[structopt(name = "OUTFILE", help = "output file (default: stdout)", parse(from_os_str))]
    outfile: Option<PathBuf>,
}

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let mut contents = Vec::with_capacity(2 << 20);
    File::open(path)?.read_to_end(&mut contents)?;
    Ok(String::from_utf8_lossy(&contents).into_owned())
}

pub fn patch_using<F>(patcher: F)
where F: Fn(&str) -> String {
    use std::io::Write;
    let opt = PatchOpt::from_args();
    let outfile = opt.outfile.as_ref().unwrap_or(&opt.infile);
    let contents = read_file(&opt.infile).unwrap_abort();
    let patched = patcher(&contents);
    let mut output_file = File::create(outfile).unwrap_abort();
    output_file.write_all(patched.as_bytes()).unwrap_abort();
}

pub fn pipe_through<F>(processor: F)
where F: Fn(&str) -> String {
    use std::io::Write;
    let opt = PipeOpt::from_args();
    let contents = read_file(&opt.infile).unwrap_abort();
    let output = processor(&contents);
    if let Some(outfile) = opt.outfile {
        let mut output_file = File::create(outfile).unwrap_abort();
        output_file.write_all(output.as_bytes()).unwrap_abort();
    } else {
        ::std::io::stdout().write_all(output.as_bytes()).unwrap_abort();
    }
}

trait UnwrapAbort {
    type Out;

    fn unwrap_abort(self) -> Self::Out;
}

impl<T, E: fmt::Display> UnwrapAbort for Result<T, E> {
    type Out = T;

    fn unwrap_abort(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{}", e);
                process::abort();
            }
        }
    }
}
