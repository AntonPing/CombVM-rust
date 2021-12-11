#![macro_use]

#[macro_export]
macro_rules! debug {
    ($x:expr) => { 
        if cfg!(debug_assertions) {
            println!($x);
        }
    }
}



/*
struct AssocRef<K,V>(Rc<Assoc<K,V>>);
enum Assoc<K,V> {
    Empty,
    Bind(K,V,AssocRef<K,V>),
}

impl<K: PartialEq,V: Clone> AssocRef<K,V> {
    fn new(list: Assoc<K,V>) -> Self {
        AssocRef(Rc::new(list))
    }
    fn insert(&mut self, key: K, value: V) {
        std::mem::replace(self,
            AssocRef::new(Bind(key,value,*self)));
    }
    fn from_vec(vec: Vec<(K,V)>) -> Self {
        vec.iter().fold(AssocRef::new(Empty),
            |xs, (k,v)| AssocRef::new(Bind(*k,*v,xs)))
    }
    fn update(&self, key: K, value: V) -> Self {
        if let Bind(k,v,xs) = *self.0 {
            if k == key {
                AssocRef::new(Bind(key,value,xs))
            } else {
                AssocRef::new(Bind(k,v,xs.update(key,value)))
            }
        } else { *self }
    }
    fn lookup(&self, key: K) -> Option<V> {
        if let Bind(k,v,xs) = *self.0 {
            if key == k {
                Some(v.clone())
            } else { xs.lookup(key) }
        } else { None }
    }
    fn collapse(&self) -> Self {
        let mut vec = Vec::<K>::new();
        let mut list = AssocRef::new(Empty);
        let mut with = *self;
        while let Bind(k,v,xs) = *with.0 {
            if !vec.contains(&k) {
                list.insert(k, v);
                vec.push(k);
            }
            with = xs;
        }
        list
    }
}

struct Assoc<K,V>(Vec<(K,Option<V>)>);

impl<K: Eq, V> Assoc<K,V> {
    fn new() -> Assoc<K,V> {
        Assoc::<K,V>(Vec::<(K,Option<V>)>::new())
    }
    fn undo(&mut self) {
        self.0.pop();
    }
    fn delete(&mut self, key: K) {
        self.0.push((key,None));
    }
    fn update(&mut self, key: K, value: V) {
        self.0.push((key,Some(value)));
    }
    fn lookup(&self, key: K) -> Option<V> {
        for (k,v) in self.0.iter().rev() {
            if *k == key { return *v }
        }
        return None;
    }
    fn collapse(&self) -> Assoc<K,V> {
        let mut new_vec = Vec::<(K,Option<V>)>::new();
        let mut used_key = Vec::<K>::new();
        for (k,v) in self.0.iter().rev() {
            if !used_key.contains(&k) {
                used_key.push(*k);
                if v.is_some() {
                    new_vec.push((*k,*v));
                }
            }
        }
        Assoc::<K,V>(new_vec)
    }
}
*/