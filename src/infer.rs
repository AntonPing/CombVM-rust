#![macro_use]
use std::collections::HashMap;
use std::rc::Rc;
use std::ops::Deref;
use std::fmt::Debug;
use std::fmt;
use hashbag::HashBag;

use crate::infer::Type::*;
use crate::infer::Expr::*;
use crate::symbol::Symb;
use crate::util;

lazy_static::lazy_static! {
    static ref NAME_LIST: Vec<&'static str> = vec![
        "a","b","c","d","e","f","g",
        "h","i","j","k","l","m","n",
        "o","p","q","r","s","t",
        "u","v","w","x","y","z",
    ];
}

#[derive(Clone)]
struct ExprRef(Rc<Expr>);
pub enum Expr {
    LitInt(i64),
    Var(Symb),
    Lam(Symb,ExprRef),
    App(ExprRef,ExprRef),
    LetIn(Symb,ExprRef,ExprRef),
}

impl Deref for ExprRef {
    type Target = Expr;
    fn deref(&self) -> &Expr {
        &*self.0
    }
}

impl ExprRef {
    fn new(exp: Expr) -> ExprRef {
        ExprRef(Rc::new(exp))
    }
}

#[derive(Clone)]
struct TypeRef(Rc<Type>);
pub enum Type {
    Const(Symb),
    TVar(Symb),
    Arrow(TypeRef,TypeRef),
}

impl Deref for TypeRef {
    type Target = Type;
    fn deref(&self) -> &Type {
        &*self.0
    }
}

impl Debug for TypeRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self.0).fmt(f)
    }
}

impl Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Const(a) => {
                write!(f,"{:?}",a)?;
            }
            TVar(x) => {
                write!(f,"{:?}",x)?;
            }
            Arrow(t1,t2) => {
                write!(f,"(")?;
                t1.fmt(f)?;
                let mut with = t2;
                while let Arrow(t21,t22) = with.deref() {
                    write!(f," -> ")?;
                    t21.fmt(f)?;
                    with = t22;
                }
                write!(f," -> ")?;
                with.fmt(f)?;
                write!(f,")")?;
            }
        }
        Ok(())
    }
}

impl TypeRef {
    fn new(ty: Type) -> TypeRef {
        TypeRef(Rc::new(ty))
    }
    fn constant(str: &str) -> TypeRef {
        TypeRef::new(Const(Symb::new(str)))
    }
    fn ftv(&self) -> VarSet {
        let mut result = HashBag::new();
        let mut stack = Vec::<TypeRef>::new();
        stack.push(self.clone());
        while let Some(elem) = stack.pop() {
            match elem.deref() {
                Const(_) => {},
                TVar(x) => {
                    result.insert(*x);
                }
                Arrow(t1,t2) => {
                    stack.push(t2.clone());
                    stack.push(t1.clone());
                }
            }
        }
        result
    }
    fn subst(&self, sub: &Subst) -> TypeRef {
        match &*self.0 {
            Const(_) => self.clone(),
            TVar(x) => 
                if let Some(t) = sub.get(&x)
                { t.clone().subst(sub) } else { self.clone() }
            Arrow(t1,t2) =>
                TypeRef(Rc::new(Arrow(t1.subst(sub),t2.subst(sub))))
        }
    }
    fn occur_check(&self, x: Symb) -> bool {
        self.ftv().contains(&x) > 0
    }
}

type Subst = HashMap<Symb,TypeRef>;
type VarSet = HashBag<Symb>;

#[derive(Clone)]
struct Scheme(Vec<Symb>,TypeRef);

impl Debug for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.0.is_empty() {
            write!(f,"∀")?;
            for x in self.0.iter() {
                write!(f," ")?;
                x.fmt(f)?;
            }
            write!(f,".")?;
        } else {
            write!(f,":")?;
        }
        self.1.fmt(f)?;
        Ok(())
    }
}

impl Scheme {
    fn new(ty: &TypeRef) -> Scheme {
        Scheme(Vec::new(),ty.clone())
    }
    fn ftv(&self) -> VarSet {
        let mut set = self.1.ftv();
        for x in self.0.iter() {
            set.take_all(x);
        }
        set
    }

