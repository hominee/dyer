//! attribute macro of [dyer]
//!
//! they are used to
//! - marker the role of `dyer`
//! - manipulation with boxed Future
//!
//! [dyer]: https://crates.io/crates/dyer
#![allow(dead_code)]
extern crate proc_macro;

use proc_macro::TokenStream;
use std::str::FromStr;

struct FnParser {
    pub seq: String,
    pub cursor: usize,
    pub vis: String,
    pub before_fn: String,
    pub ident: String,
    pub lifetime: Vec<String>,
    pub generic: Vec<String>,
    pub wherec: Vec<String>,
    pub args: Vec<String>,
    pub ret: String,
    pub body: String,
}

impl FnParser {
    fn new(item: String /*TokenStream*/) -> Self {
        Self {
            seq: item.to_string(),
            cursor: 0,
            vis: String::new(),
            before_fn: String::new(),
            ident: String::new(),
            lifetime: Vec::new(),
            generic: Vec::new(),
            wherec: Vec::new(),
            args: Vec::new(),
            ret: String::new(),
            body: String::new(),
        }
    }

    fn seq(&self) -> &str {
        assert!(self.seq.len() > self.cursor, "Out of range");
        &self.seq[self.cursor..]
    }

    fn next(&self, range: usize) -> Option<&str> {
        if self.seq.len() < self.cursor + range {
            return None;
        }
        Some(&self.seq()[0..range])
    }

    // non space-like char
    fn next_ascii_entity(&self) -> Option<char> {
        for c in self.seq().chars() {
            if char::is_ascii(&c) && !char::is_whitespace(c) {
                return Some(c);
            }
        }
        None
    }

    fn next_ascii_alphanumeric(&self) -> Option<char> {
        for c in self.seq().chars() {
            if char::is_ascii_alphanumeric(&c) {
                return Some(c);
            }
        }
        None
    }

    fn whitespace_rm(&mut self) {
        // remove prefixing whitespace
        while let Some(e) = self.next(1) {
            if char::is_whitespace(char::from_str(e).unwrap()) {
                self.cursor += 1;
            }
            break;
        }
    }

    fn next_seg(&mut self) -> String {
        self.whitespace_rm();
        let mut segs = String::new();
        for c in self.seq().chars() {
            if c as u8 == '_' as u8 || char::is_ascii_alphanumeric(&c) {
                segs.push(c);
            } else {
                break;
            }
        }
        self.cursor += segs.len();
        segs
    }

    fn remove_with(&mut self, pat: &str) -> Option<&str> {
        let range = pat.len();
        let cur = self.cursor;
        while let Some(s) = &self.next(range) {
            if *s == pat {
                self.cursor += range;
                return Some(&self.seq[cur..self.cursor]);
            }
            self.cursor += 1;
        }
        self.cursor = cur;
        None
    }

    fn remove_till(&mut self, pat: &str) -> Option<&str> {
        let range = pat.len();
        let cur = self.cursor;
        while let Some(s) = &self.next(range) {
            if *s == pat {
                return Some(&self.seq[cur..self.cursor]);
            }
            self.cursor += 1;
        }
        self.cursor = cur;
        None
    }

    fn advance(&mut self, range: usize) {
        self.cursor += range;
    }

    fn remove(&mut self, range: usize) -> Option<&str> {
        if range > self.seq().len() {
            return None;
        }
        self.cursor += range;
        Some(&self.seq[self.cursor - range..self.cursor])
    }

    fn get_vis(&mut self) {
        self.whitespace_rm();
        if self.seq().starts_with("pub") {
            self.vis = self
                .remove_with("pub")
                .expect("Invalid Visibility")
                .to_string();
            if self.next_ascii_entity() == Some('(') {
                let vis = self
                    .remove_with(")")
                    .expect("Invalid Visibility")
                    .to_string();
                self.vis.push_str(&vis);
            }
        }
    }

    fn before_fn(&mut self) {
        self.whitespace_rm();
        self.before_fn = self
            .remove_till("fn")
            .expect("Invalid Keyword")
            .trim()
            .to_string();
    }

