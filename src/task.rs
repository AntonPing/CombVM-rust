use crate::heap;
use crate::eval::Task;

use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

lazy_static::lazy_static! {
    static ref TASK_POOL: Mutex<VecDeque<Task>> =
                            Mutex::new(VecDeque::new());
    static ref HANDLE_POOL: Mutex<Vec<JoinHandle<()>>> =
                            Mutex::new(Vec::new());
}
static THREAD_MAX : usize = 8;
static THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn thread_init() {
    assert_eq!(THREAD_COUNT.load(Ordering::SeqCst), 0);
    heap::set_singal_run();
    let mut handles = HANDLE_POOL.lock().unwrap();
    for i in 0..THREAD_MAX {
        handles.push(thread::spawn(thread_loop));
        THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
        if cfg!(test) { println!("spawn thread {}", i); }
    }
}

pub fn thread_exit() {
    let mut handles = HANDLE_POOL.lock().unwrap();
    while let Some(handle) = handles.pop() {
        handle.join().unwrap();
    }
}

pub fn fetch_task() -> Option<Task> {
    let mut pool = TASK_POOL.lock().unwrap();
    pool.pop_front()
}

pub fn send_task(task: Task) {
    let mut pool = TASK_POOL.lock().unwrap();
    pool.push_back(task);
}

pub fn drain_task() -> Vec<Task> {
    let mut pool = TASK_POOL.lock().unwrap();
    let vec : Vec<Task> = pool.drain(..).collect();
    vec
}

fn thread_loop() {
    while heap::singal_running() {
        if let Some(mut task) = fetch_task() {
            if let Some(ret) = task.eval(1024) {
                println!("task end with: {:?} ", *ret);
            } else {
                send_task(task);
            }
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }
    heap::dump_page();
    let old_count = THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
    //thread_debug_log();
    if old_count == 1 {
        // Oh! you are the chosen one!
        // Do the garbage collection please!
        heap::run_gc();
        thread_init();
    }
    heap::dump_page();
    // Ok, you die now.
}

pub fn thread_debug_log() {
    println!("THREAD_COUNT : {:?}", THREAD_COUNT);
}