    fn rename(&self, name: &Vec<&str>) -> Scheme {
        let mut sub = HashMap::new();
        let mut stack = Vec::new();
        let mut i = 0;
        stack.push(self.1.clone());
        while let Some(elem) = stack.pop() {
            match elem.deref() {
                Const(_) => {},
                TVar(x) => {
                    if self.0.contains(x) && !sub.contains_key(x) {
                        sub.insert(*x,TypeRef::new(
                            Type::TVar(Symb::new(name[i]))));
                        i += 1;
                    }
                }
                Arrow(t1,t2) => {
                    stack.push(t2.clone());
                    stack.push(t1.clone());
                }
            }
        }
        let newvars = name.iter()
            .take(self.0.len())
            .map(|x| Symb::new(x))
            .collect();
        Scheme(newvars,self.1.subst(&sub))
    }
}

#[derive(Debug)]
enum EnvHistory {
    Delete(Symb,Scheme),
    Insert(Symb,Scheme),
    Update(Symb,Scheme,Scheme),
    Nothing,
}

#[derive(Debug)]
struct Environment {
    current: HashMap<Symb,Scheme>,
    freevars: HashBag<Symb>,
    history: Vec<EnvHistory>,
}

impl Environment {
    fn new() -> Environment {
        Environment {
            current: HashMap::new(),
            freevars: HashBag::new(),
            history: Vec::new(),
        }
    }

    fn lookup(&self, x: Symb) -> Option<&Scheme> {
        self.current.get(&x)
    }

    fn contains(&self, x: Symb) -> bool {
        self.freevars.contains(&x) > 0
    }

    fn add_scheme(&mut self, sc: &Scheme) {
        for (x,n) in sc.ftv() {
            self.freevars.insert_many(x, n);
        }
    }

    fn remove_scheme(&mut self, sc: &Scheme) {
        for (x,n) in sc.ftv() {
            if let Some((_,m)) = self.freevars.get(&x) {
                self.freevars.take_all(&x);
                self.freevars.insert_many(x, m - n);
            } else {
                self.freevars.insert_many(x, n);
            }
        }
    }

    fn update(&mut self, k: Symb, v: &Scheme) -> usize {
        if let Some(old) = self.current.insert(k,v.clone()) {
            self.add_scheme(v);
            self.remove_scheme(&old);
            self.history.push(EnvHistory::Update(k,old,v.clone()));
        } else {
            self.add_scheme(v);
            self.history.push(EnvHistory::Insert(k,v.clone()));
        }
        self.history.len()
    }

    fn delete(&mut self, k: Symb) -> usize {
        if let Some(old) = self.current.remove(&k) {
            self.remove_scheme(&old);
            self.history.push(EnvHistory::Delete(k,old));
        } else {
            self.history.push(EnvHistory::Nothing);
        }
        self.history.len()

    }
    fn backup(&self) -> usize {
        self.history.len()
    }

    fn recover(&mut self, mark: usize) {
        for _ in mark..self.history.len() {
            if let Some(row) = self.history.pop() {
                match row {
                    EnvHistory::Delete(x,sc) => {
                        let r = self.current.insert(x, sc.clone());
                        self.add_scheme(&sc);
                        assert!(r.is_none());
                    }
                    EnvHistory::Insert(x,sc) => {
                        let r = self.current.remove(&x);
                        self.remove_scheme(&sc);
                        assert!(r.is_some());
                    }
                    EnvHistory::Update(x,t0,t1) => {
                        let r = self.current.insert(x,t0.clone());
                        self.add_scheme(&t0);
                        self.remove_scheme(&t1);
                        assert!(r.is_some());
                    }
                    EnvHistory::Nothing => {
                        // Well, Nothing...
                    }
                }
            } else {
                panic!("Can't Be!")
            }
        }
    }
}

struct Constraints {
    cons: Vec<(TypeRef,TypeRef)>,
}

impl Constraints {
    fn new() -> Constraints {
        Constraints { cons: Vec::new() }
    }
    fn unify(&mut self, t1: &TypeRef, t2: &TypeRef) {
        self.cons.push((t1.clone(),t2.clone()));
    }
    fn solve(&mut self) -> Result<Subst,String> {
        let mut map = HashMap::new();
        while let Some((t1,t2)) = self.cons.pop() {
            let ref ty1 = t1.subst(&map);
            let ref ty2 = t2.subst(&map);
            match (ty1.deref(),ty2.deref()) {
                (TVar(x),_) => {
                    if ty2.occur_check(*x) {
                        return Err("Occur check failed!".to_string());
                    } else {
                        map.insert(*x, ty2.clone());
                    }
                }
                (_,TVar(x)) => {
                    if ty1.occur_check(*x) {
                        return Err("Occur check failed!".to_string());
                    } else {
                        map.insert(*x, ty1.clone());
                    }
                }
                (Const (a),Const(b)) => {
                    if a == b {
                        continue;
                    } else {
                        return Err(format!("Can't unify {:?} and {:?}!",a,b));
                    }
                }
                (Arrow(a1,a2),Arrow(b1,b2)) => {
                    self.cons.push((a1.clone(),b1.clone()));
                    self.cons.push((a2.clone(),b2.clone()));
                }
                (a,b) => {
                    return Err(format!("Can't unify {:?} and {:?}!",a,b))
                }
            }
        }
        Ok(map)
    }
    
}


