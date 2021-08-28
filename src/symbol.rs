use std::fmt;
use std::fmt::Debug;
use std::sync::Mutex;

extern crate rand;
use rand::Rng;

use bimap::BiMap;
use std::collections::HashMap;

use crate::term;
use crate::term::TermRef;
use crate::parser;
use crate::compile;

lazy_static::lazy_static! {
    static ref SYMB_MAP: Mutex<BiMap<u32,String>> = 
                            Mutex::new(BiMap::new());
    pub static ref DICT_MAP: Mutex<HashMap<Symb,DictValue>> = 
                            Mutex::new(HashMap::new());
}

#[derive(Eq,Clone,Copy,Hash)]
pub struct Symb(u32);

impl PartialEq for Symb {
    fn eq(&self, other: &Symb) -> bool {
        self.0 == other.0
    }
}

impl Symb {
    pub fn new(right: &str) -> Symb {
        let mut map = SYMB_MAP.lock().unwrap();
        if let Some(left) = map.get_by_right(&right.to_string()) {
            return Symb(*left);
        } else {
            let mut rnd: u32 = rand::thread_rng().gen();
            while map.contains_left(&rnd) {
                rnd = rand::thread_rng().gen();
            }
            map.insert(rnd,right.to_string());
            return Symb(rnd);
        }
    }
    fn str(&self) -> String {
        let map = SYMB_MAP.lock().unwrap();
        if let Some(right) = map.get_by_left(&self.0) {
            return right.clone();
        } else {
            panic!("can't find Symbol");
        }
    }
}

impl Debug for Symb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let map = SYMB_MAP.lock().unwrap();
        if let Some(right) = map.get_by_left(&self.0) {
            write!(f,"{}",right)?;
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }
}

#[derive(Debug)]
pub struct DictValue {
    related: Vec<Symb>,
    text: String,
    parsed: TermRef,
    compiled: TermRef,
    linked: Option<TermRef>,
}

impl DictValue {
    pub fn new(input: String) -> Option<DictValue> {
        let text = input;
        let parsed = parser::parse_term(&text[..])?;
        let compiled = compile::compile_ski(parsed);
        let compiled = compile::optimize(compiled);
        let linked = None;
        // TODO related
        let related = Vec::new();
        Some(DictValue { related, text, parsed, compiled, linked })
    }
}

pub fn lookup(symb: Symb) -> Option<TermRef> {
    let map = DICT_MAP.lock().unwrap();
    let value = map.get(&symb)?;
    if let Some(linked) = value.linked {
        Some(linked)
    } else {
        Some(value.compiled)
    }
}
pub fn define(symb: Symb, input: String) -> Option<()> {
    let mut map = DICT_MAP.lock().unwrap();
    if !map.contains_key(&symb) {
        if let Some(new_value) = DictValue::new(input) {
            map.insert(symb,new_value);
            println!("{:?} defined.",symb);
            Some(())
        } else {
            println!("(:define) Can't parse term!");
            None
        }
    } else {
        println!("key {:?} already exist!",symb);
        None
    }
}

pub fn update(symb: Symb, input: String) -> Option<()> {
    let mut map = DICT_MAP.lock().unwrap();
    if map.contains_key(&symb) {
        if let Some(new_value) = DictValue::new(input) {
            map.insert(symb,new_value);
            println!("{:?} updated.",symb);
            Some(())
        } else {
            println!("(:update) Can't parse term!");
            None
        }
    } else {
        println!("key {:?} doen't exist!",symb);
        None
    }
}

pub fn delete(symb: Symb) -> Option<()> {
    let mut map = DICT_MAP.lock().unwrap();
    if map.contains_key(&symb) {
        map.remove(&symb);
        println!("{:?} deleted.", symb);
        Some(())
    } else {
        println!("definition doesn't exist!");
        None
    }
}

pub fn show_dict() {
    let map = DICT_MAP.lock().unwrap();
    for (key, value) in &*map {
        println!("{:?} := {:?}", key, *value);
    }
}

pub fn dict_copy() {
    let mut map = DICT_MAP.lock().unwrap();
    for (_, value) in &mut *map {
        dict_value_copy(value);
        //map.insert(key,new_value);
    }
}

pub fn dict_value_copy(dict: &mut DictValue) {
    dict.parsed = term::term_copy(dict.parsed);
    dict.compiled = term::term_copy(dict.parsed);
    if let Some(linked) = dict.linked {
        dict.linked = Some(term::term_copy(linked));
    }
}