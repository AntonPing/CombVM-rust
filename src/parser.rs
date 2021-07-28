use crate::TermRef;
use crate::term::*;
use crate::term::Term::*;

extern crate regex;
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Debug)]
pub struct Parser {
    text: String,
    index: usize,
}

use crate::symbol::*;

impl Parser {
    pub fn new(str: String) -> Parser {
        Parser { text: str, index: 0 }
    }
    pub fn is_end(&mut self) -> Option<()> {
        assert!(self.index <= self.text.len());
        if self.index == self.text.len() { Some(()) }
        else { None }
    }
    pub fn skip_space(&mut self) {
        assert!(self.index <= self.text.len());
        loop {
            if self.index == self.text.len() {
                break;  
            }
            match self.text.as_bytes()[self.index] {
                b' ' | b'\n' | b'\r' | b'\t' => {
                    self.index += 1;
                }
                _ => { break; }
            }
        }
    }
    pub fn read_regex(&mut self, regex: &Regex) -> Option<&str> {
        let mat = regex.find(&self.text[self.index..])?;
        assert_eq!(mat.start(), 0);
        self.index += mat.end();
        Some(mat.as_str())
    }
    pub fn read_string(&mut self, string: &str) -> Option<()> {
        let new_index = self.index + string.len();
        if new_index > self.text.len() {
            return None;
        }
        let cut = &self.text[self.index..new_index];
        if string == cut {
            self.index = new_index;
            Some(())
        } else {
            None
        }
    }
    pub fn try_read<T>(&mut self,
                func: fn(&mut Parser)->Option<T>) -> Option<T> {
        let record = self.index;
        if let Some(value) = func(self) {
            return Some(value);
        } else {
            self.index = record;
            return None;
        }
    }
    pub fn try_read_many<T>(&mut self,
                funcs: Vec<fn(&mut Parser)->Option<T>>) -> Option<T> {
        let record = self.index;
        for func in funcs.iter() {
            if let Some(value) = func(self) {
                return Some(value);
            } else {
                self.index = record;
            }
        }
        None
    }
    /*
    pub fn try_peek<T>(&mut self,
        func: fn(&mut Parser)->Option<T>) -> Option<T> {
        let record = self.index;
        if let Some(value) = func(self) {
            self.index = record;
            return Some(value);
        } else {
            self.index = record;
            return None;
        }
    }
    */
    pub fn get_rest(&mut self) -> String {
        String::from(&self.text[self.index..])
    }
}

lazy_static::lazy_static! {
    static ref CHAR_RE: Regex = Regex::new(
        r"^.").unwrap();
    static ref SPACE_RE: Regex = Regex::new(
        r"^[\s\t\n]*").unwrap();
    static ref INT_RE: Regex = Regex::new(
        r"^\d+").unwrap();
    static ref SYMB_RE: Regex = Regex::new(
        r"^[_A-Za-z][_A-Za-z0-9]*").unwrap();
    static ref PATH_RE: Regex = Regex::new(
        r"^.+").unwrap();
}

pub fn read_int(par: &mut Parser) -> Option<i64> {
    par.try_read(|p|{
        let string = p.read_regex(&*INT_RE)?;
        let result = string.parse::<i64>().ok()?;
        Some(result)
    })
}
pub fn read_symb(par: &mut Parser) -> Option<Symb> {
    par.try_read(|p|{
        let string = p.read_regex(&*SYMB_RE)?;
        Some(Symb::new(string))
    })
}

pub fn read_term(par: &mut Parser) -> Option<TermRef> {
    par.try_read_many(vec![
        |p|{ read_const_func(p) },
        |p|{
            let value = read_int(p)?;
            Some(int!(value))
        },
        |p|{ read_var(p) },
        |p|{ read_lam(p) },
        |p|{ read_app(p) }
    ])
}
pub fn read_var(par: &mut Parser) -> Option<TermRef> {
    par.try_read(|p|{
        let string = p.read_regex(&*SYMB_RE)?;
        Some(var!(Symb::new(string)))
    })
}
pub fn read_lam(par: &mut Parser) -> Option<TermRef> {
    par.try_read(|p|{
        p.read_string("\\")?;
        p.skip_space();
        let x = read_symb(p)?;
        p.skip_space();
        p.read_string(".")?;
        p.skip_space();
        let t = read_app_list(p)?;
        Some(lam!(x,t))
    })
}
pub fn read_app(par: &mut Parser) -> Option<TermRef> {
    par.try_read(|p|{
        p.read_string("(")?;
        p.skip_space();
        let t = read_app_list(p)?;
        p.skip_space();
        p.read_string(")")?;
        Some(t)
    })
}

