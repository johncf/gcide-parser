use std::fmt::{self, Display, Formatter};

use nom::types::CompleteStr;
use nom::alphanumeric1;

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

pub struct Parser<'a> {
    contents: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(contents: &'a str) -> Parser<'a> {
        Parser { contents }
    }

    pub fn remaining(&self) -> &'a str {
        self.contents.trim()
    }
}

named!(block_start<&str, &str>, take_until!("<p>"));

impl<'a> Iterator for Parser<'a> {
    type Item = (&'a str, Result<Block<'a>, ParserError<'a>>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok((remaining, skipped)) = block_start(self.contents) {
            let end_idx_opt = remaining.find("</p>");
            if end_idx_opt.is_none() {
                self.contents = "";
                return Some((skipped, Err(ParserError {
                    leading: &remaining[..0],
                    trailing: remaining,
                })));
            }
            let end_idx = end_idx_opt.unwrap() + 4;
            let block_str = &remaining[..end_idx];
            self.contents = &remaining[end_idx..];
            match parse_items(CompleteStr(block_str)) {
                Ok((unparsed, items)) => {
                    if unparsed.len() > 0 {
                        let lead_len = block_str.len() - unparsed.len();
                        Some((skipped.trim(), Err(ParserError {
                            leading: &block_str[..lead_len],
                            trailing: unparsed.0,
                        })))
                    } else {
                        Some((skipped.trim(), Ok(process_block_items(items))))
                    }
                }
                Err(_) => unreachable!(),
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ParserError<'a> {
    pub leading: &'a str,
    pub trailing: &'a str,
}

impl<'a> Display for ParserError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}[ERROR->]{}", self.leading, self.trailing)
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
    use super::Parser;

    fn identity(input: &str) -> String {
        use std::fmt::Write;
        let mut block_iter = Parser::new(input);
        let (skipped, block_res) = block_iter.next().expect("no block found!");
        assert!(skipped.is_empty());
        assert!(block_iter.remaining().is_empty());
        let block = block_res.expect("bad block");
        let mut output = String::new();
        write!(output, "{}", block).unwrap();
        output
    }

    #[test]
    fn simple() {
        let block_str = "<p><ent>Q</ent><br/\n<hw>Q</hw> <pr>(k<umac/)</pr>, <def>the seventeenth letter of the English alphabet.</def><br/\n[<source>1913 Webster</source>]</p>";
        assert_eq!(block_str, identity(block_str));
    }

    #[test]
    fn source_misplaced() {
        let block_str = "<p><ent>Q</ent><br/\n<hw>Q</hw> <pr>(k<umac/)</pr>, <def>the seventeenth letter of the English alphabet.</def><source>1913 Webster</source></p>";
        let expected = "<p><ent>Q</ent><br/\n<hw>Q</hw> <pr>(k<umac/)</pr>, <def>the seventeenth letter of the English alphabet.</def>[ERROR->]<source>1913 Webster</source></p>";
        assert_eq!(expected, identity(block_str));
    }

    #[test]
    fn source_nonplain() {
        let block_str = "<p><ent>Q</ent><br/\n<hw>Q</hw> <pr>(k<umac/)</pr>, <def>the seventeenth letter of the English alphabet.</def><br/\n[<source>Am<oe/ba</source>]</p>";
        let expected = "<p><ent>Q</ent><br/\n<hw>Q</hw> <pr>(k<umac/)</pr>, <def>the seventeenth letter of the English alphabet.</def><br/\n[<source>[ERROR->]plaintext</source>]</p>";
        assert_eq!(expected, identity(block_str));
    }

    #[test]
    fn unpaired() {
        let block_str = "<p><ent>Q</ent><br/\n<hw>Q</hw> <def>here are two <i>unpaired tags</b>.</def></p>";
        let expected = "<p><ent>Q</ent><br/\n<hw>Q</hw> <def>here are two [ERROR->]<i>unpaired tags[ERROR->]</b>.</def></p>";
        assert_eq!(expected, identity(block_str));
    }
}
