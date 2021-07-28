use crate::eval::Task;

use std::fmt;
use std::fmt::Debug;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::collections::VecDeque;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref TASK_POOL: Mutex<VecDeque<Task>> =
                            Mutex::new(VecDeque::new());
}
static THREAD_NUM : usize = 8;

pub fn thread_init() -> Vec<JoinHandle<()>> {
    let mut vec = Vec::new();
    for i in 0..THREAD_NUM {
        vec.push(thread::spawn(thread_loop));
        if cfg!(test) { println!("spawn thread {}", i); }
    }
    vec    
}

pub fn thread_exit(vec: Vec<JoinHandle<()>>) {
    for handle in vec {
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

fn thread_loop() {
    loop {
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
}