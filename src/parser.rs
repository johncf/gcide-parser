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

named!(block_start<&str, &str>, take_until!("<p>"));

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Block<'a>, ParserError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok((remaining, _)) = block_start(self.contents) {
            let end_idx_opt = remaining.find("</p>");
            if end_idx_opt.is_none() {
                self.contents = "";
                return Some(Err(ParserError {
                    leading: &remaining[..0],
                    trailing: remaining,
                }));
            }
            let end_idx = end_idx_opt.unwrap() + 4;
            let block_str = &remaining[..end_idx];
            self.contents = &remaining[end_idx..];
            match parse_items(CompleteStr(block_str)) {
                Ok((unparsed, items)) => {
                    if unparsed.len() > 0 {
                        let lead_len = block_str.len() - unparsed.len();
                        Some(Err(ParserError {
                            leading: &block_str[..lead_len],
                            trailing: unparsed.0,
                        }))
                    } else {
                        Some(Ok(process_block_items(items)))
                    }
                }
                Err(_) => unreachable!(),
            }
        } else {
            None
        }
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

pub struct EntryBuilder<'a> {
    parser: Parser<'a>,
    block_buffer: Option<Result<Block<'a>, ParserError<'a>>>,
}

impl<'a> EntryBuilder<'a> {
    pub fn new(contents: &'a str) -> EntryBuilder<'a> {
        EntryBuilder { parser: Parser { contents }, block_buffer: None }
    }

    pub fn get_preface(&self) -> Option<&'a str> {
        self.parser.get_preface()
    }

    pub fn remaining(&self) -> &'a str {
        self.parser.remaining()
    }
}

impl<'a> Iterator for EntryBuilder<'a> {
    type Item = Result<Entry<'a>, BuilderError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        use self::BlockItem::*;

        let main_word;
        let mut blocks = Vec::new();
        let next_block = self.block_buffer.take().or_else(|| self.parser.next());
        match next_block {
            Some(Ok(mut block)) => {
                let mut last_ent = None;
                let mut bad_block = false;
                for (idx, item) in block.items.iter().enumerate() {
                    match *item {
                        Tagged("ent", ref ent_items) => {
                            if ent_items.len() == 1 {
                                if let Plain(text) = ent_items[0] {
                                    last_ent = Some((idx, text));
                                    continue;
                                }
                            }
                            bad_block = true;
                            break;
                        }
                        EntityBr => continue,
                        _ => break,
                    }
                }
                if bad_block {
                    return Some(Err(BuilderError::BadBlock(block)));
                }
                match last_ent {
                    Some((ent_idx, ent_text)) => {
                        if let EntityBr = block.items[ent_idx+1] {
                            block.items.drain(..ent_idx+2);
                        } else {
                            block.items.drain(..ent_idx+1);
                        }
                        blocks.push(block);
                        main_word = ent_text;
                    }
                    None => return Some(Err(BuilderError::BadBlock(block)))
                }
            }
            Some(Err(err)) => return Some(Err(BuilderError::FromParser(err))),
            None => return None,
        }
        while let Some(block_res) = self.parser.next() {
            match block_res {
                Ok(block) => {
                    if let Tagged("ent", _) = block.items[0] {
                        self.block_buffer = Some(Ok(block));
                        break;
                    } else {
                        blocks.push(block);
                    }
                }
                Err(err) => {
                    self.block_buffer = Some(Err(err));
                    break;
                }
            }
        }
        Some(Ok(Entry { main_word, blocks }))
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

#[derive(Debug)]
pub enum BuilderError<'a> {
    FromParser(ParserError<'a>),
    BadBlock(Block<'a>),
}

impl<'a> Display for BuilderError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::BuilderError::*;
        match *self {
            FromParser(parser_err) => parser_err.fmt(f),
            BadBlock(ref block) => write!(f, "[ERROR->]{}", block),
        }
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
    use super::Parser;

    fn identity(input: &str) -> String {
        use std::fmt::Write;
        let mut block_iter = Parser::new(input);
        let block_res = block_iter.next().expect("no block found!");
        assert!(block_iter.remaining().is_empty());
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
