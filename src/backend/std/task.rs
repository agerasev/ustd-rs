extern crate std;

use crate::Error;
use core::time::Duration;
use std::{
    cell::Cell,
    ops::{Add, AddAssign},
    thread::{self, Thread, ThreadId},
    thread_local,
};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct TaskId(ThreadId);

#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Priority(usize);

#[derive(Clone, Copy, Debug)]
struct TaskInfo {
    priority: Priority,
}

#[derive(Clone, Debug)]
pub struct Task {
    thread: Thread,
    info: TaskInfo,
}

thread_local! {
    static TASK_INFO: Cell<Option<TaskInfo>> = Cell::new(None);
}

impl Task {
    pub fn id(&self) -> TaskId {
        TaskId(self.thread.id())
    }
    pub fn priority(&self) -> Priority {
        self.info.priority
    }
    pub(crate) fn thread(&self) -> Thread {
        self.thread.clone()
    }
}

pub fn spawn<F: FnOnce() + Send + 'static>(priority: Priority, func: F) -> Result<Task, Error> {
    let info = TaskInfo { priority };
    Ok(Task {
        thread: thread::spawn(move || {
            TASK_INFO.with(|this| this.set(Some(info)));
            func();
        })
        .thread()
        .clone(),
        info,
    })
}

pub fn current() -> Result<Task, Error> {
    Ok(Task {
        thread: thread::current(),
        info: TASK_INFO
            .with(|this| this.get())
            .unwrap_or_else(|| TaskInfo {
                priority: Priority::default(),
            }),
    })
}

pub fn sleep(dur: Duration) {
    thread::sleep(dur);
}

impl Add<usize> for Priority {
    type Output = Self;
    fn add(mut self, rhs: usize) -> Self::Output {
        self.0 += rhs;
        self
    }
}

impl AddAssign<usize> for Priority {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}
