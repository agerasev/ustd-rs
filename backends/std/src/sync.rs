extern crate std;

use crate::{
    error::Error,
    task::{BlockingContext, Context, TaskContext},
};
use core::{mem::replace, time::Duration};
use std::{
    io::ErrorKind,
    ops::{Deref, DerefMut},
    sync::{Condvar, Mutex as StdMutex, MutexGuard as StdMutexGuard, TryLockError},
    time::Instant,
};

/// Binary semaphore.
pub struct Semaphore {
    value: StdMutex<bool>,
    condvar: Condvar,
}

impl Semaphore {
    fn with_value(value: bool) -> Self {
        Self {
            value: StdMutex::new(value),
            condvar: Condvar::new(),
        }
    }

    /// Create semaphore in empty state.
    pub fn new() -> Result<Self, Error> {
        Ok(Self::with_value(false))
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

pub struct Mutex<T> {
    value: StdMutex<T>,
    sem: Semaphore,
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Result<Self, Error> {
        Ok(Self {
            value: StdMutex::new(value),
            sem: Semaphore::with_value(true),
        })
    }

    pub fn try_lock(&self, cx: &mut TaskContext) -> Result<Option<MutexGuard<'_, T>>, Error> {
        if self.sem.try_take(cx) {
            match self.value.try_lock() {
                Ok(guard) => Ok(Some(MutexGuard {
                    guard,
                    sem: &self.sem,
                })),
                Err(TryLockError::WouldBlock) => unreachable!(),
                Err(TryLockError::Poisoned(_)) => Err(Error::other("Poisoned mutex")),
            }
        } else {
            Ok(None)
        }
    }
    pub fn lock(
        &self,
        cx: &mut TaskContext,
        timeout: Option<Duration>,
    ) -> Result<MutexGuard<'_, T>, Error> {
        if self.sem.take(cx, timeout) {
            match self.value.try_lock() {
                Ok(guard) => Ok(MutexGuard {
                    guard,
                    sem: &self.sem,
                }),
                Err(TryLockError::WouldBlock) => unreachable!(),
                Err(TryLockError::Poisoned(_)) => Err(Error::other("Poisoned mutex")),
            }
        } else {
            Err(ErrorKind::TimedOut.into())
        }
    }
}

pub struct MutexGuard<'a, T> {
    guard: StdMutexGuard<'a, T>,
    sem: &'a Semaphore,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        assert!(self.sem.try_give(&mut TaskContext::current()));
    }
}
