use std::fmt::{self, Display, Formatter};

use nom::types::CompleteStr;
use nom::{alphanumeric1, self};

#[derive(Debug)]
pub struct Entry<'a> {
    pub main_word: &'a str,
    pub items: Vec<EntryItem<'a>>,
    pub source: &'a str,
}

#[derive(Debug, PartialEq)]
pub enum EntryItem<'a> {
    Tagged { name: &'a str, items: Vec<EntryItem<'a>>, source: Option<&'a str> },
    Comment(&'a str),
    Entity(&'a str),
    EntityBr,
    EntityUnk,
    ExternalLink(&'a str, &'a str),
    PlainText(&'a str),
    UnpairedTagOpen(&'a str, Option<&'a str>),
    UnpairedTagClose(&'a str),
}

named!(parse_items<CompleteStr, Vec<EntryItem>>, many0!(entry_item));

named!(entry_item<CompleteStr, EntryItem>,
       alt!(plain_text | open_tag | close_tag | entity | comment | ext_link));

named!(plain_text<CompleteStr, EntryItem>,
       map!(is_not!("<>"), |s| EntryItem::PlainText(s.0)));

named!(source_attr<CompleteStr, CompleteStr>,
       delimited!(tag!(" source=\""), take_till!(|c| c == '"'), tag!("\"")));

named!(open_tag<CompleteStr, EntryItem>,
       do_parse!(
           tag!("<") >>
           name: alphanumeric1 >>
           source: opt!(source_attr) >>
           tag!(">") >>
           ( EntryItem::UnpairedTagOpen(name.0, source.map(|s| s.0)) )));

named!(close_tag<CompleteStr, EntryItem>,
       map!(delimited!(tag!("</"), alphanumeric1, tag!(">")), |s| EntryItem::UnpairedTagClose(s.0)));

named!(entity<CompleteStr, EntryItem>,
       alt!(map!(tag!("<?/"), |_| EntryItem::EntityUnk) |
            map!(tuple!(tag!("<br/"), opt!(char!('\n'))), |_| EntryItem::EntityBr) |
            map!(delimited!(tag!("<"), alphanumeric1, tag!("/")), |s| EntryItem::Entity(s.0))));

named!(comment<CompleteStr, EntryItem>,
       map!(delimited!(tag!("<--"), take_until!("-->"), tag!("-->")), |s| EntryItem::Comment(s.0)));

named!(ext_link<CompleteStr, EntryItem>,
       do_parse!(
           tag!("<a href=\"") >>
           url: take_till!(|c| c == '"') >>
           tag!("\">") >>
           text: is_not!("<>") >>
           tag!("</a>") >>
           ( EntryItem::ExternalLink(url.0, text.0) )));

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
                Ok((entry_str, EntryHead { main_word, source })) => {
                    match parse_items(CompleteStr(entry_str)) {
                        Ok((unparsed, items)) => {
                            if unparsed.len() > 0 {
                                let lead_len = end_idx - unparsed.len();
                                Err(ParserError {
                                    leading: &remaining[..lead_len],
                                    trailing: &remaining[lead_len..end_idx + close_len],
                                })
                            } else {
                                Ok(Entry {
                                    main_word: main_word,
                                    items: pair_up_items(items),
                                    source: source,
                                })
                            }
                        }
                        Err(_) => unreachable!(),
                    }
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

fn pair_up_items<'a>(items: Vec<EntryItem<'a>>) -> Vec<EntryItem<'a>> {
    use self::EntryItem::*;

    let mut stack = Vec::with_capacity(items.len()*2/3 + 1);
    for item in items {
        match item {
            UnpairedTagClose(name) => {
                let is_tag_open_name = |item: &EntryItem<'a>| {
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