    fn ident(&mut self) {
        self.whitespace_rm();
        // it must start with fn
        assert!(self.seq().starts_with("fn"));
        self.remove(2);
        self.whitespace_rm();
        self.ident = self.next_seg();
    }

    fn generic(&mut self) {
        self.whitespace_rm();
        if self.next(1) == Some("<") {
            let mut fwd = 0;
            let mut bwd = 0;
            let mut p = 0;
            let mut n = 0;
            let mut dash = 0;
            let mut cur = self.cursor;
            while let Some(e) = self.next(1) {
                if e == "<" {
                    fwd += 1;
                } else if e == ">" {
                    if fwd - bwd == 1 && p == n && dash == 0 {
                        self.cursor = cur;
                        let mut ty = self.remove_till(">").unwrap().trim().to_string();
                        if ty.starts_with("<") {
                            ty = ty.strip_prefix("<").unwrap().trim().to_string();
                        }
                        if ty.starts_with("'") {
                            // it is lifetime
                            //let ss = format!("{}: 'dyer,", ty);
                            //self.wherec.push(ss);
                            //println!("ss: {:?}", self.wherec);
                            self.lifetime.push(ty);
                        } else {
                            //let ss = format!("{}: 'dyer,", ty);
                            //self.wherec.push(ss);
                            //println!("ss: {:?}", self.wherec);
                            self.generic.push(ty);
                        }
                        self.remove_with(">");
                        cur = self.cursor;
                    }
                    bwd += 1;
                } else if e == "(" {
                    p += 1;
                } else if e == ")" {
                    if dash != 0 {
                        dash -= 1;
                        assert!(dash < 1, "more that one dash");
                    } else {
                        n += 1;
                    }
                } else if e == "-" {
                    dash += 1;
                    assert!(dash < 2, "more that one dash");
                } else if e == "," && fwd - bwd == 1 && p == n && dash == 0 {
                    self.cursor = cur;
                    let mut ty = self.remove_till(",").unwrap().trim().to_string();
                    if ty.starts_with("<") {
                        ty = ty.strip_prefix("<").unwrap().trim().to_string();
                    }
                    if ty.starts_with("'") {
                        // it is lifetime
                        //let ss = format!("{}: 'dyer,", ty);
                        //self.wherec.push(ss);
                        //println!("ss: {:?}", self.wherec);
                        self.lifetime.push(ty);
                    } else {
                        //let ss = format!("{}: 'dyer,", ty);
                        //self.wherec.push(ss);
                        //println!("ss: {:?}", self.wherec);
                        self.generic.push(ty);
                    }
                    self.remove_with(",");
                    cur = self.cursor;
                }
                self.cursor += 1;
                if fwd == bwd && bwd != 0 {
                    break;
                }
            }
        }
        if self.lifetime.is_empty() {
            self.lifetime.push("'dyer".to_string());
        }
    }

    fn args(&mut self) {
        self.whitespace_rm();
        if self.next(1) == Some("(") {
            let mut fwd = 0;
            let mut bwd = 0;
            let mut p = 0;
            let mut n = 0;
            let mut dash = 0;
            let mut cur = self.cursor;
            while let Some(e) = self.next(1) {
                if e == "(" {
                    fwd += 1;
                } else if e == ")" {
                    if fwd - bwd == 1 && p == n && dash == 0 {
                        self.cursor = cur;
                        let ty = self.remove_till(")").unwrap().trim().to_string();
                        self.args.push(ty);
                        self.remove_with(")");
                        cur = self.cursor;
                    }
                    bwd += 1;
                } else if e == "<" {
                    p += 1;
                } else if e == ">" {
                    if dash != 0 {
                        dash -= 1;
                        assert!(dash < 1, "more that one dash");
                    } else {
                        n += 1;
                    }
                } else if e == "-" {
                    dash += 1;
                    assert!(dash < 2, "more that one dash");
                } else if e == "," && fwd - bwd == 1 && p == n && dash == 0 {
                    self.cursor = cur;
                    let ty = self.remove_till(",").unwrap().trim().to_string();
                    self.args.push(ty);
                    self.remove_with(",");
                    cur = self.cursor;
                } else if e == ":" && fwd - bwd == 1 && p == n && dash == 0 {
                    self.cursor = cur;
                    let mut val = self.remove_till(":").unwrap().trim().to_string();
                    if val.starts_with("(") {
                        val = val.strip_prefix("(").unwrap().to_string();
                    }
                    self.args.push(val);
                    self.remove_with(":");
                    cur = self.cursor;
                }
                self.cursor += 1;
                if fwd == bwd && bwd != 0 {
                    break;
                }
            }
        }
    }

