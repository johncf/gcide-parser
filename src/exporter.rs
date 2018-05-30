use std::fmt::{self, Display, Formatter};

use parser::{Entry, EntryItem, GreekItem, GreekMods};

use unicode_normalization::char::compose as unic_compose;

pub struct CIDE<'a>(pub &'a Entry<'a>);

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
            Greek(ref gitems) => {
                write!(f, "<grk>")?;
                for gi in gitems {
                    gi.fmt_cide(f)?;
                }
                write!(f, "</grk>")
            }
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

impl DisplayCIDE for GreekItem {
    fn fmt_cide(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            GreekItem::Letter(base, mods) => {
                if mods.contains(GreekMods::SLENIS) {
                    write!(f, "'")?;
                } else if mods.contains(GreekMods::SASPER) {
                    write!(f, "\"")?;
                }
                write!(f, "{}", base)?;
                if mods.contains(GreekMods::DIAERESIS) {
                    write!(f, ":")?;
                }
                if mods.contains(GreekMods::ACUTE) {
                    write!(f, "`")?;
                } else if mods.contains(GreekMods::GRAVE) {
                    write!(f, "~")?;
                } else if mods.contains(GreekMods::CIRCUMFLEX) {
                    write!(f, "^")?;
                }
                if mods.contains(GreekMods::IOTASUB) {
                    write!(f, ",")?;
                }
                Ok(())
            }
            GreekItem::Other(c) => write!(f, "{}", c),
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

impl<'a> Display for EntryItem<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use parser::EntryItem::*;
        use std::fmt::Write;
        match *self {
            Entity(name) => f.write_str(entity_to_unicode(name)),
            EntityBr => f.write_char('\n'),
            EntityUnk => f.write_char('\u{fffd}'),
            ExternalLink(_, text) => f.write_str(text),
            Greek(ref gitems) => {
                for gi in gitems {
                    gi.fmt(f)?;
                }
                Ok(())
            }
            PlainText(text) => f.write_str(&process_symbols_in_text(text)),
            Tagged { ref items, .. } => {
                for item in items {
                    item.fmt(f)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

pub fn process_symbols_in_text(text: &str) -> String {
    text.replace("'", "\u{2019}")
        .replace("----", "\u{23af}\u{23af}\u{23af}")
        .replace("--", entity_to_unicode("mdash"))
}

impl Display for GreekItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use std::fmt::Write;
        match *self {
            GreekItem::Letter(base, mods) => {
                let mut letter = Some(grktrans_to_unicode(base, mods.contains(GreekMods::TERMINAL)));
                let compose = |l_opt: Option<char>, m| l_opt.and_then(|l| unic_compose(l, m));
                if mods.contains(GreekMods::SLENIS) {
                    letter = compose(letter, '\u{0313}');
                } else if mods.contains(GreekMods::SASPER) {
                    letter = compose(letter, '\u{0314}');
                }
                if mods.contains(GreekMods::DIAERESIS) {
                    letter = compose(letter, '\u{0308}');
                }
                if mods.contains(GreekMods::ACUTE) {
                    letter = compose(letter, '\u{0301}');
                } else if mods.contains(GreekMods::GRAVE) {
                    letter = compose(letter, '\u{0300}');
                } else if mods.contains(GreekMods::CIRCUMFLEX) {
                    letter = compose(letter, '\u{0342}');
                }
                if mods.contains(GreekMods::IOTASUB) {
                    letter = compose(letter, '\u{0345}');
                }
                match letter {
                    Some(c) => f.write_char(c),
                    None => {
                        eprintln!("possibly bad greek letter: {} {:b}", base, mods);
                        f.write_char('\u{fffd}')
                    }
                }
            }
            GreekItem::Other(c) => f.write_char(c),
        }
    }
}

pub fn entity_to_unicode(entity: &str) -> &'static str {
    match entity {
        "lt"       => "<",
        "gt"       => ">",
        "ait"     => "a",
        "eit"     => "e",
        "iit"     => "i",
        "oit"     => "o",
        "uit"     => "u",
        "ae"       => "\u{00e6}",
        "AE"       => "\u{00c6}",
        "oe"       => "\u{0153}",
        "OE"       => "\u{0152}",
        "cced"     => "\u{00e7}",
        "aring"    => "\u{00e5}",
        "uring"    => "\u{016f}",
        "aacute"   => "\u{00e1}",
        "eacute"   => "\u{00e9}",
        "iacute"   => "\u{00ed}",
        "oacute"   => "\u{00f3}",
        "uacute"   => "\u{00fa}",
        "Eacute"   => "\u{00c9}",
        "acir"     => "\u{00e2}",
        "ecir"     => "\u{00ea}",
        "icir"     => "\u{00ee}",
        "ocir"     => "\u{00f4}",
        "ucir"     => "\u{00fb}",
        "agrave"   => "\u{00e0}",
        "egrave"   => "\u{00e8}",
        "igrave"   => "\u{00ec}",
        "ograve"   => "\u{00f2}",
        "ugrave"   => "\u{00f9}",
        "aum"      => "\u{00e4}",
        "eum"      => "\u{00eb}",
        "ium"      => "\u{00ef}",
        "oum"      => "\u{00f6}",
        "uum"      => "\u{00fc}",
        "atil"     => "\u{00e3}",
        "etil"     => "\u{1ebd}",
        "ltil"     => "l\u{0303}",
        "mtil"     => "m\u{0303}",
        "ntil"     => "\u{00f1}",
        "amac"     => "\u{0101}",
        "emac"     => "\u{0113}",
        "imac"     => "\u{012b}",
        "omac"     => "\u{014d}",
        "umac"     => "\u{016b}",
        "ymac"     => "\u{0233}",
        "aemac"    => "\u{01e3}",
        "oomac"    => "o\u{035e}o",
        "acr"      => "\u{0103}",
        "ecr"      => "\u{0115}",
        "icr"      => "\u{012d}",
        "ocr"      => "\u{014f}",
        "ucr"      => "\u{016d}",
        "ycr"      => "y\u{0306}",
        "oocr"     => "o\u{035d}o",
        "ocar"     => "\u{01d2}",
        "asl"      => "a\u{0304}\u{0307}",
        "esl"      => "e\u{0304}\u{0307}",
        "isl"      => "i\u{0304}\u{0307}",
        "osl"      => "o\u{0304}\u{0307}",
        "usl"      => "u\u{0304}\u{0307}",
        "adot"     => "\u{0227}",
        "ndot"     => "\u{1e45}",
        "dsdot"    => "\u{1e0d}",
        "nsdot"    => "\u{1e47}",
        "rsdot"    => "\u{1e5b}",
        "tsdot"    => "\u{1e6d}",
        "usdot"    => "\u{1ee5}",
        "add"      => "a\u{0324}",
        "udd"      => "\u{1e73}",
        "nsm"      => "\u{1e49}",
        "hand"     => "\u{261e}",
        "deg"      => "\u{00b0}",
        "prime"    => "\u{2032}",
        "dprime"   => "\u{2033}",
        "ldquo"    => "\u{201c}",
        "rdquo"    => "\u{201d}",
        "lsquo"    => "\u{2018}",
        "rsquo"    => "\u{2019}",
        "sect"     => "\u{00a7}",
        "sharp"    => "\u{266f}",
        "flat"     => "\u{266d}",
        "pound"    => "\u{00a3}",
        "minus"    => "\u{2212}",
        "mdash"    => "\u{2014}",
        "th"       => "t\u{035f}h",
        "par"      => "\u{2016}",
        "cre"      => "\u{2323}",
        "edh"      => "\u{00f0}",
        "thorn"    => "\u{00fe}",
        "yogh"     => "\u{021d}",
        "divide"   => "\u{00f7}",
        "times"    => "\u{00d7}",
        "rarr"     => "\u{2192}",
        "middot"   => "\u{00b7}",
        "root"     => "\u{221a}",
        "cuberoot" => "\u{221b}",
        "alpha"    => "\u{03b1}",
        "beta"     => "\u{03b2}",
        "gamma"    => "\u{03b3}",
        "GAMMA"    => "\u{0393}",
        "delta"    => "\u{03b4}",
        "DELTA"    => "\u{0394}",
        "epsilon"  => "\u{03b5}",
        "zeta"     => "\u{03b6}",
        "eta"      => "\u{03b7}",
        "theta"    => "\u{03b8}",
        "THETA"    => "\u{0398}",
        "iota"     => "\u{03b9}",
        "kappa"    => "\u{03ba}",
        "lambda"   => "\u{03bb}",
        "LAMBDA"   => "\u{039b}",
        "mu"       => "\u{03bc}",
        "nu"       => "\u{03bd}",
        "xi"       => "\u{03be}",
        "XI"       => "\u{039e}",
        "omicron"  => "\u{03bf}",
        "pi"       => "\u{03c0}",
        "PI"       => "\u{03a0}",
        "rho"      => "\u{03c1}",
        "sigma"    => "\u{03c3}",
        "sigmat"   => "\u{03c2}",
        "SIGMA"    => "\u{03a3}",
        "tau"      => "\u{03c4}",
        "upsilon"  => "\u{03c5}",
        "phi"      => "\u{03c6}",
        "PHI"      => "\u{03a6}",
        "chi"      => "\u{03c7}",
        "psi"      => "\u{03c8}",
        "PSI"      => "\u{03a8}",
        "omega"    => "\u{03c9}",
        "OMEGA"    => "\u{03a9}",
        "acute"    => "\u{00b4}",
        "grave"    => "`",
        "star"     => "*",
        "asterism" => "\u{2042}",
        "cflex"    => "\u{02c6}",
        "srtil"    => "\u{02dc}",
        "invbre"   => " \u{0311}",
        "bacc"     => "\u{02c8}",
        "lacc"     => "\u{02cc}",
        "sdiv"     => "\u{00b7}",
        "tsup"     => "\u{1d57}",
        "esup"     => "\u{1d49}",
        "isub"     => "\u{1d62}",
        _          => "\u{fffd}"
    }
}

/// Transcribed Greek in ASCII (per GCIDE spec) to Unicode Greek character.
pub fn grktrans_to_unicode(trans: char, is_terminal: bool) -> char {
    match trans {
        'a' => '\u{03b1}', 'b' => '\u{03b2}',
        'g' => '\u{03b3}', 'd' => '\u{03b4}',
        'e' => '\u{03b5}', 'z' => '\u{03b6}',
        'h' => '\u{03b7}', 'q' => '\u{03b8}',
        'i' => '\u{03b9}', 'k' => '\u{03ba}',
        'l' => '\u{03bb}', 'm' => '\u{03bc}',
        'n' => '\u{03bd}', 'x' => '\u{03be}',
        'o' => '\u{03bf}', 'p' => '\u{03c0}',
        'r' => '\u{03c1}', 's' => if is_terminal { '\u{03c2}' } else { '\u{03c3}' },
        't' => '\u{03c4}', 'y' => '\u{03c5}',
        'f' => '\u{03c6}', 'c' => '\u{03c7}',
        'j' => '\u{03c8}', 'w' => '\u{03c9}',
        'v' => '\u{03dd}',
        'A' => '\u{0391}', 'B' => '\u{0392}',
        'G' => '\u{0393}', 'D' => '\u{0394}',
        'E' => '\u{0395}', 'Z' => '\u{0396}',
        'H' => '\u{0397}', 'Q' => '\u{0398}',
        'I' => '\u{0399}', 'K' => '\u{039a}',
        'L' => '\u{039b}', 'M' => '\u{039c}',
        'N' => '\u{039d}', 'X' => '\u{039e}',
        'O' => '\u{039f}', 'P' => '\u{03a0}',
        'R' => '\u{03a1}', 'S' => '\u{03a3}',
        'T' => '\u{03a4}', 'Y' => '\u{03a5}',
        'F' => '\u{03a6}', 'C' => '\u{03a7}',
        'J' => '\u{03a8}', 'W' => '\u{03a9}',
        _   => '\u{fffd}',
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
