use std::fmt::{self, Display, Formatter};

use parser::{Entry, EntryItem};

pub struct CIDE<'a>(pub &'a Entry<'a>);
pub struct HTML<'a>(pub &'a Entry<'a>);
//pub struct Plain<'a>(pub &'a Entry<'a>);

trait DisplayCIDE {
    fn fmt_cide(&self, f: &mut Formatter) -> fmt::Result;
}

trait DisplayHTML {
    fn fmt_html(&self, f: &mut Formatter) -> fmt::Result;
}

impl<'a> Display for CIDE<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt_cide(f)
    }
}

impl<'a> Display for HTML<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt_html(f)
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

impl<'a> DisplayHTML for Entry<'a> {
    fn fmt_html(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<div class=\"entry\" data-word=\"{}\" data-source=\"{}\">", self.main_word, self.source)?;
        for item in &self.items {
            item.fmt_html(f)?;
        }
        write!(f, "</div>")
    }
}

impl<'a> DisplayHTML for EntryItem<'a> {
    fn fmt_html(&self, f: &mut Formatter) -> fmt::Result {
        use parser::EntryItem::*;
        match *self {
            Comment(_) => Ok(()),
            Entity(name) => write!(f, "{}", entity_to_html(name)),
            EntityBr => write!(f, "<br/>\n"),
            EntityUnk => write!(f, "&#xfffd;"),
            ExternalLink(url, text) => write!(f, "<a class=\"extern\" href=\"{}\">{}</a>", url, text),
            Greek(_) => unimplemented!(), // TODO
            PlainText(text) => write!(f, "{}", text.replace("&", "&amp;")), // TODO dashes
            Tagged { name, ref items, source } => {
                match name {
                    "p" => {
                        match source {
                            Some(source) => write!(f, "<p data-source=\"{}\">", source)?,
                            None => write!(f, "<p>")?,
                        }
                        fmt_html(f, items)?;
                        write!(f, "</p>")
                    }
                    "grk" => unimplemented!(),
                    _ => write!(f, "&#xfffd;"),

                }
            }
            UnpairedTagOpen(_, _) => Ok(()),
            UnpairedTagClose(_) => Ok(()),
        }
    }
}

fn fmt_html(f: &mut Formatter, items: &Vec<EntryItem>) -> fmt::Result {
    for item in items {
        item.fmt_cide(f)?;
    }
    Ok(())
}

fn entity_to_html(entity: &str) -> &'static str {
    match entity {
        "lt"       => "&lt;",
        "gt"       => "&gt;",
        "ae"       => "&aelig;",
        "AE"       => "&AElig;",
        "oe"       => "&oelig;",
        "OE"       => "&OElig;",
        "cced"     => "&ccedil;",
        "aring"    => "&aring;",
        "aacute"   => "&aacute;",
        "eacute"   => "&eacute;",
        "iacute"   => "&iacute;",
        "oacute"   => "&oacute;",
        "uacute"   => "&uacute;",
        "Eacute"   => "&Eacute;",
        "acir"     => "&acirc;",
        "ecir"     => "&ecirc;",
        "icir"     => "&icirc;",
        "ocir"     => "&ocirc;",
        "ucir"     => "&ucirc;",
        "agrave"   => "&agrave;",
        "egrave"   => "&egrave;",
        "igrave"   => "&igrave;",
        "ograve"   => "&ograve;",
        "ugrave"   => "&ugrave;",
        "aum"      => "&auml;",
        "eum"      => "&euml;",
        "ium"      => "&iuml;",
        "oum"      => "&ouml;",
        "uum"      => "&uuml;",
        "atil"     => "&atilde;",
        "ntil"     => "&ntilde;",
        "frac12"   => "&frac12;",
        "frac14"   => "&frac14;",
        "deg"      => "&deg;",
        "prime"    => "&prime;",
        "dprime"   => "&Prime;",
        "ldquo"    => "&ldquo;",
        "rdquo"    => "&rdquo;",
        "lsquo"    => "&lsquo;",
        "rsquo"    => "&rsquo;",
        "sect"     => "&sect;",
        "pound"    => "&pound;",
        "mdash"    => "&mdash;",
        "edh"      => "&eth;",
        "thorn"    => "&thorn;",
        "divide"   => "&divide;",
        "times"    => "&times;",
        "rarr"     => "&rarr;",
        "middot"   => "&middot;",
        "root"     => "&radic;",
        "alpha"    => "&alpha;",
        "beta"     => "&beta;",
        "gamma"    => "&gamma;",
        "GAMMA"    => "&Gamma;",
        "delta"    => "&delta;",
        "DELTA"    => "&Delta;",
        "epsilon"  => "&epsilon;",
        "zeta"     => "&zeta;",
        "eta"      => "&eta;",
        "theta"    => "&theta;",
        "THETA"    => "&Theta;",
        "iota"     => "&iota;",
        "kappa"    => "&kappa;",
        "lambda"   => "&lambda;",
        "LAMBDA"   => "&Lambda;",
        "mu"       => "&mu;",
        "nu"       => "&nu;",
        "xi"       => "&xi;",
        "XI"       => "&Xi;",
        "omicron"  => "&omicron;",
        "pi"       => "&pi;",
        "PI"       => "&Pi;",
        "rho"      => "&rho;",
        "sigma"    => "&sigma;",
        "sigmat"   => "&sigmaf;",
        "SIGMA"    => "&Sigma;",
        "tau"      => "&tau;",
        "upsilon"  => "&upsilon;",
        "phi"      => "&phi;",
        "PHI"      => "&Phi;",
        "chi"      => "&chi;",
        "psi"      => "&psi;",
        "PSI"      => "&Psi;",
        "omega"    => "&omega;",
        "OMEGA"    => "&Omega;",
        "acute"    => "&acute;",
        "cflex"    => "&circ;",
        "srtil"    => "&tilde;",
        _          => entity_to_unicode(entity),
    }
}

fn entity_to_unicode(entity: &str) -> &'static str {
    match entity {
        "lt"       => "<",
        "gt"       => ">",
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
        "frac12"   => "\u{00bd}",
        "frac14"   => "\u{00bc}",
        "frac13"   => "\u{2153}",
        "frac23"   => "\u{2154}",
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
