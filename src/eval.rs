use crate::term;
use crate::term::{ TermRef };
use crate::term::Term::*;
use crate::symbol;
use crate::compile;

use std::fmt;
use std::fmt::Debug;

#[derive(PartialEq,Eq)]
pub struct Task {
    stack: Vec<TermRef>,
    with: TermRef,
    frame: Vec<usize>,
    len: usize,
    ret: Option<TermRef>,
}


impl Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f,"ret: {:?}", self.ret).unwrap();
        writeln!(f,"----------------").unwrap();
        writeln!(f,"{:?}", *self.with).unwrap();
        let mut idx = self.stack.len();
        for _ in 0..self.len {
            idx -= 1;
            writeln!(f,"{:?}", *self.stack[idx]).unwrap();
        }
        writeln!(f,"----------------").unwrap();
        for len in self.frame.iter() {
            for _ in 0..*len {
                idx -= 1;
                writeln!(f,"{:?}", *self.stack[idx]).unwrap();
            }
            writeln!(f,"----------------").unwrap();
        }
        Ok(())
    }
}

pub fn task_copy(task: &mut Task) {
    task.with = term::term_copy(task.with);
    for ptr in &mut task.stack {
        *ptr = term::term_copy(*ptr)
    }
    if let Some(ret) = task.ret {
        task.ret = Some(term::term_copy(ret));
    }
}