    fn ret(&mut self) {
        self.whitespace_rm();
        if self.next(2) == Some("->") {
            self.cursor += 2;
            let cur = self.cursor;
            let s = self
                .remove_till("{")
                .expect("Invalid Return Type")
                .trim()
                .to_string();
            if s.contains(&"where") {
                self.cursor = cur;
                self.ret = self
                    .remove_till("where")
                    .expect("missing return type")
                    .trim()
                    .to_string();
                let ss = self
                    .remove_till("{")
                    .expect("Invalid Return Type")
                    .trim()
                    .to_string();
                self.wherec.insert(0, ss);
            } else {
                self.ret = s;
            }
        }
    }

    fn body(&mut self) {
        self.body = self.seq().to_string();
    }

    fn parse(&mut self) -> TokenStream {
        self.get_vis();
        //println!("vis {}", self.vis);
        self.before_fn();
        //println!("before_fn {}", self.before_fn);
        self.ident();
        //println!("ident {}", self.ident);
        self.generic();
        let mut gen = vec!["<"];
        self.lifetime.iter().for_each(|e| {
            gen.push(e);
            gen.push(", ");
        });
        self.generic.iter().for_each(|e| {
            gen.push(e);
            gen.push(", ");
        });
        gen.push(">");
        let gen = gen.join("");
        //println!("generic {:?}", self.generic);
        self.args();
        //println!("args {:?}", self.args);
        self.ret();
        if self.ret.is_empty() {
            self.ret.insert_str(0, "()");
        }
        //println!("ret {}", self.ret);
        self.body();
        let tpl = r#"
        <+Vis+> fn <+Ident+><+Generic+><+Args+> 
            -> std::pin::Pin<Box<dyn std::future::Future<Output = <+Ret+>> + Send + <+Lf+>>> 
        <+Where+>
        {
            <+Inner+>

            std::pin::Pin::from(Box::new(<+Ident+>(<+args+>)))
        }
            "#;
        let vis = if self.vis.is_empty() {
            "pub"
        } else {
            &self.vis
        };
        let mut ind = 0;
        let mut gl = vec!["(".to_string()];
        let mut gs: Vec<&str> = Vec::new();
        for ele in self.args.iter() {
            if ind % 2 == 0 {
                gl.push(ele.to_string());
                gl.push(": ".to_string());
                let item = if ele.contains(&" ") {
                    ele.rsplitn(2, " ").collect::<Vec<_>>()[0]
                } else {
                    ele
                };
                gs.push(item);
                gs.push(", ");
            } else {
                if ele.starts_with("&") {
                    let mut f = false;
                    for c in ele.chars().skip(1) {
                        if !char::is_whitespace(c) {
                            if c == '\'' {
                                f = true;
                            }
                            break;
                        }
                    }
                    if !f {
                        let e = format!("&{}", self.lifetime[0]);
                        let v = ele.replace("&", &e);
                        gl.push(v);
                    } else {
                        gl.push(ele.to_string());
                    }
                } else {
                    gl.push(ele.to_string());
                }
                gl.push(", ".to_string());
            }
            ind += 1;
        }
        gl.push(")".to_string());
        let gl = gl.join("");
        let gs = gs.join("");
        let s = tpl
            .replace("<+Vis+>", vis)
            .replace("<+Ident+>", &self.ident)
            .replace("<+Generic+>", &gen)
            .replace("<+Args+>", &gl)
            .replace("<+args+>", &gs)
            .replace("<+Lf+>", &self.lifetime[0])
            .replace("<+Ret+>", &self.ret)
            .replace("<+Where+>", &self.wherec.join("\n    "))
            .replace("<+Inner+>", &self.seq);
        //println!("item {}", s);
        TokenStream::from_str(&s).unwrap()
    }
}

