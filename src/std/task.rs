extern crate std;

use core::time::Duration;
use std::thread::{self, Thread, ThreadId};

pub type Priority = usize;
pub type TaskId = ThreadId;

pub struct Task {
    thread: Thread,
    priority: Priority,
}

impl Task {
    pub fn priority(&self) -> Priority {
        self.priority
    }
    pub fn id(&self) -> TaskId {
        self.thread.id()
    }
}

pub fn spawn<F: FnOnce() + Send + 'static>(priority: Priority, func: F) -> Task {
    Task {
        thread: thread::spawn(func).thread().clone(),
        priority,
    }
}

pub fn sleep(dur: Duration) {
    thread::sleep(dur);
}
