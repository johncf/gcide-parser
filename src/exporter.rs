use std::fmt::{self, Display, Formatter};

use parser::{Entry, EntryItem};

pub struct CIDE<'a>(pub &'a Entry<'a>);
//pub struct HTML<'a>(pub &'a Entry<'a>);
//pub struct Plain<'a>(pub &'a Entry<'a>);

trait DisplayCIDE {
    fn fmt_cide(&self, f: &mut Formatter) -> fmt::Result;
}

impl<'a> Display for CIDE<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt_cide(f)
    }
}

impl<'a> DisplayCIDE for Entry<'a> {
    fn fmt_cide(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<entry main-word=\"{}\" source=\"{}\">", self.main_word, self.source)?;
        for item in &self.items {
            item.fmt_cide(f)?;
        }
        write!(f, "</entry>")
    }
}

impl<'a> DisplayCIDE for EntryItem<'a> {
    fn fmt_cide(&self, f: &mut Formatter) -> fmt::Result {
        use parser::EntryItem::*;
        let allowed_to_dangle = &["collapse", "cs", "note", "usage"];
        match *self {
            Comment(text) => write!(f, "<--{}-->", text),
            Entity(name) => write!(f, "<{}/", name),
            EntityBr => write!(f, "<br/\n"),
            EntityUnk => write!(f, "<?/"),
            ExternalLink(url, text) => write!(f, "<a href=\"{}\">{}</a>", url, text),
            Greek(_) => unimplemented!(), // TODO
            PlainText(text) => write!(f, "{}", text),
            Tagged { name, ref items, source } => {
                write_tag_open(f, name, source)?;
                for item in items {
                    item.fmt_cide(f)?;
                }
                write!(f, "</{}>", name)
            }
            UnpairedTagOpen(name, source) => {
                if !allowed_to_dangle.contains(&name) {
                    write!(f, "[ERROR->]")?;
                }
                write_tag_open(f, name, source)
            }
            UnpairedTagClose(name) => {
                if !allowed_to_dangle.contains(&name) {
                    write!(f, "[ERROR->]</{}>", name)
                } else {
                    write!(f, "</{}>", name)
                }
            }
        }
    }
}

fn write_tag_open(f: &mut Formatter, name: &str, source: Option<&str>) -> fmt::Result {
    match source {
        Some(source) => {
            if name == "p" || name == "extra" {
                write!(f, "<{} source=\"{}\">", name, source)
            } else {
                write!(f, "<{} [ERROR->]source=\"{}\">", name, source)
            }
        }
        None => write!(f, "<{}>", name),
    }
}

#[cfg(test)]
mod test {
    use CIDE; use EntryParser;

    fn identity(input: &str) -> String {
        use std::fmt::Write;
        let mut entry_iter = EntryParser::new(input);
        let entry_res = entry_iter.next().expect("no block found!");
        assert!(entry_iter.remaining().is_empty());
        let entry = entry_res.expect("bad entry");
        let mut output = String::new();
        write!(output, "{}", CIDE(&entry)).unwrap();
        output
    }

    #[test]
    fn simple() {
        let block_str = "<entry main-word=\"Q\" source=\"1913 Webster\">\n<p><hw>Q</hw> <pr>(k<umac/)</pr>, <def>the seventeenth letter of the English alphabet.</def></p>\n</entry>";
        let expected = block_str;
        assert_eq!(expected, identity(block_str));
    }

    #[test]
    fn unpaired() {
        let block_str = "<entry main-word=\"Q\" source=\"\">\n<p><hw>Q</hw> <def>here are two <i>unpaired tags</b>.</def></p>\n</entry>";
        let expected = "<entry main-word=\"Q\" source=\"\">\n<p><hw>Q</hw> <def>here are two [ERROR->]<i>unpaired tags[ERROR->]</b>.</def></p>\n</entry>";
        assert_eq!(expected, identity(block_str));
    }
}
