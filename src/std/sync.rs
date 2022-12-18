extern crate std;

use core::time::Duration;
use std::{
    mem::replace,
    sync::Mutex,
    thread::{self, Thread},
    time::Instant,
};

#[derive(Default)]
struct SemaphoreUnprotected {
    value: bool,
    waiter: Option<Thread>,
}

/// MPSC binary semaphore.
#[derive(Default)]
pub struct Semaphore {
    shared: Mutex<SemaphoreUnprotected>,
}

#[derive(Clone)]
pub struct SemaphoreProducer<'a> {
    shared: &'a Mutex<SemaphoreUnprotected>,
}

pub struct SemaphoreConsumer<'a> {
    shared: &'a Mutex<SemaphoreUnprotected>,
}

impl Semaphore {
    pub fn new() -> Self {
        Self {
            shared: Mutex::new(SemaphoreUnprotected {
                value: false,
                waiter: None,
            }),
        }
    }

    pub fn split(&mut self) -> (SemaphoreProducer<'_>, SemaphoreConsumer<'_>) {
        let shared = &self.shared;
        (SemaphoreProducer { shared }, SemaphoreConsumer { shared })
    }
}

impl<'a> SemaphoreProducer<'a> {
    /// Returns `true` on success.
    pub fn try_give(&self) -> bool {
        let mut guard = self.shared.lock().unwrap();
        let waiter = guard.waiter.take();
        let value = replace(&mut guard.value, true);
        drop(guard);
        if let Some(thread) = waiter {
            thread.unpark();
        }
        !value
    }
}

impl<'a> SemaphoreConsumer<'a> {
    /// Returns `true` on success.
    pub fn try_take(&mut self) -> bool {
        replace(&mut self.shared.lock().unwrap().value, false)
    }

    pub fn take(&mut self) {
        let mut guard = self.shared.lock().unwrap();
        if guard.value {
            guard.value = false;
            return;
        } else {
            debug_assert!(guard.waiter.replace(thread::current()).is_none());
        }
        drop(guard);

        loop {
            thread::park();

            let mut guard = self.shared.lock().unwrap();
            if guard.value {
                guard.value = false;
                debug_assert!(guard.waiter.is_none());
                break;
            }
        }
    }

    /// Returns `true` on success, `false` when timed out.
    pub fn take_timeout(&mut self, timeout: Duration) -> bool {
        let mut guard = self.shared.lock().unwrap();
        if guard.value {
            guard.value = false;
            return true;
        } else {
            assert!(guard.waiter.replace(thread::current()).is_none());
        }
        drop(guard);

        let mut remaining = timeout;
        loop {
            let start = Instant::now();
            thread::park_timeout(remaining);

            let stop = Instant::now();
            let mut guard = self.shared.lock().unwrap();
            if replace(&mut guard.value, false) {
                guard.waiter.take();
                break true;
            } else if start + remaining <= stop {
                guard.waiter.take();
                break false;
            }
            remaining -= stop - start;
        }
    }
}