struct Infer {
    env: Environment,
    cons: Constraints,
    fresh_idx: usize,
    err_msg: Vec<String>,
}

impl Infer {
    fn new() -> Infer {
        Infer {
            env: Environment::new(),
            cons: Constraints::new(),
            fresh_idx: 0,
            err_msg: Vec::new()
        }
    }
    fn newvar(&mut self) -> TypeRef {
        let mut var_name = "#".to_string();
        let suffix = self.fresh_idx.to_string();
        var_name.push_str(&suffix);
        let ty = TypeRef::new(TVar(Symb::from_string(var_name)));
        self.fresh_idx += 1;
        ty
    }
    fn generalize(&mut self, ty: &TypeRef) -> Scheme {
        let mut vec = Vec::new();
        for (x,_) in ty.ftv() {
            if !self.env.contains(x) {
                vec.push(x);
            } else {
                dbg!(&self.env);
            }
        }
        Scheme(vec,ty.clone()).rename(&NAME_LIST)
    }
    fn instantiate(&mut self, sc: &Scheme) -> TypeRef {
        let mut sub = HashMap::new();
        let len = sc.0.len();
        for i in 1..len {
            sub.insert(sc.0[i], self.newvar());
        }
        sc.1.subst(&sub)
    }
    fn infer(&mut self, exp: &ExprRef) -> Result<TypeRef,String> {
        match exp.deref() {
            LitInt(_) => {
                Ok(TypeRef::constant("Int"))
            }
            Var(x) => {
                if let Some(sc) = self.env.lookup(*x).cloned() {
                    Ok(self.instantiate(&sc))
                } else {
                    Err("Variable not in the environment!".to_string())
                }
                
            }
            Lam(x,t) => {
                let x2 = self.newvar();
                let mark = self.env.update(*x, &Scheme::new(&x2));
                let t2 = self.infer(t)?;
                self.env.recover(mark);
                Ok(TypeRef::new(Arrow(x2,t2)))
            }
            App(ea,eb) => {
                let ta = self.infer(ea)?;
                let tb = self.infer(eb)?;
                let tc = self.newvar();
                self.cons.unify(&ta, &TypeRef::new(Arrow(tb,tc.clone())));
                Ok(tc)
            },
            LetIn(x,ea,eb) => {
                let ta = self.infer(ea)?;
                let sc = self.generalize(&ta);
                let mark = self.env.update(*x, &sc);
                let tb = self.infer(eb)?;
                self.env.recover(mark);
                Ok(tb)
            }
        }
    }
    fn infer_top(&mut self, exp: &ExprRef) -> Result<Scheme,String> {
        let mark = self.env.backup();
        let ty = self.infer(&exp)?;
        let sub = self.cons.solve()?;
        self.env.recover(mark);


        let sc = self.generalize(&ty.subst(&sub));
        Ok(sc)
    }
}

macro_rules! letin {
    ($x:expr, $e1:expr, $e2:expr) => {
        ExprRef::new(Expr::LetIn(Symb::new($x),$e1,$e2))
    };
}

macro_rules! var {
    ($x:expr) => {
        ExprRef::new(Expr::Var(Symb::new($x)))
    };
}

macro_rules! lam {
    ($x:expr, $e:expr) => {
        ExprRef::new(Expr::Lam(Symb::new($x),$e))
    };
    ($x:expr,$y:expr,$($rest:tt)*) => {
        lam!($x,lam!($y,$($rest)*))
    };
}

macro_rules! app {
    ($e1:expr,$e2:expr) => {
        ExprRef::new(Expr::App($e1,$e2))
    };
    ($e1:expr,$e2:expr,$($rest:tt)*) => {
        app!(app!($e1,$e2),$($rest)*)
    };
}


#[test]
pub fn infer_test() -> Result<(),String> {
    let mut inf = Infer::new();

    let e1 = lam!("f","x",app!(var!("f"),var!("x")));
    let e2 = lam!("f","g","x",
        app!(var!("f"),app!(var!("g"),var!("x"))));

    
    let sc1 = inf.infer_top(&e1)?;
    println!("type: {:?}",sc1);
    let sc2 = inf.infer_top(&e2)?;
    println!("type: {:?}",sc2);
    Ok(())
}