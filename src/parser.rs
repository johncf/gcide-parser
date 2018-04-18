use std::fmt::{self, Display, Formatter};

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

#[derive(Debug)]
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
        unimplemented!()
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

impl<'a> Iterator for Parser<'a> {
    type Item = (&'a str, Result<BlockItem<'a>, &'a str>);

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
