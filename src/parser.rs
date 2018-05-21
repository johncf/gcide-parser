use std::fmt::{self, Display, Formatter};

use nom::types::CompleteStr;
use nom::{alphanumeric1, self};

#[derive(Debug)]
pub struct Block<'a> {
    pub items: Vec<BlockItem<'a>>,
    pub source: Option<&'a str>,
}

impl<'a> Display for Block<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.source {
            Some(source) => write!(f, "<p source=\"{}\">", source)?,
            None => write!(f, "<p>")?,
        }
        for t in &self.items {
            write!(f, "{}", t)?;
        }
        write!(f, "</p>")
    }
}

#[derive(Debug, PartialEq)]
pub enum BlockItem<'a> {
    Tagged { name: &'a str, items: Vec<BlockItem<'a>>, source: Option<&'a str> },
    Comment(&'a str),
    Entity(&'a str),
    EntityBr,
    EntityUnk,
    ExternalLink(&'a str, &'a str),
    PlainText(&'a str),
    UnpairedTagOpen(&'a str, Option<&'a str>),
    UnpairedTagClose(&'a str),
}

fn write_tag_open(f: &mut Formatter, name: &str, source: Option<&str>) -> fmt::Result {
    match source {
        Some(source) => {
            if name == "extra" {
                write!(f, "<{} source=\"{}\">", name, source)
            } else {
                write!(f, "<{} [ERROR->]source=\"{}\">", name, source)
            }
        }
        None => write!(f, "<{}>", name),
    }
}

impl<'a> Display for BlockItem<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::BlockItem::*;
        let allowed_to_dangle = &["collapse", "cs", "note", "usage"];
        match *self {
            Comment(text) => write!(f, "<--{}-->", text),
            Entity(name) => write!(f, "<{}/", name),
            EntityBr => write!(f, "<br/\n"),
            EntityUnk => write!(f, "<?/"),
            ExternalLink(url, text) => write!(f, "<a href=\"{}\">{}</a>", url, text),
            PlainText(text) => write!(f, "{}", text),
            Tagged { name, ref items, source } => {
                write_tag_open(f, name, source)?;
                for item in items {
                    write!(f, "{}", item)?;
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

named!(parse_items<CompleteStr, Vec<BlockItem>>, many0!(block_item));

named!(block_item<CompleteStr, BlockItem>,
       alt!(plain_text | open_tag | close_tag | entity | comment | ext_link));

named!(plain_text<CompleteStr, BlockItem>,
       map!(is_not!("<>"), |s| BlockItem::PlainText(s.0)));

named!(source_attr<CompleteStr, CompleteStr>,
       delimited!(tag!(" source=\""), take_till!(|c| c == '"'), tag!("\"")));

named!(open_tag<CompleteStr, BlockItem>,
       do_parse!(
           tag!("<") >>
           name: alphanumeric1 >>
           source: opt!(source_attr) >>
           tag!(">") >>
           ( BlockItem::UnpairedTagOpen(name.0, source.map(|s| s.0)) )));

named!(close_tag<CompleteStr, BlockItem>,
       map!(delimited!(tag!("</"), alphanumeric1, tag!(">")), |s| BlockItem::UnpairedTagClose(s.0)));

named!(entity<CompleteStr, BlockItem>,
       alt!(map!(tag!("<?/"), |_| BlockItem::EntityUnk) |
            map!(tuple!(tag!("<br/"), opt!(char!('\n'))), |_| BlockItem::EntityBr) |
            map!(delimited!(tag!("<"), alphanumeric1, tag!("/")), |s| BlockItem::Entity(s.0))));

named!(comment<CompleteStr, BlockItem>,
       map!(delimited!(tag!("<--"), take_until!("-->"), tag!("-->")), |s| BlockItem::Comment(s.0)));

named!(ext_link<CompleteStr, BlockItem>,
       do_parse!(
           tag!("<a href=\"") >>
           url: take_till!(|c| c == '"') >>
           tag!("\">") >>
           text: is_not!("<>") >>
           tag!("</a>") >>
           ( BlockItem::ExternalLink(url.0, text.0) )));

named!(block_head<CompleteStr, Option<&str>>,
       do_parse!(
           tag!("<p") >>
           source: opt!(source_attr) >>
           tag!(">") >>
           ( source.map(|s| s.0) )));

struct BlockParser<'a> {
    contents: &'a str,
}

impl<'a> BlockParser<'a> {
    fn new(contents: &'a str) -> BlockParser<'a> {
        BlockParser { contents }
    }
}

impl<'a> Iterator for BlockParser<'a> {
    type Item = Result<Block<'a>, ParserError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.contents.find("<p").map(|start_idx| {
            let remaining = &self.contents[start_idx..];
            let end_idx = match remaining.find("</p>") {
                Some(i) => i,
                None => {
                    self.contents = ""; // further parsing not possible
                    return Err(ParserError {
                        leading: "",
                        trailing: remaining,
                    });
                }
            };
            self.contents = &remaining[end_idx + 4..];
            match block_head(CompleteStr(&remaining[..end_idx])) {
                Ok((block_str, source_opt)) => {
                    match parse_items(block_str) {
                        Ok((unparsed, items)) => {
                            if unparsed.len() > 0 {
                                self.contents = ""; // further parsing not needed
                                let lead_len = end_idx - unparsed.len();
                                Err(ParserError {
                                    leading: &remaining[..lead_len],
                                    trailing: &remaining[lead_len..],
                                })
                            } else {
                                Ok(Block {
                                    items: pair_up_items(items),
                                    source: source_opt,
                                })
                            }
                        }
                        Err(_) => unreachable!(),
                    }
                }
                Err(nom::Err::Error(nom::simple_errors::Context::Code(context, _))) => {
                    self.contents = ""; // further parsing not needed
                    let lead_len = end_idx - context.len();
                    Err(ParserError {
                        leading: &remaining[..lead_len],
                        trailing: &remaining[lead_len..],
                    })
                }
                Err(_) => unreachable!(),
            }
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ParserError<'a> {
    pub leading: &'a str,
    pub trailing: &'a str,
}

impl<'a> Display for ParserError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}[ERROR->]{}", self.leading, self.trailing)
    }
}

pub struct Entry<'a> {
    pub main_word: &'a str,
    pub blocks: Vec<Block<'a>>,
    pub source: &'a str,
}

pub struct EntryParser<'a> {
    contents: &'a str,
}

impl<'a> EntryParser<'a> {
    pub fn new(contents: &'a str) -> EntryParser<'a> {
        EntryParser { contents }
    }

    pub fn get_preface(&self) -> Option<&'a str> {
        if self.contents.starts_with("<-- This file is part") {
            self.contents.find("-->").map(|i| &self.contents[..i + 3])
        } else {
            None
        }
    }

    pub fn remaining(&self) -> &'a str {
        self.contents.trim()
    }
}

struct EntryHead<'a> {
    main_word: &'a str,
    source: &'a str,
}

named!(entry_head<&str, EntryHead>,
       do_parse!(
           tag!("<entry") >>
           main_word: delimited!(tag!(" main-word=\""), take_till!(|c| c == '"'), tag!("\"")) >>
           source: delimited!(tag!(" source=\""), take_till!(|c| c == '"'), tag!("\"")) >>
           tag!(">") >>
           ( EntryHead { main_word, source } )));

impl<'a> Iterator for EntryParser<'a> {
    type Item = Result<Entry<'a>, ParserError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.contents.find("<entry ").map(|start_idx| {
            let remaining = &self.contents[start_idx..];
            let end_idx = match remaining.find("</entry>") {
                Some(i) => i,
                None => {
                    self.contents = ""; // further parsing not possible
                    return Err(ParserError {
                        leading: "",
                        trailing: "",
                    });
                }
            };
            let close_len = "</entry>".len();
            self.contents = &remaining[end_idx + close_len..];
            match entry_head(&remaining[..end_idx]) {
                Ok((block_str, EntryHead { main_word, source })) => {
                    let mut blocks = Vec::new();
                    for block_res in BlockParser::new(block_str) {
                        match block_res {
                            Ok(mut block) => blocks.push(block),
                            Err(ParserError { trailing, .. }) => {
                                let lead_len = end_idx - trailing.len();
                                return Err(ParserError {
                                    leading: &remaining[..lead_len],
                                    trailing: &remaining[lead_len..end_idx + close_len],
                                })
                            },
                        }
                    }
                    Ok(Entry { main_word, blocks, source })
                }
                Err(nom::Err::Error(nom::simple_errors::Context::Code(context, _))) => {
                    let lead_len = end_idx - context.len();
                    Err(ParserError {
                        leading: &remaining[..lead_len],
                        trailing: &remaining[lead_len..end_idx + close_len],
                    })
                }
                Err(_) => unreachable!(),
            }
        })
    }
}

