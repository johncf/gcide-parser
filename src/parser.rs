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
        write!(f, "\n<p>")?;
        for t in &self.items {
            write!(f, "{}", t)?;
        }
        if self.sources.len() > 0 {
            let mut sources = Vec::new();
            for s in &self.sources {
                sources.push(format!("<source>{}</source>", s));
            }
            write!(f, "<br/\n[{}]</p>\n", sources.join(" + "))
        } else {
            write!(f, "</p>\n")
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BlockItem<'a> {
    Tagged(&'a str, Vec<BlockItem<'a>>),
    Comment(&'a str),
    Entity(&'a str),
    EntityUnk,
    ExternalLink(&'a str, &'a str),
    Plain(&'a str),
    UnpairedTagOpen(&'a str),
    UnpairedTagClose(&'a str),
}

impl<'a> Display for BlockItem<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::BlockItem::*;
        match *self {
            Comment(text) => write!(f, "<--{}-->", text),
            Entity(name) => write!(f, "<{}/", name),
            EntityUnk => write!(f, "<?/"),
            ExternalLink(url, text) => write!(f, "<a href=\"{}\">{}</a>", url, text),
            Plain(text) => write!(f, "{}", text),
            Tagged(name, ref items) => {
                write!(f, "<{}>", name)?;
                for item in items {
                    write!(f, "{}", item)?;
                }
                write!(f, "</{}>", name)
            }
            UnpairedTagOpen(name) => write!(f, "<{}>", name),
            UnpairedTagClose(name) => write!(f, "</{}>", name),
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

pub struct ParserError<'a> {
    pub leading: &'a str,
    pub trailing: &'a str,
}

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

fn pair_up_items<'a>(mut items: Vec<BlockItem<'a>>) -> Vec<BlockItem<'a>> {
    // TODO proccess to pair up tags
    items
}

fn process_block_items<'a>(mut items: Vec<BlockItem<'a>>) -> Block<'a> {
    assert_eq!(items.remove(0), BlockItem::UnpairedTagOpen("p"));
    assert_eq!(items.pop(), Some(BlockItem::UnpairedTagClose("p")));
    // TODO take out sources
    Block {
        items: items,
        sources: Vec::new(),
    }
}

#[cfg(test)]
mod test {
    use super::Parser;
    #[test]
    fn simple() {
        let block_str = "<p><ent>Q</ent><br/\n<hw>Q</hw> <pr>(k<umac/)</pr>, <def>the seventeenth letter of the English alphabet.</def><br/\n[<source>1913 Webster</source>]</p>";
        let mut block_iter = Parser::new(block_str);
        let mut output = String::new();
        while let Some((skipped, block)) = block_iter.next() {
            use std::fmt::Write;
            assert!(skipped.is_empty());
            let block = block.expect("bad block");
            write!(output, "{}", block).unwrap();
        }
        assert!(block_iter.remaining().is_empty());
        assert_eq!(block_str, output.trim());
    }
}