#[proc_macro_attribute]
pub fn affix(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn entity(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn middleware(_attr: TokenStream, item: TokenStream) -> TokenStream {
    FnParser::new(item.to_string()).parse()
}

#[proc_macro_attribute]
pub fn pipeline(_attr: TokenStream, item: TokenStream) -> TokenStream {
    FnParser::new(item.to_string()).parse()
}

#[proc_macro_attribute]
pub fn actor(_attr: TokenStream, mut item: TokenStream) -> TokenStream {
    if !item.to_string().trim().starts_with("pub") {
        let s = "pub ".to_string() + &item.to_string();
        item = TokenStream::from_str(&s).unwrap();
    }
    item
}

#[proc_macro_attribute]
pub fn parser(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut parse = FnParser::new(item.to_string());
    parse.get_vis();
    //println!("vis {}", self.vis);
    parse.before_fn();
    //println!("before_fn {}", self.before_fn);
    parse.ident();
    let ident = parse.ident.clone();
    parse.cursor = 0;
    let part1 = parse.remove_with("{").unwrap();
    //let part2 = parse.seq();
    let tpl = r#"<+part1+> 
    use std::sync::Once;
    static ONCE: Once = Once::new();
    unsafe {
        ONCE.call_once(|| {
            const F0: *const () = <+ident+> as *const ();
            const I0: &str = "<+ident+>";
            FNMAP.push((I0, F0));
        });
    }

    <+part2+>
"#;
    let s = tpl
        .replace("<+part1+>", part1)
        .replace("<+part2+>", parse.seq())
        .replace("<+ident+>", &ident);
    //println!("item: {:?}", s);
    TokenStream::from_str(&s).unwrap()
}

#[test]
fn test_fnparser() {
    let s = "pub async fn func<'r, E>(arg:&'r mut Vec<E>) -> Option<(u32, String)> { None }";
    let mut parser = FnParser::new(s.into());
    assert_eq!(Some("pub"), parser.remove(3));
    assert_eq!(parser.remove(6), Some(" async"));
    let mut parser = FnParser::new(s.into());
    assert_eq!(Some("pub async "), parser.remove_till("fn "));
    assert_eq!(parser.remove(3), Some("fn "));
    let mut parser = FnParser::new(s.into());
    assert_eq!(Some("pub async fn "), parser.remove_with("fn "));
    assert_eq!("func".to_string(), parser.next_seg());
    //assert_eq!(parser.remove(4), Some("func"));
    let s = "pub(crate) async fn func<'r, E, F:Vec<E>>(arg:&'r mut Vec<E>) -> Option<(u32, String)> { None }";
    let mut parser = FnParser::new(s.into());
    parser.get_vis();
    assert_eq!("pub(crate)".to_string(), parser.vis);
    parser.before_fn();
    assert_eq!("async".to_string(), parser.before_fn);
    parser.ident();
    assert_eq!("func".to_string(), parser.ident);
    parser.generic();
    assert_eq!("'r".to_string(), parser.generic[0]);
    assert_eq!("E".to_string(), parser.generic[1]);
    assert_eq!("F:Vec<E>".to_string(), parser.generic[3]);
    parser.args();
    assert_eq!("arg".to_string(), parser.args[0]);
    assert_eq!("&'r mut Vec<E>".to_string(), parser.args[1]);
    parser.ret();
    assert_eq!("Option<(u32, String)>".to_string(), parser.ret);
    parser.body();
    assert_eq!("{ None }".to_string(), parser.body);
    let s = "pub(crate) async fn func<'r, E, F:Vec<E>, F: Fn() -> (Vec<>, i128)>(arg:&'r mut Vec<E>, app: App<E>) -> Option<(u32, String)> { None }";
    let mut parser = FnParser::new(s.into());
    parser.parse();
    assert_eq!("arg".to_string(), parser.args[0]);
    assert_eq!("&'r mut Vec<E>".to_string(), parser.args[1]);
    assert_eq!("app".to_string(), parser.args[2]);
    assert_eq!("App<E>".to_string(), parser.args[3]);
    assert_eq!(
        "<'r, E, F:Vec<E>, F: Fn() -> (Vec<>, i128)>".to_string(),
        parser.generic
    );
}