impl<'a> Display for Entry<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<entry main-word=\"{}\" source=\"{}\">", self.main_word, self.source)?;
        for b in &self.blocks {
            write!(f, "\n{}\n", b)?;
        }
        write!(f, "</entry>")
    }
}

fn pair_up_items<'a>(items: Vec<BlockItem<'a>>) -> Vec<BlockItem<'a>> {
    use self::BlockItem::*;

    let mut stack = Vec::with_capacity(items.len()*2/3 + 1);
    for item in items {
        match item {
            UnpairedTagClose(name) => {
                let is_tag_open_name = |item: &BlockItem<'a>| {
                    match *item {
                        UnpairedTagOpen(n, src) if n == name => Some(src),
                        _ => None,
                    }
                };
                if let Some((open_idx, source)) = linear_search_rev_by(&stack, is_tag_open_name) {
                    let tagged = Tagged {
                        name: name,
                        items: stack.drain(open_idx+1..).collect(),
                        source: source,
                    };
                    stack[open_idx] = tagged;
                } else {
                    stack.push(item);
                }
            }
            _ => stack.push(item),
        }
    }
    stack
}

fn linear_search_rev_by<T, U, F>(haystack: &Vec<T>, filter_map: F) -> Option<(usize, U)>
where T: PartialEq, F: Fn(&T) -> Option<U> {
    for (idx, item) in haystack.iter().enumerate().rev() {
        if let Some(out) = filter_map(item) {
            return Some((idx, out));
        }
    }
    return None;
}

