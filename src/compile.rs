use crate::term::*;
use crate::term::Term::*;
use crate::symbol::*;

pub fn is_free_in(symb: Symb, term: TermRef) -> bool {
    match *term {
        Var(x) => { x == symb }
        Lam(x,t) => {
            if x == symb { false }
            else { is_free_in(symb,t) }
        }
        App(t1,t2) => {
            is_free_in(symb,t1) || is_free_in(symb,t2)
        }
        _ => { false }
    }
}

pub fn compile_ski(term: TermRef) -> TermRef {
    match *term {
        Var(_) => { term }
        App(t1,t2) => {
            app!(compile_ski(t1),compile_ski(t2))
        }
        Lam(x,t) => {
            if !is_free_in(x,t) {
                // T[\x.E] => (K T[E]), if x is not free in E
                app!(C_K,compile_ski(t))
            } else {
                match *t {
                    // if x is free in t, and t is a Var
                    // It could only be \x.x
                    // T[\x.x] => I
                    Var(_) => { C_I }
                    // T[\x.\y.E] => T[\x.T[\y.E]]
                    Lam(x2,t2) => {
                        compile_ski(lam!(x,
                            compile_ski(lam!(x2,t2))))
                    }
                    // T[\x.(E1 E2)] => (S T[\x.E1] T[\x.E2])
                    App(t1,t2) => {
                        app!(C_S,
                            compile_ski(lam!(x,t1)),
                            compile_ski(lam!(x,t2)))
                    }
                    // x can't be free in constant!
                    _ => { panic!("impossible!"); }
                }
            }
        }
        // constants
        _ => { term }
    }
}

// S (K p) I = p
// S (K p) (K q) = K (p q)
// S (K p) (B q r) = B* p q r
// S (K p) q = B p q
// S (B p q) (K r) = C' p q r
// S (B p q) r = S' p q r
// S p (K q) = C p q
pub fn optimize(term: TermRef) -> TermRef {
    if let App(t1,t2) = *term {
        if let App(t11,t12) = *t1 {
            if let S = *t11 {
                let arg1 = t12;
                let arg2 = t2;
                // term in form of (S arg1 arg2)
                if let App(t1,t2) = *arg1 {
                    if let K = *t1 {
                        let p = t2;
                        // term in form of (S (K p) arg2)
                        if let I = *arg2 {
                             // S (K p) I = p
                            return p;
                        }
                        if let App(t1,t2) = *arg2 {
                            if let K = *t1 {
                                let q = t2;
                                // S (K p) (K q) = K (p q)
                                return app!(C_K,app!(p,q));
                            }
                            if let App(t1,t2) = *t1 {
                                if let B = *t1 {
                                    let q = t2;
                                    // S (K p) (B q r) = B* p q r
                                    return app!(C_BS,app!(p,q));
                                }
                            }
                        }
                        let q = arg2;
                        // S (K p) q = B p q
                        return app!(C_B,p,q);
                    }
                    if let App(t11,t12) = *t1 {
                        if let B = *t11 {
                            let p = t12;
                            let q = t2;
                            // term in form of (S (B p q) arg2)
                            if let App(t1,t2) = *arg2 {
                                if let K = *t1 {
                                    let r = t2;
                                    // S (B p q) (K r) = C' p q r
                                    return app!(C_CP,p,q,r);
                                }
                                let r = arg2;
                                // S (B p q) r = S' p q r
                                return app!(C_SP,p,q,r);
                            }
                        }
                    }
                }
                let p = arg1;
                if let App(t1,t2) = *arg2 {
                    if let K = *t1 {
                        let q = t2;
                        // S p (K q) = C p q
                        return app!(C_C,p,q);
                    }
                }
                let q = arg2;
                // S p q = S T[p] T[q]
                return app!(C_S,optimize(p),optimize(q));

            } else {
                return app!(optimize(t1),optimize(t2));
            }
        } else {
            return app!(t1,optimize(t2));
        }
    } else {
        return term;
    }
}