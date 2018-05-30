extern crate gcide;

use gcide::{binutils, Entry, EntryParser};
use gcide::parser::EntryItem;
use std::fmt::{self, Display, Formatter};

const HTMLHEAD: &str = r#"<!DOCTYPE html>
<html>
<head>
<meta http-equiv="content-type" content="text/html; charset=utf-8">
<title>Webster's Unabridged Dictionary 1913</title>
</head>
<body>
"#;

const HTMLTAIL: &str = "\n</body>\n</html>";

struct HTML<'a>(pub &'a Entry<'a>);

fn main() {
    binutils::pipe_through(conv_html);
}

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

impl<'a> Display for HTML<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt_html(f)
    }
}

trait DisplayHTML {
    fn fmt_html(&self, f: &mut Formatter) -> fmt::Result;
}

impl<'a> DisplayHTML for Entry<'a> {
    fn fmt_html(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<div class=\"entry\" data-word=\"{}\" data-source=\"{}\">", self.main_word, self.source)?;
        self.items.fmt_html(f)?;
        write!(f, "</div>")
    }
}

impl<'a> DisplayHTML for EntryItem<'a> {
    fn fmt_html(&self, f: &mut Formatter) -> fmt::Result {
        use gcide::parser::EntryItem::*;
        use gcide::exporter::process_symbols_in_text;
        match *self {
            Comment(_) => Ok(()),
            Entity(name) => write!(f, "{}", entity_to_html(name)),
            EntityBr => write!(f, "<br/>\n"),
            EntityUnk => write!(f, "&#xfffd;"),
            ExternalLink(url, text) => write!(f, "<a class=\"extern\" href=\"{}\">{}</a>", url, text),
            Greek(ref gitems) => {
                write!(f, "<em>")?;
                for gi in gitems {
                    gi.fmt(f)?;
                }
                write!(f, "</em>")
            }
            PlainText(text) => write!(f, "{}", process_symbols_in_text(text).replace("&", "&amp;")),
            Tagged { name, ref items, source } => {
                match name {
                    "p" => {
                        match source {
                            Some(source) => write!(f, "<p data-source=\"{}\">", source)?,
                            None => write!(f, "<p>")?,
                        }
                        items.fmt_html(f)?;
                        write!(f, "</p>")
                    }
                    "hw" => {
                        write!(f, "<strong class=\"hw\">")?;
                        items.fmt_html(f)?;
                        write!(f, "</strong>")
                    }
                    "ety" | "ets" | "etsep" | "pr" | "def" | "altname" | "col" | "cd" | "plain"
                        | "fld" | "mark" | "sd" | "sn" | "au" | "ecol" | "stype" => {
                        write!(f, "<span class=\"{}\">", name)?;
                        items.fmt_html(f)?;
                        write!(f, "</span>")
                    }
                    "pos" | "pluf" | "singf" => {
                        write!(f, "<em>")?;
                        items.fmt_html(f)?;
                        write!(f, "</em>")
                    }
                    "asp" | "adjf" | "conjf" | "decf" | "plw" | "singw" | "wf" => {
                        write!(f, "<strong class=\"altf\">")?;
                        items.fmt_html(f)?;
                        write!(f, "</strong>")
                    }
                    "er" | "snr" | "sdr" | "cref" => {
                        write!(f, "<a href=\"#\" class=\"{}\">", name)?;
                        items.fmt_html(f)?;
                        write!(f, "</a>")
                    }
                    "as" | "def2" | "altsp" | "cs" | "mcol" | "mhw" | "note" | "syn" | "usage"
                        | "mord" | "rj" | "specif" | "book" | "org" | "city" | "country" | "geog"
                        | "plu" | "sing" | "amorph" | "nmorph" | "vmorph" | "wordforms" => {
                        items.fmt_html(f)
                    }
                    "oneof" | "c" => { // TODO
                        items.fmt_html(f)
                    }
                    "q" | "qau" => { // TODO use blockquote
                        items.fmt_html(f)
                    }
                    "class" | "fam" | "gen" | "ord" | "spn" | "ex" | "qex" | "xex" | "it" => {
                        write!(f, "<em>")?;
                        items.fmt_html(f)?;
                        write!(f, "</em>")
                    }
                    _ => {
                        eprintln!("unknown tag: {}", name);
                        write!(f, "&#xfffd;<!--{}-->", name)
                    }
                }
            }
            UnpairedTagOpen(_, _) => Ok(()),
            UnpairedTagClose(_) => Ok(()),
        }
    }
}

impl<'a> DisplayHTML for Vec<EntryItem<'a>> {
    fn fmt_html(&self, f: &mut Formatter) -> fmt::Result {
        for item in self {
            item.fmt_html(f)?;
        }
        Ok(())
    }
}

fn entity_to_html(entity: &str) -> &'static str {
    use gcide::exporter::entity_to_unicode;
    match entity {
        "lt"       => "&lt;",
        "gt"       => "&gt;",
        "ait"     => "<i>a</i>",
        "eit"     => "<i>e</i>",
        "iit"     => "<i>i</i>",
        "oit"     => "<i>o</i>",
        "uit"     => "<i>u</i>",
        _          => entity_to_unicode(entity),
    }
}
