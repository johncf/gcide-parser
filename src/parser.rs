use std::fmt::{self, Display, Formatter};

use nom::types::CompleteStr;

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
            write!(f, "<br/\n[{}]</p>\n", sources.join(" + "))
        } else {
            write!(f, "</p>\n")
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BlockItem<'a> {
    Comment(&'a str),
    Entity(&'a str),
    Plain(&'a str),
    Tagged(&'a str, Vec<BlockItem<'a>>),
    UnpairedTagOpen(&'a str),
    UnpairedTagClose(&'a str),
}

impl<'a> Display for BlockItem<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::BlockItem::*;
        match *self {
            Comment(text) => write!(f, "<--{}-->", text),
            Entity(name) => write!(f, "<{}/", name),
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

named!(parse_block<CompleteStr, Block>,
       map!(many1!(block_item), process_items));

named!(block_item<CompleteStr, BlockItem>,
       alt!(plain | open_tag | close_tag | entity | comment));

named!(plain<CompleteStr, BlockItem>,
       map!(is_not!("<>"), |s| BlockItem::Plain(s.0)));

named!(open_tag<CompleteStr, BlockItem>,
       map!(delimited!(tag!("<"), is_not!("</>"), tag!(">")), |s| BlockItem::UnpairedTagOpen(s.0)));

named!(close_tag<CompleteStr, BlockItem>,
       map!(delimited!(tag!("</"), is_not!("</>"), tag!(">")), |s| BlockItem::UnpairedTagClose(s.0)));

named!(entity<CompleteStr, BlockItem>,
       map!(delimited!(tag!("<"), is_not!("</>"), tag!("/")), |s| BlockItem::Entity(s.0)));

named!(comment<CompleteStr, BlockItem>,
       map!(delimited!(tag!("<--"), take_until!("-->"), tag!("-->")), |s| BlockItem::Comment(s.0)));

fn process_items<'a>(mut items: Vec<BlockItem<'a>>) -> Block<'a> {
    assert_eq!(items.remove(0), BlockItem::UnpairedTagOpen("p"));
    assert_eq!(items.pop(), Some(BlockItem::Plain("\n")));
    assert_eq!(items.pop(), Some(BlockItem::UnpairedTagClose("p")));
    // TODO proccess to pair up tags and take out sources
    Block {
        items: items,
        sources: Vec::new(),
    }
}

pub struct Parser<'a> {
    contents: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(contents: &'a str) -> Parser<'a> {
        Parser { contents }
    }

    pub fn remaining(&self) -> &'a str {
        self.contents
    }
}

named!(block_start<&str, &str>, take_until!("<p>"));

impl<'a> Iterator for Parser<'a> {
    type Item = (&'a str, Result<Block<'a>, &'a str>);

    fn next(&mut self) -> Option<Self::Item> {
        use nom::Err;
        use nom::simple_errors::Context;
        if let Ok((remaining, skipped)) = block_start(self.contents) {
            let end_idx = remaining.find("</p>\n").expect("block ended improperly") + 5;
            let block_str = &remaining[..end_idx];
            self.contents = &remaining[end_idx..];
            match parse_block(CompleteStr(block_str)) {
                Ok((empty, block)) => {
                    assert!(empty.is_empty());
                    Some((skipped, Ok(block)))
                }
                Err(Err::Error(Context::Code(err, _))) |
                Err(Err::Failure(Context::Code(err, _))) => {
                    Some((skipped, Err(err.0)))
                }
                Err(Err::Incomplete(_)) => {
                    Some((skipped, Err("")))
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::Parser;
    #[test]
    fn simple() {
        let block_str = "<p><ent>Q</ent><br/\n<hw>Q</hw> <pr>(k<umac/)</pr>, <def>the seventeenth letter of the English alphabet.</def><br/\n[<source>1913 Webster</source>]</p>\n";
        let mut block_iter = Parser::new(block_str);
        let mut output = String::new();
        while let Some((skipped, block)) = block_iter.next() {
            use std::fmt::Write;
            assert!(skipped.is_empty());
            let block = block.expect("bad block");
            write!(output, "{}", block).unwrap();
        }
        assert!(block_iter.remaining().is_empty());
        assert_eq!(block_str, output);
    }
}
