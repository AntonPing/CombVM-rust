use std::ptr;
use std::mem;
use std::sync::Mutex;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

//use crate::term;
use crate::symbol;
use crate::term::{Term, TermRef};
use crate::eval;
use crate::task;

pub unsafe fn malloc<T>(size: usize) -> *mut T {
    if size == 0 { return ptr::null_mut(); }
    debug_assert!(mem::size_of::<T>() > 0,
        "manually allocating a buffer of ZST is a very dangerous idea"
    );
    let mut vec = Vec::<T>::with_capacity(size);
    let ret = vec.as_mut_ptr();
    mem::forget(vec); // avoid dropping the memory
    ret
}

pub unsafe fn free<T> (array: *mut T, size: usize) {
    if size == 0 { return; }
    let _ : Vec<T> = Vec::from_raw_parts(array, 0, size);
}


#[test]
pub fn malloc_and_free_test() {
    unsafe {
        let ptr: *mut Term = malloc(100);
        free(ptr,100);
    }
}


lazy_static::lazy_static! {
    static ref DUMP_POOL: Mutex<Vec<Page>> = 
                            Mutex::new(Vec::new());
}

static WATERMARK: usize = 32;
static PAGE_SIZE: usize = 65536;

static STW_SINGAL: AtomicBool = AtomicBool::new(true);
pub fn singal_running() -> bool {
    let singal = STW_SINGAL.load(Ordering::Relaxed);
    singal
}
pub fn set_singal_run() {
    STW_SINGAL.store(true,Ordering::Relaxed);
}
pub fn set_singal_stop() {
    STW_SINGAL.store(false,Ordering::Relaxed);
}

thread_local! {
    pub static PAGE : RefCell<Page> =
                RefCell::new(Page::new(PAGE_SIZE));
}

#[derive(Debug)]
pub struct Page {
    array: *mut Term,
    size: usize,
    index: usize,
}

unsafe impl Send for Page {}
unsafe impl Sync for Page {}

impl Drop for Page {
    fn drop(&mut self) {
        unsafe { free(self.array,self.size) };
        //println!("free {:?}", self.array);
    }
}

impl Page {
    pub fn new(size: usize) -> Page {
        let array: *mut Term = unsafe { malloc(size) };
        //assert!(!array.is_null());
        Page { array: array, size: size, index: 0 }
    }
}

/*
pub trait GCable {
    fn gc(&self) -> Self;
}
*/

pub fn drain_dump() -> Vec<Page> {
    let mut dump = DUMP_POOL.lock().unwrap();
    dump.drain(..).collect()
}

pub fn run_gc() {
    //println!("gc_start");
    let _dump = drain_dump();
    let mut vec = task::drain_task();
    while let Some(mut task) = vec.pop() {
        eval::task_copy(&mut task);
        task::send_task(task);
    }
    symbol::dict_copy();
    //println!("gc_end");
}

pub fn term_alloc(term: Term) -> TermRef {
    let mut result : Option<TermRef> = None;
    loop {
        PAGE.with(|page| {
            let mut p = page.borrow_mut();
            if p.index < p.size {
                unsafe {
                    let ptr = p.array.offset(p.index as isize);
                    *ptr = term;
                    p.index += 1;
                    result = Some(TermRef::new(ptr));
                }
            }
        });
        if let Some(term) = result {
            return term;       
        } else {
            next_page();
        }
    }
}

pub fn next_page() {
    //println!("refresh");
    PAGE.with(|page| {
        let page2 = RefCell::new(Page::new(PAGE_SIZE));
        page.swap(&page2);
        let mut dump = DUMP_POOL.lock().unwrap();
        dump.push(page2.into_inner());
        if dump.len() >= WATERMARK {
            set_singal_stop();
        }
    })
}

pub fn dump_page() {
    let mut dump = DUMP_POOL.lock().unwrap();
    PAGE.with(|page| {
        let page2 = RefCell::new(Page::new(0));
        page.swap(&page2);
        dump.push(page2.into_inner());
    })
}