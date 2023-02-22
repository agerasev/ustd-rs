extern crate std;

use crate::{
    interrupt::InterruptContext,
    task::{self, Task, TaskId},
    Error,
};
use core::time::Duration;
use fnv::FnvBuildHasher;
use std::{
    collections::HashMap,
    sync::Mutex,
    thread::{self, ThreadId},
    time::Instant,
};

#[derive(Clone, Copy, Debug, Default)]
enum Value {
    #[default]
    Down,
    Up,
    Unpark(ThreadId),
}

#[derive(Default)]
struct SemaphoreUnprotected {
    value: Value,
    queue: HashMap<TaskId, Task, FnvBuildHasher>,
}

/// MPMC binary semaphore.
pub struct Semaphore {
    shared: Mutex<SemaphoreUnprotected>,
}

impl Semaphore {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            shared: Mutex::new(SemaphoreUnprotected {
                value: Value::Down,
                queue: HashMap::default(),
            }),
        })
    }

    /// Returns `true` on success.
    pub fn try_give(&self) -> bool {
        let mut guard = self.shared.lock().unwrap();
        match guard.value {
            Value::Down => (),
            _ => return false,
        }
        match guard
            .queue
            .iter()
            .max_by_key(|(_, v)| v.priority())
            .map(|(k, _)| *k)
            .map(|k| guard.queue.remove(&k).unwrap())
        {
            Some(task) => {
                let thread = task.thread();
                guard.value = Value::Unpark(thread.id());
                drop(guard);
                thread.unpark();
            }
            None => guard.value = Value::Up,
        }
        true
    }

    pub fn try_give_from_intr(&self, _intr_ctx: &mut InterruptContext) -> bool {
        self.try_give()
    }

    /// Returns `true` on success.
    pub fn try_take(&self) -> bool {
        let mut guard = self.shared.lock().unwrap();
        if let Value::Up = guard.value {
            guard.value = Value::Down;
            true
        } else {
            false
        }
    }

    pub fn try_take_from_intr(&self, _intr_ctx: &mut InterruptContext) -> bool {
        self.try_take()
    }

    pub fn take(&self) {
        let task = task::current().unwrap();

        let mut guard = self.shared.lock().unwrap();
        if let Value::Up = guard.value {
            guard.value = Value::Down;
            return;
        } else {
            assert!(guard.queue.insert(task.id(), task.clone()).is_none());
        }
        drop(guard);

        loop {
            thread::park();

            let mut guard = self.shared.lock().unwrap();
            match guard.value {
                Value::Up => unreachable!(),
                Value::Unpark(id) => {
                    if id == task.thread().id() {
                        assert!(!guard.queue.contains_key(&task.id()));
                        guard.value = Value::Down;
                        break;
                    }
                }
                Value::Down => (),
            }
        }
    }

    /// Returns `true` on success, `false` when timed out.
    pub fn take_timeout(&self, timeout: Duration) -> bool {
        let task = task::current().unwrap();

        let mut guard = self.shared.lock().unwrap();
        if let Value::Up = guard.value {
            guard.value = Value::Down;
            return true;
        } else {
            assert!(guard.queue.insert(task.id(), task.clone()).is_none());
        }
        drop(guard);

        let mut remaining = timeout;
        loop {
            let start = Instant::now();
            thread::park_timeout(remaining);

            let stop = Instant::now();
            let mut guard = self.shared.lock().unwrap();
            match guard.value {
                Value::Up => unreachable!(),
                Value::Unpark(id) => {
                    if id == task.thread().id() {
                        assert!(!guard.queue.contains_key(&task.id()));
                        guard.value = Value::Down;
                        break true;
                    }
                }
                Value::Down => {
                    if start + remaining <= stop {
                        assert!(guard.queue.remove(&task.id()).is_some());
                        break false;
                    }
                }
            }

            remaining -= stop - start;
        }
    }
}