impl Task {
    pub fn new(term: TermRef) -> Task {
        Task {
            stack: Vec::new(),
            with: term,
            frame: Vec::new(),
            len: 0,
            ret: None,
        }
    }
    fn push(&mut self, term: TermRef) {
        self.len += 1;
        self.stack.push(term);
    }
    fn pop(&mut self) -> TermRef {
        if self.len == 0 {
            panic!("Can't pop elem from an empty frame!");
        } else {
            self.len -= 1;
            self.stack.pop().unwrap()
        }
    }
    fn call(&mut self, term: TermRef) {
        self.stack.push(self.with);
        self.frame.push(self.len + 1);
        self.with = term;
        self.len = 0;
    }
    fn retn(&mut self) {
        let mut term = self.with;
        for _ in 0..self.len {
            term = app!(term,self.stack.pop().unwrap());
        }
        assert!(self.ret.is_none());
        self.ret = Some(term);
        self.with = self.stack.pop().unwrap();
        self.len = self.frame.pop().unwrap() - 1;
    }
    fn eager(&mut self, index: u8) {
        let n = index as usize;
        if self.len < n + 1 {
            self.retn();
        } else {
            let m = self.stack.len();
            if let Some(res) = self.ret {
                self.stack[m - n - 1] = res;
                self.ret = None;
                if n == 1 {
                    self.with = self.pop();
                } else {
                    self.with = eager!(index-1);
                }
            } else {
                self.call(self.stack[m - n - 1]);
            }
        }
    }
    pub fn eval(&mut self, timeslice: i32) -> Option<TermRef> {
        macro_rules! rewind_check {
            ($n: expr) => {
                if self.len < $n {
                    self.retn();
                    continue;
                }
            }
        }
        macro_rules! reserve {
            ($x: ident) => {
                rewind_check!(1);
                let $x = self.pop();
            };
            ($x: ident,$y:ident) => {
                rewind_check!(2);
                let $x = self.pop();
                let $y = self.pop();
            };
            ($x: ident,$y:ident,$z:ident) => {
                rewind_check!(3);
                let $x = self.pop();
                let $y = self.pop();
                let $z = self.pop();
            };
            ($x: ident,$y:ident,$z:ident,$zz:ident) => {
                rewind_check!(4);
                let $x = self.pop();
                let $y = self.pop();
                let $z = self.pop();
                let $zz = self.pop();
            };
        }
        assert!(timeslice > 0);
        for _ in 0..timeslice {
            //println!("eval: {:?}",self);
            match *self.with {
                Var(x) => {
                    self.with = symbol::lookup(x).expect(
                        &format!("Definition {:?} not found!",x)[..]
                    );
                }
                Lam(_,_) => {
                    self.with = compile::compile_ski(self.with);
                }
                App(t1,t2) => {
                    self.push(t2);
                    self.with = t1;
                }
                I => {
                    reserve!(x);
                    self.with = x;
                }
                K => {
                    reserve!(c,_x);
                    self.with = c;
                }
                S => {
                    reserve!(f,g,x);
                    self.push(app!(g,x));
                    self.push(x);
                    self.with = f;
                }
                B => {
                    reserve!(f,g,x);
                    self.push(app!(g,x));
                    self.with = f;
                }
                C => {
                    reserve!(f,g,x);
                    self.push(g);
                    self.push(x);
                    self.with = f;
                }
                Sp => {
                    reserve!(c,f,g,x);
                    self.push(app!(g,x));
                    self.push(app!(f,x));
                    self.with = c;
                }
                Bs => {
                    reserve!(c,f,g,x);
                    self.push(app!(g,x));
                    self.push(f);
                    self.with = c;
                }
                Cp => {
                    reserve!(c,f,g,x);
                    self.push(g);
                    self.push(app!(f,x));
                    self.with = c;
                }
                E1 => {
                    self.eager(1);
                }
                E2 => {
                    self.eager(2);
                }
                E3 => {
                    self.eager(3);
                }
                E4 => {
                    self.eager(4);
                }
                E(n) => {
                    self.eager(n);
                }
                AddI => {
                    reserve!(x,y);
                    if let (DInt(a),DInt(b)) = (*x,*y) {
                        self.with = i!(a + b);
                    } else {
                        panic!("{:?} takes two interger!",self.with);
                    }
                }
                SubI => {
                    reserve!(x,y);
                    if let (DInt(a),DInt(b)) = (*x,*y) {
                        self.with = i!(a - b);
                    } else {
                        panic!("{:?} takes two interger!",self.with);
                    }
                }
                MulI => {
                    reserve!(x,y);
                    if let (DInt(a),DInt(b)) = (*x,*y) {
                        self.with = i!(a * b);
                    } else {
                        panic!("{:?} takes two interger!",self.with);
                    }
                }
                DivI => {
                    reserve!(x,y);
                    if let (DInt(a),DInt(b)) = (*x,*y) {
                        self.with = i!(a / b);
                    } else {
                        panic!("{:?} takes two interger!",self.with);
                    }
                }
                GrtI => {
                    reserve!(x,y);
                    if let (DInt(a),DInt(b)) = (*x,*y) {
                        self.with = b!(a > b);
                    } else {
                        panic!("{:?} takes two interger!",self.with);
                    }
                }
                LssI => {
                    reserve!(x,y);
                    if let (DInt(a),DInt(b)) = (*x,*y) {
                        self.with = b!(a < b);
                    } else {
                        panic!("{:?} takes two interger!",self.with);
                    }
                }
                EqlI => {
                    reserve!(x,y);
                    if let (DInt(a),DInt(b)) = (*x,*y) {
                        self.with = b!(a == b);
                    } else {
                        panic!("{:?} takes two interger!",self.with);
                    }
                }
                Not => {
                    reserve!(x,y);
                    if let (DInt(a),DInt(b)) = (*x,*y) {
                        self.with = b!(a == b);
                    } else {
                        panic!("{:?} takes two interger!",self.with);
                    }
                }
                Ifte => {
                    reserve!(x,y,z);
                    if let DBool(p) = *x {
                        if p {
                            self.with = y;
                        } else {
                            self.with = z;
                        }
                    } else {
                        panic!("{:?} takes a boolean and two terms!",self.with);
                    }
                }
                
                DInt(_) | DBool(_) => {
                    assert_eq!(self.len, 0);
                    if self.frame.is_empty() {
                        // task finished
                        return Some(self.with);
                    } else {
                        self.retn();
                    }
                }
                _ => {
                    panic!("{:?}, unknown term for eval",*self.with);
                }
            }
        }
        // not finished
        return None;

    }
}


