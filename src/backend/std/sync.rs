extern crate std;

use super::task::is_interrupt;
use crate::error::Error;
use core::{mem::replace, time::Duration};
use std::{
    sync::{Condvar, Mutex},
    time::Instant,
};

/// MPMC binary semaphore.
pub(crate) struct Semaphore {
    value: Mutex<bool>,
    condvar: Condvar,
}

impl Semaphore {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            value: Mutex::new(false),
            condvar: Condvar::new(),
        })
    }

    pub fn give(&self) -> bool {
        let mut guard = self.value.lock().unwrap();
        let prev = replace(&mut *guard, true);
        self.condvar.notify_one();
        !prev
    }

    pub fn take(&self, timeout: Option<Duration>) -> bool {
        if is_interrupt() {
            assert_eq!(timeout, Some(Duration::ZERO))
        }
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
