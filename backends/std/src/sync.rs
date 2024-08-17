extern crate std;

use crate::{
    error::Error,
    task::{BlockingContext, Context},
};
use core::{mem::replace, time::Duration};
use std::{
    sync::{Condvar, Mutex},
    time::Instant,
};

/// Binary semaphore.
pub struct Semaphore {
    value: Mutex<bool>,
    condvar: Condvar,
}

impl Semaphore {
    /// Create semaphore in empty state.
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            value: Mutex::new(false),
            condvar: Condvar::new(),
        })
    }

    /// Try to release semaphore.
    ///
    /// Returns `true` on success, `false` when already released.
    pub fn try_give<C: Context>(&self, _cx: &mut C) -> bool {
        let mut guard = self.value.lock().unwrap();
        let prev = replace(&mut *guard, true);
        self.condvar.notify_one();
        !prev
    }

    /// Try to acquire semaphore.
    ///
    /// Returns `true` on success, `false` when already acquired.
    pub fn try_take<C: Context>(&self, _cx: &mut C) -> bool {
        replace(&mut *self.value.lock().unwrap(), false)
    }

    /// Acquire semaphore.
    ///
    /// When `timeout` is `None` then wait infinitely.
    ///
    /// Returns `true` on success, `false` when timed out.
    pub fn take<C: BlockingContext>(&self, _cx: &mut C, timeout: Option<Duration>) -> bool {
        let mut guard_slot = Some(self.value.lock().unwrap());
        let instant = Instant::now();
        loop {
            let mut guard = guard_slot.take().unwrap();
            if replace(&mut *guard, false) {
                break true;
            }
            guard_slot.replace(match timeout {
                Some(total) => {
                    let current = instant.elapsed();
                    if current >= total {
                        break false;
                    }
                    self.condvar.wait_timeout(guard, total - current).unwrap().0
                }
                None => self.condvar.wait(guard).unwrap(),
            });
        }
    }
}
