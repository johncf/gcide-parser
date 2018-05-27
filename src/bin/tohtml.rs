extern crate gcide;

use gcide::{binutils, EntryParser};
use gcide::exporter::HTML;

const HTMLHEAD: &str = r#"<!DOCTYPE html>
<html>
<head>
<meta http-equiv="content-type" content="text/html; charset=utf-8">
<title>Webster's Unabridged Dictionary 1913</title>
</head>
<body>
"#;

const HTMLTAIL: &str = "\n</body>\n</html>";

fn conv_html(contents: &str) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(contents.len()/3);
    write!(output, "{}", HTMLHEAD).unwrap();
    let mut entry_iter = EntryParser::new(contents);
    while let Some(entry_res) = entry_iter.next() {
        match entry_res {
            Ok(entry) => write!(output, "\n{}\n", HTML(&entry)).unwrap(),
            Err(_) => write!(output, "\n<!-- ERROR while parsing an entry -->\n").unwrap(),
        }
    }
    write!(output, "{}", HTMLTAIL).unwrap();
    output
}

fn main() {
    binutils::pipe_through(conv_html);
}
