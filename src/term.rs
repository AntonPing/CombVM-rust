#![macro_use]
use std::fmt;
use std::ops::Deref;
use std::fmt::Debug;

use crate::term::Term::*;
use crate::symbol::Symb;

#[derive(Clone,Copy,PartialEq)]
pub enum Term {
    App(TermRef,TermRef),
    Lam(Symb,TermRef),
    Var(Symb),
    DBool(bool),
    DChar(char),
    DInt(i64),
    DReal(f64),
    E1,E2,E3,E4,E(u8),
    I,K,S,B,C,Sp,Bs,Cp,
    AddI,SubI,MulI,DivI,
    GrtI,LssI,EqlI,
    Not,And,Or,Ifte,
    //List(TermRef,TermRef),
    //EndOfList,
    Array(usize,*mut TermRef),
    Alloc,Free,Load,Save,
}

#[derive(Eq)]
pub struct TermRef(*mut Term);

unsafe impl Send for TermRef {}
unsafe impl Sync for TermRef {}
impl TermRef {
    pub fn new(ptr: *mut Term) -> TermRef {
        TermRef(ptr)
    } 
    unsafe fn set(&self, x: Term) {
        *self.0 = x;
    }
}
impl Deref for TermRef {
    type Target = Term;
    fn deref(&self) -> &Term {
        unsafe { &*self.0 }
    }
}
impl Copy for TermRef {}
impl Clone for TermRef {
    fn clone(&self) -> TermRef {
        TermRef(self.0)
    }
}
impl Debug for TermRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"<{:?}>",self.deref())?;
        Ok(())
    }
}
impl PartialEq for TermRef {
    fn eq(&self, other: &Self) -> bool {
           self.0 == other.0
        || self.deref() == other.deref()
    }
}

macro_rules! const_term {
    ($var:ident, $term:expr) => {
        pub static $var : TermRef = TermRef(&$term as *const Term as *mut Term);
    };
}
const_term!(C_I,I);
const_term!(C_K,K);
const_term!(C_S,S);
const_term!(C_B,B);
const_term!(C_C,C);
const_term!(C_SP,Sp);
const_term!(C_BS,Bs);
const_term!(C_CP,Cp);
const_term!(C_E1,E1);
const_term!(C_E2,E2);
const_term!(C_E3,E3);
const_term!(C_E4,E4);
const_term!(C_ADDI,AddI);
const_term!(C_SUBI,SubI);
const_term!(C_MULI,MulI);
const_term!(C_DIVI,DivI);
const_term!(C_GRTI,GrtI);
const_term!(C_LSSI,LssI);
const_term!(C_EQLI,EqlI);
const_term!(C_NOT,Not);
const_term!(C_AND,And);
const_term!(C_OR,Or);
const_term!(C_IFTE,Ifte);

#[macro_export]
macro_rules! alloc {
    ($v:expr) => {
        crate::heap::term_alloc($v)
    };
}

#[macro_export]
macro_rules! eager {
    ($n:expr) => {
        match $n {
            1 => { crate::term::C_E1 }
            2 => { crate::term::C_E2 }
            3 => { crate::term::C_E3 }
            4 => { crate::term::C_E4 }
            _ => alloc!(E($n))
        }
    };
}
#[macro_export]
macro_rules! b {
    ($v:expr) => {
        alloc!(DBool($v))
    };
}
#[macro_export]
macro_rules! c {
    ($v:expr) => {
        alloc!(DChar($v))
    };
}
#[macro_export]
macro_rules! i {
    ($v:expr) => {
        alloc!(DInt($v))
    };
}
macro_rules! r {
    ($v:expr) => {
        alloc!(DReal($v))
    };
}

#[macro_export]
macro_rules! app {
    ($t1:expr,$t2:expr) => {
        alloc!(App($t1,$t2))
    };
    ($t1:expr,$t2:expr,$($rest:tt)*) => {
        app!(alloc!(App($t1,$t2)),$($rest)*)
    };
}

#[macro_export]
macro_rules! lam {
    ($x:expr,$t:expr) => {
        alloc!(Lam($x,$t))
    };
    ($x:expr,$($rest:tt)*) => {
        alloc!(Lam($x,lam!($($rest)*)))
    };
}

#[macro_export]
macro_rules! var {
    ($x:expr) => {
        alloc!(Var($x))
    };
}



impl Term {
    fn app_list_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let App(t1,t2) = self {
            t1.deref().app_list_fmt(f)?;
            write!(f," ")?;
            t2.deref().fmt(f)?;
        } else {
            self.fmt(f)?;
        }
        Ok(())
    }
}

impl Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            App(_,_) => {
                write!(f,"(")?;
                self.app_list_fmt(f)?;
                write!(f,")")?;
            }
            Lam(x,t) => {
                write!(f,"λ {:?}",x)?;
                let mut with = t;
                while let Lam(wx,wt) = with.deref() {
                    write!(f," {:?}", wx)?;
                    with = wt;
                }
                write!(f,". ")?;
                with.app_list_fmt(f)?;
            }
            Var(x) => { write!(f,"{:?}",x)?; }
            DBool(x) => { write!(f,"{}",x)?; }
            DChar(x) => { write!(f,"{}",x)?; }
            DInt(x) => { write!(f,"{}",x)?; }
            DReal(x) => { write!(f,"{}",x)?; }
            E1 => { write!(f,"E1")?; }
            E2 => { write!(f,"E2")?; }
            E3 => { write!(f,"E3")?; }
            E4 => { write!(f,"E4")?; }
            E(n) => { write!(f,"E{}",n)?; }
            I => { write!(f,"I")?; }
            K => { write!(f,"K")?; }
            S => { write!(f,"S")?; }
            B => { write!(f,"B")?; }
            C => { write!(f,"C")?; }
            Sp => { write!(f,"S'")?; }
            Bs => { write!(f,"B*")?; }
            Cp => { write!(f,"C'")?; }
            AddI => { write!(f,"AddI")?; }
            SubI => { write!(f,"SubI")?; }
            MulI => { write!(f,"MulI")?; }
            DivI => { write!(f,"DivI")?; }
            GrtI => { write!(f,"GrtI")?; }
            LssI => { write!(f,"LssI")?; }
            EqlI => { write!(f,"EqlI")?; }
            Not => { write!(f,"Not")?; }
            And => { write!(f,"And")?; }
            Or => { write!(f,"Or")?; }
            Ifte => { write!(f,"Ifte")?; }
            Array(n,ptr) => { write!(f,"Array{}:{:p}",n,ptr)?; }
            Alloc => { write!(f,"Alloc")?; }
            Free => { write!(f,"Free")?; }
            Load => { write!(f,"Load")?; }
            Save => { write!(f,"Save")?; }
        }
        Ok(())
    }
}

pub fn term_copy(term: TermRef) -> TermRef {
    //println!("term:{:?}",term);
    match *term {
        App(t1,t2) => {
            let new_t1 = term_copy(t1);
            let new_t2 = term_copy(t2);
            app!(new_t1,new_t2)
        }
        Lam(x,t) => {
            let new_t = term_copy(t);
            lam!(x,new_t)
        }
        Var(x) => {
            var!(x)
        }
        DChar(x) => { c!(x) }
        DBool(x) => { b!(x) }
        DInt(x) => { i!(x) }
        DReal(x) => { r!(x) }
        E(n) => {
            eager!(n)
        }
        _ => {
            term
        }
    }
}