pub fn read_app_list(par: &mut Parser) -> Option<TermRef> {
    par.try_read(|p|{
        let mut t1 = read_term(p)?;
        p.skip_space();
        loop {
            if let Some(t2) = read_term(p) {
                t1 = app!(t1,t2);
                p.skip_space();
            } else if let Some(()) = p.read_string(";") {
                p.skip_space();
                let list = read_app_list(p)?;
                t1 = app!(t1,list);
                p.skip_space();
            } else {
                break;
            }
        }
        Some(t1)
    })
}

pub fn read_const_func(par: &mut Parser) -> Option<TermRef> {
    macro_rules! const_parser {
        ($term:expr, $str:expr) => {
            |p| {
                p.read_string($str)?;
                Some($term)
            }
        };
    }
    par.try_read_many(vec![
        const_parser!(C_I,"I"),
        const_parser!(C_K,"K"),
        const_parser!(C_S,"S"),
        const_parser!(C_B,"B"),
        const_parser!(C_C,"C"),
        const_parser!(C_SP,"S'"),
        const_parser!(C_BS,"B*"),
        const_parser!(C_CP,"C'"),
        const_parser!(C_E1,"E1"),
        const_parser!(C_E1,"E2"),
        const_parser!(C_E1,"E3"),
        const_parser!(app!(C_E2,C_ADDI),"+"),
        const_parser!(app!(C_E2,C_SUBI),"-"),
        const_parser!(app!(C_E2,C_MULI),"*"),
        const_parser!(app!(C_E2,C_DIVI),"/"),
        const_parser!(app!(C_E2,C_GRTI),">"),
        const_parser!(app!(C_E2,C_LSSI),"<"),
        const_parser!(app!(C_E2,C_EQLI),"="),
        const_parser!(app!(C_E2,C_NOT),"not"),
        const_parser!(app!(C_E2,C_AND),"and"),
        const_parser!(app!(C_E2,C_OR),"or"),
        const_parser!(app!(C_E1,C_IFTE),"if"),
    ])
}

pub fn read_path(par: &mut Parser) -> Option<String> {
    par.try_read(|p|{
        let string = p.read_regex(&*PATH_RE)?;
        Some(string.to_string())
    })
}


pub enum Command {
    Quit,Dict,
    Define(Symb,String),
    Update(Symb,String),
    Delete(Symb),
    Load(String),
    Repl(TermRef),
}

pub fn read_command(par: &mut Parser) -> Option<Command> {
    par.try_read_many(vec![
        |p| {
            if(p.read_string(":").is_some()) {
                return None;
            } else {
                p.skip_space();
                let term = read_app_list(p)?;
                p.skip_space();
                p.is_end()?;
                Some(Command::Repl(term))
            }
        },
        |p|{
            p.read_string(":quit")?;
            p.skip_space();
            p.is_end()?;
            Some(Command::Quit)
        },
        |p|{
            p.read_string(":dict")?;
            p.skip_space();
            p.is_end()?;
            Some(Command::Dict)
        },
        |p|{
            p.read_string(":define")?;
            p.skip_space();
            let symb = read_symb(p)?;
            p.skip_space();
            let input = p.get_rest();
            Some(Command::Define(symb,input))
        },
        |p|{
            p.read_string(":update")?;
            p.skip_space();
            let symb = read_symb(p)?;
            p.skip_space();
            let input = p.get_rest();
            Some(Command::Update(symb,input))
        },
        |p|{
            p.read_string(":delete")?;
            p.skip_space();
            let symb = read_symb(p)?;
            p.skip_space();
            p.is_end()?;
            Some(Command::Delete(symb))
        },
        |p|{
            p.read_string(":load")?;
            p.skip_space();
            let path = read_path(p)?;
            p.skip_space();
            p.is_end()?;
            Some(Command::Load(path))
        },
    ])
}

pub fn parse_term(input: &str) -> Option<TermRef> {
    let mut par = Parser::new(String::from(input));
    let term = read_term(&mut par)?;    
    Some(term)
}