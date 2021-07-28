use std::mem;
use std::ptr;
use std::sync::Mutex;
use std::cell::RefCell;

use crate::term::{ Term, TermRef };
use crate::term::Term::*;
use crate::symbol::Symb;

pub unsafe fn malloc<T>(size: usize) -> *mut T {
    debug_assert!(mem::size_of::<T>() > 0,
        "manually allocating a buffer of ZST is a very dangerous idea"
    );
    let mut vec = Vec::<T>::with_capacity(size);
    let ret = vec.as_mut_ptr();
    mem::forget(vec); // avoid dropping the memory
    ret
}

pub unsafe fn free<T> (array: *mut T, size: usize) {
    let _ : Vec<T> = Vec::from_raw_parts(array, 0, size);
}

lazy_static::lazy_static! {
    static ref DUMP_POOL: Mutex<Vec<Page>> = 
                            Mutex::new(Vec::new());
}

static PAGE_SIZE: usize = 65536;

thread_local! {
    pub static PAGE : RefCell<Page> =
                RefCell::new(Page::new(PAGE_SIZE));
}

#[derive(Copy,Clone,Debug)]
pub struct Page {
    array: *mut Term,
    size: usize,
    index: usize,
}

unsafe impl Send for Page {}
unsafe impl Sync for Page {}

impl Page {
    pub fn new(size: usize) -> Page {
        let array: *mut Term = unsafe { 
            malloc(size)
        };
        if array.is_null() {
            panic!("failed to malloc page");
        }
        Page { array: array, size: size, index: 0 }
    }
    pub fn alloc(&mut self, term: Term) -> TermRef {
        //println!("alloc on {:?}", self);
        unsafe {
            if self.index < self.size {
                let ptr = self.array.offset(self.index as isize);
                *ptr = term;
                self.index += 1;
                //println!("ok {:?} ", *ptr);
                TermRef::new(ptr)
            } else {
                println!("refresh");
                self.refresh();
                self.alloc(term)
            }
        }
    }
    pub fn refresh(&mut self) {
        let mut dump = DUMP_POOL.lock().unwrap();
        dump.push(self.clone());
        *self = Page::new(self.size);
    }
}

pub fn term_alloc(term: Term) -> TermRef {
    PAGE.with(|page| {
        let mut page2 = *page.borrow_mut();
        let result = page2.alloc(term);
        *page.borrow_mut() = page2;
        result
    })
}