#[cfg(test)]
mod test {
    use super::EntryParser;

    fn identity(input: &str) -> String {
        use std::fmt::Write;
        let mut entry_iter = EntryParser::new(input);
        let block_res = entry_iter.next().expect("no block found!");
        assert!(entry_iter.remaining().is_empty());
        let block = block_res.expect("bad block");
        let mut output = String::new();
        write!(output, "{}", block).unwrap();
        output
    }

    #[test]
    fn simple() {
        let block_str = "<entry main-word=\"Q\">\n<p><hw>Q</hw> <pr>(k<umac/)</pr>, <def>the seventeenth letter of the English alphabet.</def><br/\n[<source>1913 Webster</source>]</p>\n</entry>";
        let expected = "<entry main-word=\"Q\" source=\"1913 Webster\">\n<p><hw>Q</hw> <pr>(k<umac/)</pr>, <def>the seventeenth letter of the English alphabet.</def></p>\n</entry>";
        assert_eq!(expected, identity(block_str));
    }

    #[test]
    fn unpaired() {
        let block_str = "<entry main-word=\"Q\">\n<p><hw>Q</hw> <def>here are two <i>unpaired tags</b>.</def></p>\n</entry>";
        let expected = "<entry main-word=\"Q\" source=\"\">\n<p><hw>Q</hw> <def>here are two [ERROR->]<i>unpaired tags[ERROR->]</b>.</def></p>\n</entry>";
        assert_eq!(expected, identity(block_str));
    }
}
