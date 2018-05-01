use std::fmt::{self, Display, Formatter};

use nom::types::CompleteStr;
use nom::{alphanumeric1, self};

#[derive(Debug)]
pub struct Block<'a> {
    pub items: Vec<BlockItem<'a>>,
    pub sources: Vec<&'a str>,
}

impl<'a> Display for Block<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<p>")?;
        for t in &self.items {
            write!(f, "{}", t)?;
        }
        if self.sources.len() > 0 {
            let mut sources = Vec::new();
            for s in &self.sources {
                sources.push(format!("<source>{}</source>", s));
            }
            write!(f, "<br/\n[{}]</p>", sources.join(" + "))
        } else {
            write!(f, "</p>")
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BlockItem<'a> {
    Tagged(&'a str, Vec<BlockItem<'a>>),
    Comment(&'a str),
    Entity(&'a str),
    EntityBr,
    EntityUnk,
    ExternalLink(&'a str, &'a str),
    Plain(&'a str),
    Source(&'a str),
    UnpairedTagOpen(&'a str),
    UnpairedTagClose(&'a str),
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
            Plain(text) => write!(f, "{}", text),
            Source(text) => write!(f, "[ERROR->]<source>{}</source>", text),
            Tagged(name, ref items) => {
                write!(f, "<{}>", name)?;
                for item in items {
                    write!(f, "{}", item)?;
                }
                write!(f, "</{}>", name)
            }
            UnpairedTagOpen(name) => if allowed_to_dangle.contains(&name) {
                write!(f, "<{}>", name)
            } else {
                write!(f, "[ERROR->]<{}>", name)
            }
            UnpairedTagClose(name) => if allowed_to_dangle.contains(&name) {
                write!(f, "</{}>", name)
            } else {
                write!(f, "[ERROR->]</{}>", name)
            }
        }
    }
}

named!(parse_items<CompleteStr, Vec<BlockItem>>, many0!(block_item));

named!(block_item<CompleteStr, BlockItem>,
       alt!(plain | open_tag | close_tag | entity | comment | ext_link));

named!(plain<CompleteStr, BlockItem>,
       map!(is_not!("<>"), |s| BlockItem::Plain(s.0)));

named!(open_tag<CompleteStr, BlockItem>,
       map!(delimited!(tag!("<"), alphanumeric1, tag!(">")), |s| BlockItem::UnpairedTagOpen(s.0)));

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
           url: is_not!("\"") >>
           tag!("\">") >>
           text: is_not!("<>") >>
           tag!("</a>") >>
           ( BlockItem::ExternalLink(url.0, text.0) )));

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
        self.contents.find("<p>").map(|start_idx| {
            let remaining = &self.contents[start_idx..];
            let end_idx = match remaining.find("</p>") {
                Some(i) => i + 4,
                None => {
                    self.contents = ""; // further parsing not possible
                    return Err(ParserError {
                        leading: "",
                        trailing: remaining,
                    });
                }
            };
            let block_str = &remaining[..end_idx];
            self.contents = &remaining[end_idx..];
            match parse_items(CompleteStr(block_str)) {
                Ok((unparsed, items)) => {
                    if unparsed.len() > 0 {
                        self.contents = ""; // further parsing not needed
                        let lead_len = block_str.len() - unparsed.len();
                        Err(ParserError {
                            leading: &block_str[..lead_len],
                            trailing: &remaining[lead_len..],
                        })
                    } else {
                        Ok(process_block_items(items))
                    }
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
}

named!(entry_head<&str, EntryHead>,
       do_parse!(
           tag!("<entry ") >>
           main_word: delimited!(tag!("main-word=\""), is_not!("\""), tag!("\">\n")) >>
           ( EntryHead { main_word } )));

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
                Ok((block_str, EntryHead { main_word })) => {
                    let mut blocks = Vec::new();
                    for block_res in BlockParser::new(block_str) {
                        match block_res {
                            Ok(block) => blocks.push(block),
                            Err(ParserError { trailing, .. }) => {
                                let lead_len = end_idx + 1 - trailing.len();
                                return Err(ParserError {
                                    leading: &remaining[..lead_len],
                                    trailing: &remaining[lead_len..end_idx + close_len],
                                })
                            },
                        }
                    }
                    Ok(Entry { main_word, blocks })
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
        write!(f, "<entry main-word=\"{}\">", self.main_word)?;
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
                if let Some(open_idx) = linear_search_rev(&stack, &UnpairedTagOpen(name)) {
                    if name == "source" {
                        if open_idx == stack.len() - 2 {
                            if let Some(Plain(text)) = stack.pop() {
                                stack[open_idx] = Source(text);
                                continue;
                            }
                        }
                        stack.drain(open_idx+1..);
                        stack[open_idx] = Source("[ERROR->]plaintext");
                    } else {
                        if name == "col" && open_idx == stack.len() - 2 {
                            if let Some(&Tagged("b", _)) = stack.last() {
                                if let Some(Tagged("b", col_items)) = stack.pop() {
                                    stack[open_idx] = Tagged("col", col_items);
                                    continue;
                                }
                            }
                        }
                        let tagged = Tagged(name, stack.drain(open_idx+1..).collect());
                        stack[open_idx] = tagged;
                    }
                } else {
                    stack.push(item);
                }
            }
            _ => stack.push(item),
        }
    }
    stack
}

fn process_block_items<'a>(mut items: Vec<BlockItem<'a>>) -> Block<'a> {
    use self::BlockItem::*;

    assert_eq!(items.remove(0), UnpairedTagOpen("p"));
    assert_eq!(items.pop(), Some(UnpairedTagClose("p")));

    items = pair_up_items(items);

    let mut sources = Vec::new();
    if let Some(&Plain("]")) = items.last() {
        let is_bracket_open = |item: &BlockItem| *item == Plain("[") || *item == Plain("\n[");
        if let Some(idx) = linear_search_rev_by(&items, is_bracket_open) {
            if let Some(&Source(_)) = items.get(idx+1) {
                for item in &items[idx+1..] {
                    match *item {
                        Source(name) => sources.push(name),
                        _ => (),
                    }
                }
                match items[idx-1] {
                    EntityBr => items.drain(idx-1..),
                    _ => items.drain(idx..),
                };
            }
        }
    }

    Block {
        items: items,
        sources: sources,
    }
}

fn linear_search_rev<T: PartialEq>(haystack: &Vec<T>, needle: &T) -> Option<usize> {
    linear_search_rev_by(haystack, |item| item == needle)
}

fn linear_search_rev_by<T, F>(haystack: &Vec<T>, is_needle: F) -> Option<usize>
where T: PartialEq, F: Fn(&T) -> bool {
    for (idx, item) in haystack.iter().enumerate().rev() {
        if is_needle(item) {
            return Some(idx);
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
        assert_eq!(block_str, identity(block_str));
    }

    #[test]
    fn unpaired() {
        let block_str = "<entry main-word=\"Q\">\n<p><hw>Q</hw> <def>here are two <i>unpaired tags</b>.</def></p>\n</entry>";
        let expected = "<entry main-word=\"Q\">\n<p><hw>Q</hw> <def>here are two [ERROR->]<i>unpaired tags[ERROR->]</b>.</def></p>\n</entry>";
        assert_eq!(expected, identity(block_str));
    }
}
